use std::sync::Arc;
use base64::Engine;
use desktop_mcp::config::SecurityConfig;
use desktop_mcp::hal::macos::MacosDriver;
use desktop_mcp::hal::DesktopDriver;
use desktop_mcp::security::policy::PolicyEngine;
use desktop_mcp::server::dispatcher::McpDispatcher;
use desktop_mcp::server::protocol::JsonRpcRequest;

async fn click_button(name: &str, rel_x: f64, rel_y: f64, dispatcher: &McpDispatcher, bx: f64, by: f64) {
    let abs_x = bx + rel_x;
    let abs_y = by + rel_y;
    println!(">>> LLM -> MCP [Mouse Click '{}' at ({:.1}, {:.1})]:", name, abs_x, abs_y);
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(100)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_mouse_action",
            "arguments": {
                "action": "click",
                "coordinate": { "x": abs_x, "y": abs_y },
                "target_app": "Calculator"
            }
        })),
    };
    let _ = dispatcher.dispatch(req).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
}

#[tokio::main]
async fn main() {
    let driver = Arc::new(MacosDriver::new());
    let policy = Arc::new(PolicyEngine::new(SecurityConfig::default()));
    let dispatcher = McpDispatcher::new(driver.clone(), policy);

    println!("============================================================");
    println!("LLM DRIVING MCP SERVER VIA NATIVE MOUSE INPUT");
    println!("Goal: Compute (45 * 2) + 38 = 128 using GUI button clicks");
    println!("============================================================\n");

    // Step 1: Manage App -> Launch / Focus Calculator
    let req1 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_manage_app",
            "arguments": {
                "action": "launch",
                "app_identifier": "Calculator"
            }
        })),
    };
    println!(">>> LLM -> MCP [Step 1: Launch Calculator]");
    let res1 = dispatcher.dispatch(req1).await;
    let res1_val = res1.result.unwrap();
    let text = res1_val["content"][0]["text"].as_str().unwrap();
    let win_info: serde_json::Value = serde_json::from_str(text).unwrap();
    let bx = win_info["bounds"]["x"].as_f64().unwrap();
    let by = win_info["bounds"]["y"].as_f64().unwrap();
    let bw = win_info["bounds"]["width"].as_f64().unwrap();
    let bh = win_info["bounds"]["height"].as_f64().unwrap();
    println!("<<< MCP Response: Window bounds: ({}, {}) {}x{}\n", bx, by, bw, bh);

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Step 2: Click 'C' to clear
    println!("--- CLEARING CALCULATOR ---");
    click_button("C", 92.0, 165.0, &dispatcher, bx, by).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Capture cleared state
    let req_cap1 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(2)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_capture_screen",
            "arguments": { "target_app": "Calculator", "format": "png" }
        })),
    };
    let res_cap1 = dispatcher.dispatch(req_cap1).await;
    let b64_1 = serde_json::from_str::<serde_json::Value>(res_cap1.result.unwrap()["content"][0]["text"].as_str().unwrap()).unwrap()["base64_data"].as_str().unwrap().to_string();
    let bytes1 = base64::engine::general_purpose::STANDARD.decode(b64_1).unwrap();
    std::fs::write("/Users/sheldonl/.gemini/antigravity/brain/3b60d7dd-8f87-44d0-9db2-7ce1cf07c8cc/calc_step1_cleared.png", bytes1).unwrap();
    println!("Captured cleared state to brain/calc_step1_cleared.png\n");

    // Step 3: Perform sequence of clicks: 4 -> 5 -> × -> 2 -> + -> 3 -> 8 -> =
    println!("--- ENTERING EXPRESSION: (45 * 2) + 38 = ---");
    click_button("4", 38.0, 275.0, &dispatcher, bx, by).await;
    click_button("5", 92.0, 275.0, &dispatcher, bx, by).await;
    click_button("×", 200.0, 220.0, &dispatcher, bx, by).await;
    click_button("2", 92.0, 330.0, &dispatcher, bx, by).await;
    click_button("+", 200.0, 330.0, &dispatcher, bx, by).await;
    click_button("3", 146.0, 330.0, &dispatcher, bx, by).await;
    click_button("8", 92.0, 220.0, &dispatcher, bx, by).await;
    click_button("=", 200.0, 385.0, &dispatcher, bx, by).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Step 4: Capture final result state
    println!("\n--- CAPTURING FINAL RESULT ---");
    let req_cap2 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(3)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_capture_screen",
            "arguments": { "target_app": "Calculator", "format": "png" }
        })),
    };
    let res_cap2 = dispatcher.dispatch(req_cap2).await;
    let b64_2 = serde_json::from_str::<serde_json::Value>(res_cap2.result.unwrap()["content"][0]["text"].as_str().unwrap()).unwrap()["base64_data"].as_str().unwrap().to_string();
    let bytes2 = base64::engine::general_purpose::STANDARD.decode(b64_2).unwrap();
    std::fs::write("/Users/sheldonl/.gemini/antigravity/brain/3b60d7dd-8f87-44d0-9db2-7ce1cf07c8cc/calc_step2_result.png", bytes2).unwrap();
    println!("Captured final result to brain/calc_step2_result.png\n");

    // Step 5: Read display via Accessibility
    let tree = driver.inspect_ui(Some("Calculator"), 3).await.unwrap();
    println!("Calculator UI Tree after computation:");
    for child in &tree.children {
        println!("  role: {}, title: {:?}, val: {:?}", child.role, child.title, child.value);
    }
}
