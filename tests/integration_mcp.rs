use std::sync::Arc;
use desktop_mcp::config::{SecurityConfig, SecurityMode};
use desktop_mcp::security::policy::PolicyEngine;
use desktop_mcp::server::dispatcher::McpDispatcher;
use desktop_mcp::server::protocol::JsonRpcRequest;

#[cfg(target_os = "macos")]
use desktop_mcp::hal::macos::MacosDriver;

#[tokio::test]
async fn test_mcp_initialize_and_tools_list() {
    let driver = Arc::new(MacosDriver::new());
    let policy = Arc::new(PolicyEngine::new(SecurityConfig::default()));
    let dispatcher = McpDispatcher::new(driver, policy);

    // 1. Initialize
    let init_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: None,
    };
    let init_res = dispatcher.dispatch(init_req).await;
    assert!(init_res.error.is_none());
    let res_val = init_res.result.expect("No result");
    assert_eq!(res_val["protocolVersion"], "2024-11-05");
    assert_eq!(res_val["serverInfo"]["name"], "desktop-mcp-daemon");

    // 2. Tools list
    let list_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(2)),
        method: "tools/list".to_string(),
        params: None,
    };
    let list_res = dispatcher.dispatch(list_req).await;
    assert!(list_res.error.is_none());
    let tools = list_res.result.unwrap()["tools"].as_array().unwrap().clone();
    assert_eq!(tools.len(), 9);

    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"desktop_capture_screen"));
    assert!(tool_names.contains(&"desktop_mouse_action"));
    assert!(tool_names.contains(&"desktop_keyboard_action"));
    assert!(tool_names.contains(&"desktop_inspect_ui"));
    assert!(tool_names.contains(&"desktop_manage_app"));
    assert!(tool_names.contains(&"desktop_doctor"));
    assert!(tool_names.contains(&"desktop_create_virtual_display"));
    assert!(tool_names.contains(&"desktop_destroy_virtual_display"));
    assert!(tool_names.contains(&"desktop_list_displays"));
}

#[tokio::test]
async fn test_mcp_policy_blocking_terminal() {
    let driver = Arc::new(MacosDriver::new());
    let policy = Arc::new(PolicyEngine::new(SecurityConfig {
        mode: SecurityMode::Allowlist,
        enforce_focus_lock: true,
        allowed_apps: vec!["Calculator".to_string()],
        blocked_apps: vec!["com.apple.Terminal".to_string()],
    }));
    let dispatcher = McpDispatcher::new(driver, policy);

    let call_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(3)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_manage_app",
            "arguments": {
                "action": "launch",
                "app_identifier": "com.apple.Terminal"
            }
        })),
    };

    let call_res = dispatcher.dispatch(call_req).await;
    assert!(call_res.result.is_none());
    let err = call_res.error.expect("Expected security error");
    assert!(err.message.contains("SecurityPolicyViolation"));
    assert!(err.message.contains("explicitly blocked"));
}
