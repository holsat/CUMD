use std::process::Command;
use crate::hal::driver::{AccessibilityNode, DriverError};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

pub fn is_accessibility_granted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub fn inspect_app_ui(app_name: Option<&str>, _max_depth: u32) -> Result<AccessibilityNode, DriverError> {
    let target = app_name.unwrap_or("System Events");

    let script = format!(
        r#"
        tell application "System Events"
            set p to first process whose name is "{0}" or name contains "{0}"
            set win to first window of p
            set winTitle to name of win
            set elemList to {{}}
            try
                set uiElems to entire contents of win
                repeat with e in uiElems
                    try
                        set eRole to role of e
                        set eTitle to name of e
                        set eVal to value of e
                        set end of elemList to (eRole & "::" & eTitle & "::" & eVal)
                    end try
                end repeat
            end try
            return winTitle & "|||" & (elemList as string)
        end tell
        "#,
        target
    );

    let output = Command::new("osascript").args(["-e", &script]).output();

    match output {
        Ok(out) if out.status.success() => {
            let res = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = res.split("|||").collect();
            let title = parts.get(0).unwrap_or(&"").trim().to_string();

            let mut children = Vec::new();
            if let Some(list_str) = parts.get(1) {
                for item in list_str.split(',') {
                    let subparts: Vec<&str> = item.split("::").collect();
                    let role = subparts.get(0).unwrap_or(&"unknown").trim().to_string();
                    let name = subparts.get(1).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
                    let val = subparts.get(2).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

                    if !role.is_empty() {
                        children.push(AccessibilityNode {
                            role,
                            title: name,
                            value: val,
                            bounds: None,
                            children: Vec::new(),
                            actions: Vec::new(),
                        });
                    }
                }
            }

            Ok(AccessibilityNode {
                role: "AXWindow".to_string(),
                title: Some(title),
                value: None,
                bounds: None,
                children,
                actions: Vec::new(),
            })
        }
        _ => {
            // Graceful fallback
            Ok(AccessibilityNode {
                role: "AXApplication".to_string(),
                title: app_name.map(|s| s.to_string()),
                value: None,
                bounds: None,
                children: Vec::new(),
                actions: Vec::new(),
            })
        }
    }
}

pub fn get_calculator_display_value() -> Result<String, DriverError> {
    // Method 1: In macOS Calculator, Cmd+C natively copies the active calculation display
    let _ = crate::hal::macos::input::execute_keyboard_action(
        crate::hal::driver::KeyAction::Hotkey,
        None,
        Some("c"),
        &["cmd".to_string()],
    );
    std::thread::sleep(std::time::Duration::from_millis(200));

    if let Ok(output) = Command::new("pbpaste").output() {
        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !val.is_empty() {
                return Ok(val);
            }
        }
    }

    // Method 2: Fallback to Accessibility query
    let script = r#"
        tell application "System Events"
            tell process "Calculator"
                try
                    set allTexts to (value of every static text of entire contents of window 1)
                    repeat with v in allTexts
                        if v is not missing value and v is not "" then
                            return v as string
                        end if
                    end repeat
                end try
            end tell
        end tell
        return ""
    "#;

    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| DriverError::AccessibilityError(format!("Failed to execute osascript: {}", e)))?;

    if output.status.success() {
        let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !val.is_empty() {
            return Ok(val);
        }
    }

    Err(DriverError::AccessibilityError("Could not retrieve Calculator display value".to_string()))
}
