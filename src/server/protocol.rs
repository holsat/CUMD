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
            description: "Performs mouse movements, clicks, double clicks, right clicks, drags, and scrolls".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["move", "click", "double_click", "triple_click", "right_click", "drag", "scroll"] },
                    "coordinate": {
                        "type": "object",
                        "properties": {
                            "x": { "type": "number" },
                            "y": { "type": "number" }
                        },
                        "required": ["x", "y"]
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
                    "target_app": { "type": "string" }
                },
                "required": ["action", "coordinate"]
            }),
        },
        McpTool {
            name: "desktop_keyboard_action".to_string(),
            description: "Sends keystrokes, unicode text, or hotkey combinations to the focused application".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["type", "press_key", "hotkey"] },
                    "text": { "type": "string" },
                    "key": { "type": "string" },
                    "modifiers": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "target_app": { "type": "string" }
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
            description: "Launches or brings an approved desktop application to the front".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["launch", "focus"] },
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
    ]
}
