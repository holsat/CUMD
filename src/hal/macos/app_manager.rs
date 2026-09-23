use std::process::Command;
use crate::hal::driver::{DriverError, WindowInfo};
use crate::utils::coordinates::Rect;
use core_foundation::base::TCFType;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_foundation_sys::dictionary::CFDictionaryGetValue;
use core_graphics::display::*;
use std::ffi::c_void;

pub fn launch_or_activate_app(app_identifier: &str) -> Result<(), DriverError> {
    // Attempt launching with open -a or -b
    let status = if app_identifier.contains('.') && !app_identifier.ends_with(".app") {
        Command::new("open")
            .args(["-b", app_identifier])
            .status()
    } else {
        Command::new("open")
            .args(["-a", app_identifier])
            .status()
    };

    match status {
        Ok(s) if s.success() => {
            std::thread::sleep(std::time::Duration::from_millis(400));
            Ok(())
        }
        _ => {
            let script = format!("tell application \"{}\" to activate", app_identifier);
            let as_status = Command::new("osascript").args(["-e", &script]).status();
            match as_status {
                Ok(s) if s.success() => {
                    std::thread::sleep(std::time::Duration::from_millis(400));
                    Ok(())
                }
                _ => Err(DriverError::AppNotFound(app_identifier.to_string())),
            }
        }
    }
}

pub fn get_frontmost_app_name() -> Result<String, DriverError> {
    let script = "tell application \"System Events\" to get name of first process whose frontmost is true";
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| DriverError::OsError(format!("Failed to execute osascript: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(DriverError::OsError("Failed to get frontmost application".to_string()))
    }
}

pub fn find_window_for_app(app_name: Option<&str>, window_id: Option<u64>) -> Result<WindowInfo, DriverError> {
    for attempt in 0..10 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        if let Ok(win) = find_window_once(app_name, window_id) {
            return Ok(win);
        }
    }

    Err(DriverError::NoActiveWindow(format!(
        "No window found matching app={:?}, id={:?} after retries",
        app_name, window_id
    )))
}

fn find_window_once(app_name: Option<&str>, window_id: Option<u64>) -> Result<WindowInfo, DriverError> {
    unsafe {
        let options = kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements;
        let window_list = CGWindowListCopyWindowInfo(options, kCGNullWindowID);
        if window_list.is_null() {
            return Err(DriverError::CaptureFailed("Failed to copy window list info".to_string()));
        }

        let cf_array = core_foundation::array::CFArray::<*const c_void>::wrap_under_create_rule(window_list);
        let count = cf_array.len();

        let target_app_lower = app_name.map(|s| s.to_lowercase());

        for i in 0..count {
            let dict_ptr = *cf_array.get(i).unwrap();
            let dict_ref = dict_ptr as core_foundation_sys::dictionary::CFDictionaryRef;

            let wid = get_dict_number(dict_ref, "kCGWindowNumber").unwrap_or(0) as u64;
            let layer = get_dict_number(dict_ref, "kCGWindowLayer").unwrap_or(0);
            let pid = get_dict_number(dict_ref, "kCGWindowOwnerPID").unwrap_or(0) as i32;
            let owner_name = get_dict_string(dict_ref, "kCGWindowOwnerName").unwrap_or_default();
            let window_name = get_dict_string(dict_ref, "kCGWindowName").unwrap_or_default();

            // Layer 0 is standard app windows, allow small variance
            if layer > 5 {
                continue;
            }

            if let Some(target_wid) = window_id {
                if wid != target_wid {
                    continue;
                }
            } else if let Some(ref target) = target_app_lower {
                let owner_lower = owner_name.to_lowercase();
                if !owner_lower.contains(target) && !target.contains(&owner_lower) {
                    continue;
                }
            }

            // Extract bounds dictionary
            let bounds_key = CFString::new("kCGWindowBounds");
            let bounds_ptr = CFDictionaryGetValue(dict_ref, bounds_key.as_concrete_TypeRef() as *const c_void);
            let bounds = if !bounds_ptr.is_null() {
                let b_dict = bounds_ptr as core_foundation_sys::dictionary::CFDictionaryRef;
                let x = get_dict_number(b_dict, "X").unwrap_or(0) as f64;
                let y = get_dict_number(b_dict, "Y").unwrap_or(0) as f64;
                let width = get_dict_number(b_dict, "Width").unwrap_or(0) as f64;
                let height = get_dict_number(b_dict, "Height").unwrap_or(0) as f64;
                Rect { x, y, width, height }
            } else {
                Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 }
            };

            if bounds.width < 50.0 || bounds.height < 50.0 {
                continue;
            }

            return Ok(WindowInfo {
                window_id: wid,
                app_id: owner_name.clone(),
                app_name: owner_name,
                title: window_name,
                bounds,
                is_active: true,
                pid,
            });
        }

        Err(DriverError::NoActiveWindow(format!(
            "No window found matching app={:?}, id={:?}",
            app_name, window_id
        )))
    }
}

unsafe fn get_dict_number(dict: core_foundation_sys::dictionary::CFDictionaryRef, key_name: &str) -> Option<i64> {
    let key = CFString::new(key_name);
    let val_ptr = CFDictionaryGetValue(dict, key.as_concrete_TypeRef() as *const c_void);
    if val_ptr.is_null() {
        return None;
    }
    let num = CFNumber::wrap_under_get_rule(val_ptr as _);
    num.to_i64()
}

unsafe fn get_dict_string(dict: core_foundation_sys::dictionary::CFDictionaryRef, key_name: &str) -> Option<String> {
    let key = CFString::new(key_name);
    let val_ptr = CFDictionaryGetValue(dict, key.as_concrete_TypeRef() as *const c_void);
    if val_ptr.is_null() {
        return None;
    }
    let s = CFString::wrap_under_get_rule(val_ptr as _);
    Some(s.to_string())
}
