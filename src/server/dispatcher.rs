use std::sync::Arc;
use serde_json::Value;
use crate::hal::driver::*;
use crate::security::policy::PolicyEngine;
use crate::server::protocol::{get_available_tools, JsonRpcError, JsonRpcRequest, JsonRpcResponse};

pub struct McpDispatcher {
    driver: Arc<dyn DesktopDriver>,
    policy: Arc<PolicyEngine>,
}

impl McpDispatcher {
    pub fn new(driver: Arc<dyn DesktopDriver>, policy: Arc<PolicyEngine>) -> Self {
        Self { driver, policy }
    }

    pub async fn dispatch(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();
        match request.method.as_str() {
            "initialize" => {
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::json!({
                        "protocolVersion": "2024-11-05",
                        "serverInfo": {
                            "name": "desktop-mcp-daemon",
                            "version": crate::version(),
                        },
                        "capabilities": {
                            "tools": {}
                        }
                    })),
                    error: None,
                }
            }
            "tools/list" => {
                let tools = get_available_tools();
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::json!({ "tools": tools })),
                    error: None,
                }
            }
            "tools/call" => {
                let params = request.params.unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

                match self.handle_tool_call(tool_name, arguments).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                            }]
                        })),
                        error: None,
                    },
                    Err(err_msg) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: err_msg,
                            data: None,
                        }),
                    },
                }
            }
            other => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", other),
                    data: None,
                }),
            },
        }
    }

    async fn handle_tool_call(&self, name: &str, args: Value) -> Result<Value, String> {
        match name {
            "desktop_capture_screen" => {
                let target_app = args.get("target_app").and_then(|v| v.as_str());
                let window_id = args.get("window_id").and_then(|v| v.as_u64());

                if let Some(app) = target_app {
                    self.policy.validate_app(app).map_err(|e| e.to_string())?;
                }

                let img = self.driver.capture_window(target_app, window_id).await
                    .map_err(|e| e.to_string())?;

                Ok(serde_json::to_value(img).unwrap())
            }
            "desktop_mouse_action" => {
                let action_str = args.get("action").and_then(|v| v.as_str()).unwrap_or("move");
                let coord = args.get("coordinate").ok_or("Missing coordinate")?;
                let x = coord.get("x").and_then(|v| v.as_f64()).ok_or("Missing x coordinate")?;
                let y = coord.get("y").and_then(|v| v.as_f64()).ok_or("Missing y coordinate")?;
                let target_app = args.get("target_app").and_then(|v| v.as_str());
                let window_id = args.get("window_id").and_then(|v| v.as_u64());
                let coordinate_space = args.get("coordinate_space").and_then(|v| v.as_str());

                if let Some(app) = target_app {
                    self.policy.validate_app(app).map_err(|e| e.to_string())?;
                }

                let action = match action_str {
                    "click" => MouseAction::Click,
                    "double_click" => MouseAction::DoubleClick,
                    "triple_click" => MouseAction::TripleClick,
                    "right_click" => MouseAction::RightClick,
                    "drag" => MouseAction::Drag,
                    "scroll" => MouseAction::Scroll,
                    _ => MouseAction::Move,
                };

                let button = match args.get("button").and_then(|v| v.as_str()) {
                    Some("right") => MouseButton::Right,
                    Some("middle") => MouseButton::Middle,
                    _ => MouseButton::Left,
                };

                let click_count = args.get("click_count").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                let scroll_delta = args.get("scroll_delta").and_then(|v| {
                    let dx = v.get("dx").and_then(|x| x.as_i64()).unwrap_or(0) as i32;
                    let dy = v.get("dy").and_then(|y| y.as_i64()).unwrap_or(0) as i32;
                    Some((dx, dy))
                });

                let (final_x, final_y, effective_coord_space) = if coordinate_space == Some("normalized_1000") {
                    let (w, h) = if let Ok(win) = self.driver.get_active_window().await {
                        (win.bounds.width, win.bounds.height)
                    } else {
                        (1920.0, 1080.0)
                    };
                    ((x / 1000.0) * w, (y / 1000.0) * h, Some("window_relative"))
                } else {
                    (x, y, coordinate_space)
                };

                self.driver.mouse_action(
                    action,
                    final_x,
                    final_y,
                    click_count,
                    button,
                    scroll_delta,
                    target_app,
                    window_id,
                    effective_coord_space,
                ).await.map_err(|e| e.to_string())?;

                let mut res = serde_json::json!({
                    "status": "success",
                    "action": action_str,
                    "x": final_x,
                    "y": final_y
                });
                if args.get("include_screenshot").and_then(|v| v.as_bool()).unwrap_or(false) {
                    if let Ok(img) = self.driver.capture_window(target_app, window_id).await {
                        res["screenshot"] = serde_json::to_value(img).unwrap();
                    }
                }
                Ok(res)
            }
            "desktop_keyboard_action" => {
                let action_str = args.get("action").and_then(|v| v.as_str()).unwrap_or("type");
                let text = args.get("text").and_then(|v| v.as_str());
                let key = args.get("key").and_then(|v| v.as_str());
                let target_app = args.get("target_app").and_then(|v| v.as_str());
                let window_id = args.get("window_id").and_then(|v| v.as_u64());

                if let Some(app) = target_app {
                    self.policy.validate_app(app).map_err(|e| e.to_string())?;
                }

                let modifiers: Vec<String> = args.get("modifiers")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let action = match action_str {
                    "hotkey" => KeyAction::Hotkey,
                    "press_key" => KeyAction::PressKey,
                    "key_down" => KeyAction::KeyDown,
                    "key_up" => KeyAction::KeyUp,
                    _ => KeyAction::Type,
                };

                self.driver.keyboard_action(
                    action,
                    text,
                    key,
                    &modifiers,
                    target_app,
                    window_id,
                ).await.map_err(|e| e.to_string())?;

                let mut res = serde_json::json!({ "status": "success", "action": action_str });
                if args.get("include_screenshot").and_then(|v| v.as_bool()).unwrap_or(false) {
                    if let Ok(img) = self.driver.capture_window(target_app, window_id).await {
                        res["screenshot"] = serde_json::to_value(img).unwrap();
                    }
                }
                Ok(res)
            }
            "desktop_inspect_ui" => {
                let app = args.get("app_identifier").and_then(|v| v.as_str());
                let max_depth = args.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(3) as u32;

                if let Some(a) = app {
                    self.policy.validate_app(a).map_err(|e| e.to_string())?;
                }

                let tree = self.driver.inspect_ui(app, max_depth).await
                    .map_err(|e| e.to_string())?;

                Ok(serde_json::to_value(tree).unwrap())
            }
            "desktop_manage_app" => {
                let app_id = args.get("app_identifier").and_then(|v| v.as_str())
                    .ok_or("Missing app_identifier")?;
                let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("launch");

                self.policy.validate_app(app_id).map_err(|e| e.to_string())?;

                if action == "quit" {
                    self.driver.terminate_app(app_id).await.map_err(|e| e.to_string())?;
                    Ok(serde_json::json!({ "status": "success", "action": "quit", "app_identifier": app_id }))
                } else {
                    let win = self.driver.launch_or_focus_app(app_id).await
                        .map_err(|e| e.to_string())?;
                    Ok(serde_json::to_value(win).unwrap())
                }
            }
            "desktop_doctor" => {
                let report = self.driver.check_permissions().await
                    .map_err(|e| e.to_string())?;

                Ok(serde_json::to_value(report).unwrap())
            }
            "desktop_create_virtual_display" => {
                let width = args.get("width").and_then(|v| v.as_u64()).unwrap_or(1920) as u32;
                let height = args.get("height").and_then(|v| v.as_u64()).unwrap_or(1080) as u32;
                let name = args.get("name").and_then(|v| v.as_str());

                let display = self.driver.create_virtual_display(width, height, name).await
                    .map_err(|e| e.to_string())?;

                Ok(serde_json::to_value(display).unwrap())
            }
            "desktop_destroy_virtual_display" => {
                let display_id = args.get("display_id")
                    .and_then(|v| v.as_u64())
                    .ok_or("Missing display_id")? as u32;

                self.driver.destroy_virtual_display(display_id).await
                    .map_err(|e| e.to_string())?;

                Ok(serde_json::json!({ "status": "success", "destroyed_display_id": display_id }))
            }
            "desktop_list_displays" => {
                let displays = self.driver.list_displays().await
                    .map_err(|e| e.to_string())?;

                Ok(serde_json::to_value(displays).unwrap())
            }
            other => Err(format!("Unknown tool: {}", other)),
        }
    }
}
