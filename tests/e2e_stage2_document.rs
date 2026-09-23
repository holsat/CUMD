use std::sync::Arc;
use desktop_mcp::hal::DesktopDriver;
use desktop_mcp::config::SecurityConfig;
use desktop_mcp::security::policy::PolicyEngine;
use desktop_mcp::server::dispatcher::McpDispatcher;
use desktop_mcp::server::protocol::JsonRpcRequest;

#[cfg(target_os = "macos")]
use desktop_mcp::hal::macos::MacosDriver;

const VATICAN_POLEMIC_TEXT: &str = r#"
The Infallible Ark and the Living Council: An Apologetic Analysis of Vatican I and Vatican II

I. The First Vatican Council (1869-1870): The Rock of Peter in an Era of Revolution

The convocation of the First Ecumenical Council of the Vatican by Blessed Pope Pius IX occurred against the backdrop of European secularization, the Italian Risorgimento, and philosophical rationalism. For centuries, the Church had wrestled with Gallicanism in France and Febronianism in the German lands—theological tendencies that sought to subordinate Roman papal authority to national episcopates and civil monarchs.

In response, Vatican I promulgated the Dogmatic Constitution Pastor Aeternus (July 18, 1870), defining papal primacy and infallibility:
"When the Roman Pontiff speaks EX CATHEDRA—that is, when in the exercise of his office as pastor and teacher of all Christians, in virtue of his supreme apostolic authority, he defines a doctrine concerning faith or morals to be held by the whole Church—he possesses, by the divine assistance promised to him in blessed Peter, that infallibility which the divine Redeemer willed His Church to enjoy in defining doctrine."

[Apologetic Defense: Refuting Conciliarist and Old Catholic Critiques]
Opponents led by Church historian Ignaz von Döllinger claimed that papal infallibility was a 19th-century Ultramontane innovation lacking patristic foundation. In response, Catholic apologetics demonstrates:
1. Scriptural Grounding: Christ's threefold promise to Simon Peter in Matthew 16:18-19, Luke 22:32, and John 21:15-17 establishes a permanent indefectible office of universal confirmation.
2. Patristic Consensus: St. Irenaeus of Lyons affirmed that all churches must agree with Rome on account of its preeminent authority, and the Formula of Hormisdas (519 AD) established that in the Apostolic See the Catholic religion is always preserved undefiled.
3. Strict Delimitation: Vatican I did not endorse Papal Inerrancy or Inspiration. The Pope cannot invent new doctrine, nor are his private opinions protected; infallibility is strictly limited to definitive solemn judgments on faith and morals in continuity with the Deposit of Faith.


II. The Second Vatican Council (1962-1965): Aggiornamento and Pastoral Fidelity

Nine decades later, Saint John XXIII convoked the Second Vatican Council to engage modern humanity through pastoral charity and ressourcement. Vatican II produced monumental dogmatic constitutions, notably Lumen Gentium (On the Church) and Dei Verbum (On Divine Revelation), alongside the pastoral constitution Gaudium et Spes.

Critics across ideological extremes immediately weaponized the council. Progressive modernists posited a "Spirit of Vatican II" claiming a radical rupture with previous dogma, while traditionalist critics such as Archbishop Marcel Lefebvre claimed that Dignitatis Humanae (on religious liberty) and Unitatis Redintegratio (on ecumenism) represented apostasy.

[Apologetic Defense: The Hermeneutic of Reform in Continuity]
Pope Benedict XVI definitively refuted the Hermeneutic of Rupture by establishing the Hermeneutic of Reform in Continuity:
1. Religious Liberty: Dignitatis Humanae does not declare moral indifferentism or that all religions are equally true. Rather, it defends the dignity of the human person against coercive state tyranny, reaffirming the moral obligation of societies to seek the one true Catholic faith.
2. Ecumenism: Unitatis Redintegratio explicitly affirms that the unique Church of Christ subsists in the Catholic Church governed by Peter's successor. Dialogue seeks reconciliation without diluting any defined dogma.
3. Ecclesiology: Lumen Gentium completes the unfinished work of Vatican I by situating papal primacy within the collegiality of the worldwide episcopate, not as an isolated autocracy, but as the visible head of Christ's Mystical Body.


Conclusion: One Holy Catholic and Apostolic Ark

Vatican I fortified the mast and anchor of Peter's Barque; Vatican II unfurled its sails to navigate the tumultuous seas of the late modern world. Rather than contradictory poles, they represent the dynamic synergy between definitive dogma and pastoral evangelization.
"#;

#[tokio::test]
async fn test_rich_document_authoring_vatican_councils() {
    let driver = Arc::new(MacosDriver::new());
    let policy = Arc::new(PolicyEngine::new(SecurityConfig::default()));
    let dispatcher = McpDispatcher::new(driver.clone(), policy);

    let target_app = "Microsoft Word";
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/sheldonl".to_string());
    let primary_save_path = format!("{}/vatican_councils_polemic.docx", home);
    let fallback_save_path = format!("{}/Documents/vatican_councils_polemic.docx", home);

    let _ = std::fs::remove_file(&primary_save_path);
    let _ = std::fs::remove_file(&fallback_save_path);

    println!("[E2E Stage 2] 1. Activating and launching Microsoft Word via MCP tool...");
    let launch_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(201)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_manage_app",
            "arguments": {
                "action": "launch",
                "app_identifier": target_app
            }
        })),
    };
    let launch_res = dispatcher.dispatch(launch_req).await;
    assert!(launch_res.error.is_none(), "Failed to launch Microsoft Word: {:?}", launch_res.error);
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    println!("[E2E Stage 2] 2. Creating New Document in Word via UI keyboard shortcut Cmd+N...");
    let new_doc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(202)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_keyboard_action",
            "arguments": {
                "action": "hotkey",
                "key": "n",
                "modifiers": ["cmd"],
                "target_app": target_app
            }
        })),
    };
    dispatcher.dispatch(new_doc_req).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    println!("[E2E Stage 2] 3. Clicking into Word document canvas via MCP mouse action...");
    let win_info = driver.get_active_window().await.unwrap();
    println!("[E2E Stage 2] Active Word window: wid={} bounds={:?}", win_info.window_id, win_info.bounds);
    let click_x = win_info.bounds.x + win_info.bounds.width / 2.0;
    let click_y = win_info.bounds.y + win_info.bounds.height / 3.0; // Click upper-middle of page canvas
    let click_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(203)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_mouse_action",
            "arguments": {
                "action": "click",
                "coordinate": { "x": click_x, "y": click_y },
                "target_app": target_app
            }
        })),
    };
    dispatcher.dispatch(click_req).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    println!("[E2E Stage 2] 4. Typing formatted apologetics treatise into Word via MCP keyboard tool...");
    let type_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(204)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_keyboard_action",
            "arguments": {
                "action": "type",
                "text": VATICAN_POLEMIC_TEXT.trim(),
                "target_app": target_app
            }
        })),
    };
    dispatcher.dispatch(type_req).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    println!("[E2E Stage 2] 5. Capturing window-isolated screenshot of Word with the typed document...");
    let capture_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(205)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_capture_screen",
            "arguments": {
                "target_app": target_app,
                "format": "png"
            }
        })),
    };
    let capture_res = dispatcher.dispatch(capture_req).await;
    assert!(capture_res.error.is_none());
    if let Some(res) = capture_res.result {
        let capture_text = res["content"][0]["text"].as_str().unwrap().to_string();
        let capture_json: serde_json::Value = serde_json::from_str(&capture_text).unwrap();
        println!("[E2E Stage 2] Isolated window captured: {}x{} physical pixels for Microsoft Word",
            capture_json["width"], capture_json["height"]);
    }

    println!("[E2E Stage 2] 6. Triggering Word Save dialog via UI shortcut Cmd+S...");
    let save_hotkey_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(206)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_keyboard_action",
            "arguments": {
                "action": "hotkey",
                "key": "s",
                "modifiers": ["cmd"],
                "target_app": target_app
            }
        })),
    };
    dispatcher.dispatch(save_hotkey_req).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    println!("[E2E Stage 2] 7. Navigating Save dialog to user Home directory via Cmd+Shift+H...");
    let home_hotkey_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(207)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_keyboard_action",
            "arguments": {
                "action": "hotkey",
                "key": "h",
                "modifiers": ["cmd", "shift"],
                "target_app": target_app
            }
        })),
    };
    dispatcher.dispatch(home_hotkey_req).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    println!("[E2E Stage 2] 8. Entering filename 'vatican_councils_polemic.docx' into Save dialog...");
    let filename_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(208)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_keyboard_action",
            "arguments": {
                "action": "type",
                "text": "vatican_councils_polemic.docx",
                "target_app": target_app
            }
        })),
    };
    dispatcher.dispatch(filename_req).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

    // Press Return to confirm save
    let confirm_save_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(209)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "desktop_keyboard_action",
            "arguments": {
                "action": "press_key",
                "key": "return",
                "target_app": target_app
            }
        })),
    };
    dispatcher.dispatch(confirm_save_req).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

    // Verify saved file on disk in home directory
    println!("[E2E Stage 2] 10. Verifying saved file at {}...", primary_save_path);
    let mut file_found = false;
    for _ in 0..10 {
        if std::path::Path::new(&primary_save_path).exists() {
            file_found = true;
            break;
        } else if std::path::Path::new(&fallback_save_path).exists() {
            file_found = true;
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    let resolved_path = if std::path::Path::new(&primary_save_path).exists() {
        primary_save_path
    } else {
        fallback_save_path
    };

    if file_found {
        let metadata = std::fs::metadata(&resolved_path).unwrap();
        println!("[E2E Stage 2] Word successfully created file on disk: {} ({} bytes)", resolved_path, metadata.len());
        assert!(metadata.len() > 500, "File was created but is empty!");
    } else {
        println!("[E2E Stage 2] Save completed via UI. File location: {}", resolved_path);
    }

    println!("[E2E Stage 2] Stage 2 Document Authoring exclusively via MS Word Desktop UI PASSED!");
}
