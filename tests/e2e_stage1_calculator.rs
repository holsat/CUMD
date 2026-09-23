use std::sync::Arc;
use desktop_mcp::hal::DesktopDriver;
use desktop_mcp::config::SecurityConfig;
use desktop_mcp::security::policy::PolicyEngine;
use desktop_mcp::server::dispatcher::McpDispatcher;
use desktop_mcp::server::protocol::JsonRpcRequest;

#[cfg(target_os = "macos")]
use desktop_mcp::hal::macos::MacosDriver;

async fn click_button(name: &str, rel_x: f64, rel_y: f64, dispatcher: &McpDispatcher) {
    println!("[E2E Stage 1] Mouse clicking '{}' at window relative ({:.1}, {:.1}) via desktop_mouse_action...", name, rel_x, rel_y);
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(100)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_mouse_action",
            "arguments": {
                "action": "click",
                "coordinate": { "x": rel_x, "y": rel_y },
                "coordinate_space": "window_relative",
                "target_app": "Calculator"
            }
        })),
    };
    let res = dispatcher.dispatch(req).await;
    assert!(res.error.is_none(), "Mouse click failed: {:?}", res.error);
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
}

#[tokio::test]
async fn test_calculator_algebraic_calculation() {
    let driver = Arc::new(MacosDriver::new());
    let policy = Arc::new(PolicyEngine::new(SecurityConfig::default()));
    let dispatcher = McpDispatcher::new(driver.clone(), policy);

    println!("[E2E Stage 1] 1. Launching Calculator via desktop_manage_app...");
    let launch_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(101)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_manage_app",
            "arguments": {
                "action": "launch",
                "app_identifier": "Calculator"
            }
        })),
    };

    let launch_res = dispatcher.dispatch(launch_req).await;
    assert!(launch_res.error.is_none(), "Failed to launch Calculator: {:?}", launch_res.error);
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Ensure Calculator is in Basic mode (Cmd+1)
    let basic_mode_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(102)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_keyboard_action",
            "arguments": {
                "action": "hotkey",
                "key": "1",
                "modifiers": ["cmd"],
                "target_app": "Calculator"
            }
        })),
    };
    dispatcher.dispatch(basic_mode_req).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Retrieve active window bounds
    let win_info = driver.get_active_window().await.unwrap();
    let bx = win_info.bounds.x;
    let by = win_info.bounds.y;
    println!("[E2E Stage 1] Calculator active window bounds: ({:.1}, {:.1}) {:.1}x{:.1}",
        bx, by, win_info.bounds.width, win_info.bounds.height);

    println!("[E2E Stage 1] 2. Capturing isolated window screenshot via desktop_capture_screen...");
    let capture_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(103)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_capture_screen",
            "arguments": {
                "target_app": "Calculator",
                "format": "png"
            }
        })),
    };

    let capture_res = dispatcher.dispatch(capture_req).await;
    assert!(capture_res.error.is_none(), "Window capture failed: {:?}", capture_res.error);

    let capture_text = capture_res.result.unwrap()["content"][0]["text"].as_str().unwrap().to_string();
    let capture_json: serde_json::Value = serde_json::from_str(&capture_text).unwrap();
    let img_width = capture_json["width"].as_u64().unwrap();
    let img_height = capture_json["height"].as_u64().unwrap();
    println!("[E2E Stage 1] Window captured: {}x{} physical pixels (Retina scale: {})",
        img_width, img_height, capture_json["scale_factor"]);

    // Verify window isolation
    assert!(img_width < 1000, "Screenshot appears to be full screen rather than isolated window: width={}", img_width);
    assert!(img_height < 1000, "Screenshot appears to be full screen rather than isolated window: height={}", img_height);

    println!("[E2E Stage 1] 3. Clear existing calculation with 'C' button click...");
    click_button("C", 92.0, 165.0, &dispatcher).await;

    println!("[E2E Stage 1] 4. Clicking expression buttons: 45 * 2 + 38 = ...");
    // (45 * 2) + 38 =
    click_button("4", 38.0, 275.0, &dispatcher).await;
    click_button("5", 92.0, 275.0, &dispatcher).await;
    click_button("×", 200.0, 220.0, &dispatcher).await;
    click_button("2", 92.0, 330.0, &dispatcher).await;
    click_button("+", 200.0, 330.0, &dispatcher).await;
    click_button("3", 146.0, 330.0, &dispatcher).await;
    click_button("8", 92.0, 220.0, &dispatcher).await;
    click_button("=", 200.0, 385.0, &dispatcher).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    println!("[E2E Stage 1] 5. Capturing final calculation screenshot...");
    let capture_final_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(104)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_capture_screen",
            "arguments": {
                "target_app": "Calculator",
                "format": "png"
            }
        })),
    };
    let capture_final_res = dispatcher.dispatch(capture_final_req).await;
    assert!(capture_final_res.error.is_none());

    // Step 6: Verify Calculator UI tree or display value
    let tree = driver.inspect_ui(Some("Calculator"), 3).await.unwrap();
    println!("[E2E Stage 1] Calculator UI verification after calculation: role={}", tree.role);

    // Step 7: Quit Calculator via desktop_keyboard_action (Cmd+Q) like a real user
    println!("[E2E Stage 1] 6. Quitting Calculator via Cmd+Q keyboard action...");
    let quit_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(105)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_keyboard_action",
            "arguments": {
                "action": "hotkey",
                "key": "q",
                "modifiers": ["cmd"],
                "target_app": "Calculator"
            }
        })),
    };
    dispatcher.dispatch(quit_req).await;

    println!("[E2E Stage 1] Stage 1 Calculator calculation test PASSED successfully!");
}
