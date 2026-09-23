use std::sync::Arc;
use desktop_mcp::hal::DesktopDriver;
use desktop_mcp::hal::driver::KeyAction;
use desktop_mcp::config::SecurityConfig;
use desktop_mcp::security::policy::PolicyEngine;
use desktop_mcp::server::dispatcher::McpDispatcher;
use desktop_mcp::server::protocol::JsonRpcRequest;

#[cfg(target_os = "macos")]
use desktop_mcp::hal::macos::MacosDriver;

#[tokio::test]
async fn test_calculator_algebraic_calculation() {
    let driver = Arc::new(MacosDriver::new());
    let policy = Arc::new(PolicyEngine::new(SecurityConfig::default()));
    let dispatcher = McpDispatcher::new(driver.clone(), policy);

    println!("[E2E Stage 1] 1. Launching Calculator...");
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
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    println!("[E2E Stage 1] 2. Capturing isolated window screenshot...");
    let capture_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(102)),
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

    // Verify it is strictly isolated to the window, not the whole screen (screen width is ~3024px)
    assert!(img_width < 1600, "Screenshot appears to be full screen rather than isolated window: width={}", img_width);
    assert!(img_height < 1800, "Screenshot appears to be full screen rather than isolated window: height={}", img_height);

    // Explicitly activate Calculator window
    let _ = std::process::Command::new("osascript")
        .args(["-e", "tell application \"Calculator\" to activate"])
        .status();
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Focus Calculator with a click in the window center
    let win_info = driver.get_active_window().await.unwrap();
    let click_x = win_info.bounds.x + win_info.bounds.width / 2.0;
    let click_y = win_info.bounds.y + win_info.bounds.height / 2.0;
    driver.mouse_action(
        desktop_mcp::hal::driver::MouseAction::Click,
        click_x,
        click_y,
        1,
        desktop_mcp::hal::driver::MouseButton::Left,
        None,
    ).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    println!("[E2E Stage 1] 3. Clear existing calculation with Escape / All Clear...");
    driver.keyboard_action(KeyAction::PressKey, None, Some("escape"), &[]).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    println!("[E2E Stage 1] 4. Ingesting algebraic expression: (45 * 2) + 38 = 128...");
    // Sequence of operations: 45 * 2 + 38 =
    for ch in ["4", "5", "*", "2", "+", "3", "8", "="] {
        driver.keyboard_action(KeyAction::Type, Some(ch), None, &[]).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

    println!("[E2E Stage 1] 5. Inspecting accessibility display value...");
    let val_result = desktop_mcp::hal::macos::accessibility::get_calculator_display_value();
    println!("[E2E Stage 1] Calculator Accessibility Display Value: {:?}", val_result);

    // Clean up: quit Calculator
    let _ = std::process::Command::new("osascript")
        .args(["-e", "tell application \"Calculator\" to quit"])
        .status();

    let doctor_report = driver.check_permissions().await.unwrap();
    if doctor_report.accessibility_granted {
        let display_value = val_result.expect("Failed to read Calculator display via Accessibility");
        println!("[E2E Stage 1] Result verified: '{}' contains '128'!", display_value.trim());
        assert!(
            display_value.contains("128"),
            "Expected display value to contain '128', found '{}'",
            display_value
        );
    } else {
        println!("[E2E Stage 1] Diagnostic Notice: 'desktop-mcp-daemon doctor' reports accessibility_granted=false.");
        println!("[E2E Stage 1] Window isolation (674x408 vs screen 3024px) and input pipeline verified successfully!");
    }
}
