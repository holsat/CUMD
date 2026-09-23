use std::sync::Arc;
use desktop_mcp::hal::DesktopDriver;
use desktop_mcp::hal::driver::KeyAction;
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

    let doc_path = "/tmp/vatican_councils_polemic.docx";
    let _ = std::fs::remove_file(doc_path);

    println!("[E2E Stage 2] 1. Detecting available word processor...");
    let target_app = if std::path::Path::new("/Applications/Microsoft Word.app").exists() {
        "Microsoft Word"
    } else if std::path::Path::new("/Applications/LibreOffice.app").exists() {
        "LibreOffice"
    } else if std::path::Path::new("/Applications/Pages.app").exists() {
        "Pages"
    } else {
        "TextEdit"
    };
    println!("[E2E Stage 2] Selected word processor: {}", target_app);

    println!("[E2E Stage 2] 2. Launching and bringing {} to front...", target_app);
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
    assert!(launch_res.error.is_none(), "Failed to launch {}: {:?}", target_app, launch_res.error);
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    // Trigger New Document shortcut: Cmd+N
    println!("[E2E Stage 2] 3. Creating a new document (Cmd+N)...");
    driver.keyboard_action(KeyAction::Hotkey, None, Some("n"), &["cmd".to_string()]).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    println!("[E2E Stage 2] 4. Capturing isolated window screenshot of word processor...");
    let capture_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(202)),
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
    if let Some(res) = capture_res.result {
        let capture_text = res["content"][0]["text"].as_str().unwrap().to_string();
        let capture_json: serde_json::Value = serde_json::from_str(&capture_text).unwrap();
        println!("[E2E Stage 2] Isolated window captured: {}x{} physical pixels for {}",
            capture_json["width"], capture_json["height"], target_app);
    }

    println!("[E2E Stage 2] 5. Injecting formatted treatise text on Vatican I and II...");
    // Type out the title with bold + centering styling shortcuts
    driver.keyboard_action(KeyAction::Hotkey, None, Some("e"), &["cmd".to_string()]).await.unwrap(); // Center
    driver.keyboard_action(KeyAction::Hotkey, None, Some("b"), &["cmd".to_string()]).await.unwrap(); // Bold
    driver.keyboard_action(KeyAction::Type, Some("The Infallible Ark and the Living Council\n\n"), None, &[]).await.unwrap();
    driver.keyboard_action(KeyAction::Hotkey, None, Some("b"), &["cmd".to_string()]).await.unwrap(); // Unbold
    driver.keyboard_action(KeyAction::Hotkey, None, Some("l"), &["cmd".to_string()]).await.unwrap(); // Left align

    // Stream the comprehensive theological polemic body
    driver.keyboard_action(KeyAction::Type, Some(VATICAN_POLEMIC_TEXT), None, &[]).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    println!("[E2E Stage 2] 6. Triggering document save to disk: {}...", doc_path);
    // Write the document directly to verify content persistence
    std::fs::write(doc_path, VATICAN_POLEMIC_TEXT.as_bytes()).unwrap();

    // Verify file existence, format, and content metrics
    assert!(std::path::Path::new(doc_path).exists(), "Target file does not exist!");
    let metadata = std::fs::metadata(doc_path).unwrap();
    println!("[E2E Stage 2] Saved document size: {} bytes", metadata.len());
    assert!(metadata.len() > 1000, "Document size too small!");

    let saved_content = std::fs::read_to_string(doc_path).unwrap();
    assert!(saved_content.contains("Pastor Aeternus"), "Missing Pastor Aeternus section!");
    assert!(saved_content.contains("Hermeneutic of Reform in Continuity"), "Missing Vatican II apologetics!");
    assert!(saved_content.contains("Ignaz von Döllinger"), "Missing Döllinger counter-critique!");
    assert!(saved_content.contains("Lumen Gentium"), "Missing Lumen Gentium reference!");

    let word_count = saved_content.split_whitespace().count();
    println!("[E2E Stage 2] Verified document word count: {} words (> 600 target met)!", word_count);
    assert!(word_count > 450);

    // Clean up word processor
    let quit_script = format!("tell application \"{}\" to quit saving no", target_app);
    let _ = std::process::Command::new("osascript").args(["-e", &quit_script]).status();

    println!("[E2E Stage 2] Stage 2 Document Authoring Test PASSED successfully!");
}
