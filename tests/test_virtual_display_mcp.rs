use std::sync::Arc;
use desktop_mcp::config::SecurityConfig;
use desktop_mcp::security::policy::PolicyEngine;
use desktop_mcp::server::dispatcher::McpDispatcher;
use desktop_mcp::server::protocol::JsonRpcRequest;

#[cfg(target_os = "macos")]
use desktop_mcp::hal::macos::MacosDriver;

#[tokio::test]
async fn test_virtual_display_lifecycle_and_best_practices() {
    let driver = Arc::new(MacosDriver::new());
    let policy = Arc::new(PolicyEngine::new(SecurityConfig::default()));
    let dispatcher = McpDispatcher::new(driver.clone(), policy);

    // 1. Test listing initial displays
    println!("[Test] 1. Listing initial displays...");
    let list_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_list_displays",
            "arguments": {}
        })),
    };
    let list_res = dispatcher.dispatch(list_req).await;
    assert!(list_res.error.is_none());
    let text = list_res.result.unwrap()["content"][0]["text"].as_str().unwrap().to_string();
    let initial_displays: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    println!("[Test] Initial online displays count: {}", initial_displays.len());
    assert!(!initial_displays.is_empty(), "Should detect at least one active display");

    // 2. Test creating a virtual display (1920x1080)
    println!("[Test] 2. Creating virtual display 1920x1080...");
    let create_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(2)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_create_virtual_display",
            "arguments": {
                "width": 1920,
                "height": 1080,
                "name": "Test-Headless-Monitor"
            }
        })),
    };
    let create_res = dispatcher.dispatch(create_req).await;
    assert!(create_res.error.is_none(), "Virtual display creation failed: {:?}", create_res.error);
    let create_text = create_res.result.unwrap()["content"][0]["text"].as_str().unwrap().to_string();
    let display_info: serde_json::Value = serde_json::from_str(&create_text).unwrap();
    let v_id = display_info["display_id"].as_u64().expect("Should return numeric display_id");
    println!("[Test] Created virtual display ID: {}", v_id);
    assert_eq!(display_info["width"], 1920);
    assert_eq!(display_info["height"], 1080);
    assert_eq!(display_info["is_virtual"], true);

    // 3. Verify the new virtual display appears in desktop_list_displays
    println!("[Test] 3. Verifying virtual display appears in list_displays...");
    let list_req2 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(3)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_list_displays",
            "arguments": {}
        })),
    };
    let list_res2 = dispatcher.dispatch(list_req2).await;
    let text2 = list_res2.result.unwrap()["content"][0]["text"].as_str().unwrap().to_string();
    let updated_displays: Vec<serde_json::Value> = serde_json::from_str(&text2).unwrap();
    let found = updated_displays.iter().any(|d| d["display_id"] == v_id && d["is_virtual"] == true);
    assert!(found, "New virtual display {} must be in list of displays", v_id);
    println!("[Test] Virtual display {} verified active in system display list!", v_id);

    // 4. Test normalized 0-1000 coordinate space in mouse action
    println!("[Test] 4. Testing normalized 0-1000 coordinate space mouse move...");
    let move_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(4)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_mouse_action",
            "arguments": {
                "action": "move",
                "coordinate": {
                    "x": 500.0,
                    "y": 500.0
                },
                "coordinate_space": "normalized_1000"
            }
        })),
    };
    let move_res = dispatcher.dispatch(move_req).await;
    assert!(move_res.error.is_none());
    println!("[Test] Normalized 0-1000 coordinate move succeeded!");

    // 5. Test bundled screenshot in mouse action
    println!("[Test] 5. Testing bundled screenshot in mouse action...");
    let click_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(5)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_mouse_action",
            "arguments": {
                "action": "move",
                "coordinate": {
                    "x": 100.0,
                    "y": 100.0
                },
                "coordinate_space": "screen_absolute",
                "include_screenshot": true
            }
        })),
    };
    let click_res = dispatcher.dispatch(click_req).await;
    assert!(click_res.error.is_none());
    let click_text = click_res.result.unwrap()["content"][0]["text"].as_str().unwrap().to_string();
    let click_json: serde_json::Value = serde_json::from_str(&click_text).unwrap();
    assert!(click_json.get("screenshot").is_some(), "Should bundle screenshot in tool response");
    println!("[Test] Bundled screenshot successfully returned in response!");

    // 6. Test destroying the virtual display
    println!("[Test] 6. Destroying virtual display {}...", v_id);
    let destroy_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(6)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_destroy_virtual_display",
            "arguments": {
                "display_id": v_id
            }
        })),
    };
    let destroy_res = dispatcher.dispatch(destroy_req).await;
    assert!(destroy_res.error.is_none());
    println!("[Test] Virtual display {} successfully destroyed!", v_id);

    // 7. Verify virtual display is gone
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let list_res3 = dispatcher.dispatch(JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(7)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_list_displays",
            "arguments": {}
        })),
    }).await;
    let text3 = list_res3.result.unwrap()["content"][0]["text"].as_str().unwrap().to_string();
    let final_displays: Vec<serde_json::Value> = serde_json::from_str(&text3).unwrap();
    let still_found = final_displays.iter().any(|d| d["display_id"] == v_id);
    assert!(!still_found, "Destroyed virtual display {} should no longer be in display list", v_id);
    println!("[Test] Virtual display clean teardown verified!");
}
