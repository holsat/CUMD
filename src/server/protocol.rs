use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub fn get_available_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "desktop_capture_screen".to_string(),
            description: "Captures a window-isolated screenshot of an approved desktop application".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "target_app": { "type": "string", "description": "Target application name or bundle ID" },
                    "window_id": { "type": "integer", "description": "Optional specific window ID" },
                    "format": { "type": "string", "enum": ["png", "jpeg"], "default": "png" }
                }
            }),
        },
        McpTool {
            name: "desktop_mouse_action".to_string(),
            description: "Performs mouse movements, clicks, double clicks, right clicks, drags, and scrolls with window-relative coordinates".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["move", "click", "double_click", "triple_click", "right_click", "drag", "scroll"] },
                    "coordinate": {
                        "type": "object",
                        "properties": {
                            "x": { "type": "number", "description": "X coordinate in window (or screen if screen_absolute)" },
                            "y": { "type": "number", "description": "Y coordinate in window (or screen if screen_absolute)" }
                        },
                        "required": ["x", "y"]
                    },
                    "coordinate_space": {
                        "type": "string",
                        "enum": ["window_relative", "screen_absolute", "normalized_1000"],
                        "default": "window_relative",
                        "description": "Coordinate system. 'window_relative' (default), 'screen_absolute', or 'normalized_1000' (0-1000 scale)"
                    },
                    "include_screenshot": {
                        "type": "boolean",
                        "default": false,
                        "description": "If true, bundle a fresh screenshot of the target window/screen in the tool response"
                    },
                    "button": { "type": "string", "enum": ["left", "right", "middle"], "default": "left" },
                    "click_count": { "type": "integer", "default": 1 },
                    "scroll_delta": {
                        "type": "object",
                        "properties": {
                            "dx": { "type": "integer" },
                            "dy": { "type": "integer" }
                        }
                    },
                    "target_app": { "type": "string", "description": "Target application name to deliver input to and calculate relative coordinates" },
                    "window_id": { "type": "integer", "description": "Optional specific window ID" }
                },
                "required": ["action", "coordinate"]
            }),
        },
        McpTool {
            name: "desktop_keyboard_action".to_string(),
            description: "Sends keystrokes, unicode text, key down/up, or hotkey combinations to the targeted application".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["type", "press_key", "hotkey", "key_down", "key_up"] },
                    "text": { "type": "string", "description": "Text to type into the target app" },
                    "key": { "type": "string", "description": "Key name (e.g. 'return', 'escape', 'tab', 'a', 's')" },
                    "modifiers": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Modifier keys (e.g. ['cmd'], ['shift'], ['alt'], ['ctrl'])"
                    },
                    "include_screenshot": {
                        "type": "boolean",
                        "default": false,
                        "description": "If true, bundle a fresh screenshot of the target window/screen in the tool response"
                    },
                    "target_app": { "type": "string", "description": "Target application name" },
                    "window_id": { "type": "integer", "description": "Optional specific window ID" }
                },
                "required": ["action"]
            }),
        },
        McpTool {
            name: "desktop_inspect_ui".to_string(),
            description: "Inspects the accessibility tree (roles, titles, values) of the targeted application".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "app_identifier": { "type": "string" },
                    "max_depth": { "type": "integer", "default": 3 }
                }
            }),
        },
        McpTool {
            name: "desktop_manage_app".to_string(),
            description: "Launches, focuses, or terminates an approved desktop application".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["launch", "focus", "quit"] },
                    "app_identifier": { "type": "string" }
                },
                "required": ["action", "app_identifier"]
            }),
        },
        McpTool {
            name: "desktop_doctor".to_string(),
            description: "Performs diagnostic health check of OS permissions, active displays, and security policy".to_string(),
            input_schema: serde_json::json!({
                "type": "object"
            }),
        },
        McpTool {
            name: "desktop_create_virtual_display".to_string(),
            description: "Creates an isolated software virtual display head (macOS CGVirtualDisplay / Linux headless) to run agent tasks without disturbing the user's desktop".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "width": { "type": "integer", "default": 1920, "description": "Width in physical pixels" },
                    "height": { "type": "integer", "default": 1080, "description": "Height in physical pixels" },
                    "name": { "type": "string", "description": "Optional virtual display name" }
                }
            }),
        },
        McpTool {
            name: "desktop_destroy_virtual_display".to_string(),
            description: "Destroys an active virtual software display head given its display_id".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "display_id": { "type": "integer", "description": "Virtual display ID to tear down" }
                },
                "required": ["display_id"]
            }),
        },
        McpTool {
            name: "desktop_list_displays".to_string(),
            description: "Lists all active physical and virtual displays with their IDs, resolutions, and properties".to_string(),
            input_schema: serde_json::json!({
                "type": "object"
            }),
        },
    ]
}
