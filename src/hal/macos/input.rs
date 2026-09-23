use std::process::Command;
use crate::hal::driver::{DriverError, KeyAction, MouseAction, MouseButton};
use core_graphics::event::*;
use core_graphics::event_source::*;
use core_graphics::geometry::CGPoint;
use foreign_types::ForeignType;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventKeyboardSetUnicodeString(
        event: *mut std::ffi::c_void,
        string_length: libc::c_ulong,
        unicode_string: *const u16,
    );
    fn CGEventCreateScrollWheelEvent2(
        source: *mut std::ffi::c_void,
        units: u32,
        wheelCount: u32,
        wheel1: i32,
        wheel2: i32,
        wheel3: i32,
    ) -> *mut std::ffi::c_void;
}

pub fn execute_mouse_action(
    action: MouseAction,
    x: f64,
    y: f64,
    _click_count: u32,
    button: MouseButton,
    scroll_delta: Option<(i32, i32)>,
    target_app: Option<&str>,
    window_id: Option<u64>,
    coordinate_space: Option<&str>,
) -> Result<(), DriverError> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| DriverError::InputFailed("Failed to create CGEventSource".to_string()))?;

    // Determine target window bounds & PID
    let (abs_x, abs_y, _target_pid) = if let Ok(win) = crate::hal::macos::app_manager::find_window_for_app(target_app, window_id) {
        crate::hal::macos::app_manager::activate_pid(win.pid);
        let is_window_relative = coordinate_space != Some("screen_absolute");
        let (final_x, final_y) = if is_window_relative {
            (win.bounds.x + x, win.bounds.y + y)
        } else {
            (x, y)
        };
        (final_x, final_y, Some(win.pid))
    } else {
        (x, y, None)
    };

    let point = CGPoint::new(abs_x, abs_y);

    // Save user's current physical mouse cursor position
    let initial_loc = CGEvent::new(source.clone()).ok().map(|e| e.location());

    // Helper to dispatch event: if targeted to a background PID, post to PID; otherwise post to Quartz HID
    let post_event = |event: &CGEvent| {
        event.post(CGEventTapLocation::HID);
    };

    // 1. Move cursor to target point first so WindowServer updates hit-testing
    let move_event = CGEvent::new_mouse_event(
        source.clone(),
        CGEventType::MouseMoved,
        point,
        CGMouseButton::Left,
    ).map_err(|_| DriverError::InputFailed("Failed to create MouseMoved event".to_string()))?;
    post_event(&move_event);
    std::thread::sleep(std::time::Duration::from_millis(30));

    match action {
        MouseAction::Move => {
            // Already moved above
        }
        MouseAction::Click => {
            let (down_type, up_type, cg_button) = match button {
                MouseButton::Right => (CGEventType::RightMouseDown, CGEventType::RightMouseUp, CGMouseButton::Right),
                _ => (CGEventType::LeftMouseDown, CGEventType::LeftMouseUp, CGMouseButton::Left),
            };

            const K_CG_MOUSE_EVENT_CLICK_STATE: u32 = 1;
            let down_event = CGEvent::new_mouse_event(source.clone(), down_type, point, cg_button)
                .map_err(|_| DriverError::InputFailed("Failed to create mouse down event".to_string()))?;
            down_event.set_integer_value_field(K_CG_MOUSE_EVENT_CLICK_STATE, 1);
            post_event(&down_event);

            std::thread::sleep(std::time::Duration::from_millis(40));

            let up_event = CGEvent::new_mouse_event(source.clone(), up_type, point, cg_button)
                .map_err(|_| DriverError::InputFailed("Failed to create mouse up event".to_string()))?;
            up_event.set_integer_value_field(K_CG_MOUSE_EVENT_CLICK_STATE, 1);
            post_event(&up_event);
        }
        MouseAction::DoubleClick => {
            let (down_type, up_type, cg_button) = match button {
                MouseButton::Right => (CGEventType::RightMouseDown, CGEventType::RightMouseUp, CGMouseButton::Right),
                _ => (CGEventType::LeftMouseDown, CGEventType::LeftMouseUp, CGMouseButton::Left),
            };

            const K_CG_MOUSE_EVENT_CLICK_STATE: u32 = 1;
            // First click
            let down1 = CGEvent::new_mouse_event(source.clone(), down_type, point, cg_button)
                .map_err(|_| DriverError::InputFailed("Failed to create mouse down event".to_string()))?;
            down1.set_integer_value_field(K_CG_MOUSE_EVENT_CLICK_STATE, 1);
            post_event(&down1);
            std::thread::sleep(std::time::Duration::from_millis(30));

            let up1 = CGEvent::new_mouse_event(source.clone(), up_type, point, cg_button)
                .map_err(|_| DriverError::InputFailed("Failed to create mouse up event".to_string()))?;
            up1.set_integer_value_field(K_CG_MOUSE_EVENT_CLICK_STATE, 1);
            post_event(&up1);
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Second click
            let down2 = CGEvent::new_mouse_event(source.clone(), down_type, point, cg_button)
                .map_err(|_| DriverError::InputFailed("Failed to create mouse down event".to_string()))?;
            down2.set_integer_value_field(K_CG_MOUSE_EVENT_CLICK_STATE, 2);
            post_event(&down2);
            std::thread::sleep(std::time::Duration::from_millis(30));

            let up2 = CGEvent::new_mouse_event(source.clone(), up_type, point, cg_button)
                .map_err(|_| DriverError::InputFailed("Failed to create mouse up event".to_string()))?;
            up2.set_integer_value_field(K_CG_MOUSE_EVENT_CLICK_STATE, 2);
            post_event(&up2);
        }
        MouseAction::TripleClick => {
            let (down_type, up_type, cg_button) = match button {
                MouseButton::Right => (CGEventType::RightMouseDown, CGEventType::RightMouseUp, CGMouseButton::Right),
                _ => (CGEventType::LeftMouseDown, CGEventType::LeftMouseUp, CGMouseButton::Left),
            };

            const K_CG_MOUSE_EVENT_CLICK_STATE: u32 = 1;
            for click_num in 1..=3 {
                let down = CGEvent::new_mouse_event(source.clone(), down_type, point, cg_button)
                    .map_err(|_| DriverError::InputFailed("Failed to create mouse down event".to_string()))?;
                down.set_integer_value_field(K_CG_MOUSE_EVENT_CLICK_STATE, click_num);
                post_event(&down);
                std::thread::sleep(std::time::Duration::from_millis(25));

                let up = CGEvent::new_mouse_event(source.clone(), up_type, point, cg_button)
                    .map_err(|_| DriverError::InputFailed("Failed to create mouse up event".to_string()))?;
                up.set_integer_value_field(K_CG_MOUSE_EVENT_CLICK_STATE, click_num);
                post_event(&up);
                std::thread::sleep(std::time::Duration::from_millis(35));
            }
        }
        MouseAction::RightClick => {
            let down_event = CGEvent::new_mouse_event(source.clone(), CGEventType::RightMouseDown, point, CGMouseButton::Right)
                .map_err(|_| DriverError::InputFailed("Failed to create right mouse down".to_string()))?;
            post_event(&down_event);

            std::thread::sleep(std::time::Duration::from_millis(40));

            let up_event = CGEvent::new_mouse_event(source.clone(), CGEventType::RightMouseUp, point, CGMouseButton::Right)
                .map_err(|_| DriverError::InputFailed("Failed to create right mouse up".to_string()))?;
            post_event(&up_event);
        }
        MouseAction::Drag => {
            let drag_event = CGEvent::new_mouse_event(source.clone(), CGEventType::LeftMouseDragged, point, CGMouseButton::Left)
                .map_err(|_| DriverError::InputFailed("Failed to create mouse drag event".to_string()))?;
            post_event(&drag_event);
        }
        MouseAction::Scroll => {
            if let Some((dx, dy)) = scroll_delta {
                unsafe {
                    let scroll_ref = CGEventCreateScrollWheelEvent2(
                        std::ptr::null_mut(),
                        0, // kCGScrollEventUnitPixel
                        2,
                        dy,
                        dx,
                        0,
                    );
                    if !scroll_ref.is_null() {
                        let scroll_event = CGEvent::from_ptr(scroll_ref as *mut _);
                        post_event(&scroll_event);
                    }
                }
            }
        }
    }

    // Allow application 80ms to process event before resetting cursor
    std::thread::sleep(std::time::Duration::from_millis(80));

    // Restore user's original cursor position
    if let Some(orig_pos) = initial_loc {
        if let Ok(restore_ev) = CGEvent::new_mouse_event(source, CGEventType::MouseMoved, orig_pos, CGMouseButton::Left) {
            restore_ev.post(CGEventTapLocation::HID);
        }
    }

    Ok(())
}

pub fn execute_keyboard_action(
    action: KeyAction,
    text: Option<&str>,
    key: Option<&str>,
    modifiers: &[String],
    target_app: Option<&str>,
    window_id: Option<u64>,
) -> Result<(), DriverError> {
    let target_pid = if let Ok(win) = crate::hal::macos::app_manager::find_window_for_app(target_app, window_id) {
        crate::hal::macos::app_manager::activate_pid(win.pid);
        Some(win.pid)
    } else {
        None
    };

    match action {
        KeyAction::Type => {
            if let Some(t) = text {
                type_text_smart(t, target_pid)?;
            }
        }
        KeyAction::Hotkey | KeyAction::PressKey => {
            if let Some(k) = key {
                press_key_quartz(k, modifiers, target_pid)?;
            }
        }
        KeyAction::KeyDown => {
            if let Some(k) = key {
                send_single_key(k, modifiers, true, target_pid)?;
            }
        }
        KeyAction::KeyUp => {
            if let Some(k) = key {
                send_single_key(k, modifiers, false, target_pid)?;
            }
        }
    }
    Ok(())
}

fn type_text_smart(text: &str, target_pid: Option<i32>) -> Result<(), DriverError> {
    if text.len() > 30 || text.contains('\n') {
        paste_text(text, target_pid)
    } else {
        type_text_quartz(text, target_pid)
    }
}

fn paste_text(text: &str, target_pid: Option<i32>) -> Result<(), DriverError> {
    use std::io::Write;
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| DriverError::InputFailed(format!("Failed to spawn pbcopy: {}", e)))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())
            .map_err(|e| DriverError::InputFailed(format!("Failed to write to pbcopy: {}", e)))?;
    }
    child.wait()
        .map_err(|e| DriverError::InputFailed(format!("pbcopy failed: {}", e)))?;

    std::thread::sleep(std::time::Duration::from_millis(50));
    press_key_quartz("v", &["cmd".to_string()], target_pid)?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    Ok(())
}

fn type_text_quartz(text: &str, target_pid: Option<i32>) -> Result<(), DriverError> {
    for ch in text.chars() {
        if let Some((keycode, shift)) = map_char_to_keycode(ch) {
            let mut mods = Vec::new();
            if shift {
                mods.push("shift".to_string());
            }
            send_keycode(keycode, &mods, target_pid)?;
        } else {
            send_unicode_char(ch, target_pid)?;
        }
        std::thread::sleep(std::time::Duration::from_millis(35));
    }
    Ok(())
}

fn send_unicode_char(ch: char, _target_pid: Option<i32>) -> Result<(), DriverError> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| DriverError::InputFailed("Failed to create CGEventSource".to_string()))?;

    let utf16_chars: Vec<u16> = ch.encode_utf16(&mut [0; 2]).to_vec();

    let down_event = CGEvent::new_keyboard_event(source.clone(), 0, true)
        .map_err(|_| DriverError::InputFailed("Failed to create down event".to_string()))?;

    unsafe {
        CGEventKeyboardSetUnicodeString(
            down_event.as_ptr() as *mut std::ffi::c_void,
            utf16_chars.len() as libc::c_ulong,
            utf16_chars.as_ptr(),
        );
    }
    down_event.post(CGEventTapLocation::HID);

    std::thread::sleep(std::time::Duration::from_millis(25));

    let up_event = CGEvent::new_keyboard_event(source, 0, false)
        .map_err(|_| DriverError::InputFailed("Failed to create up event".to_string()))?;
    unsafe {
        CGEventKeyboardSetUnicodeString(
            up_event.as_ptr() as *mut std::ffi::c_void,
            utf16_chars.len() as libc::c_ulong,
            utf16_chars.as_ptr(),
        );
    }
    up_event.post(CGEventTapLocation::HID);

    Ok(())
}

fn press_key_quartz(key: &str, modifiers: &[String], target_pid: Option<i32>) -> Result<(), DriverError> {
    let (kc, is_unicode) = resolve_key(key)?;
    if is_unicode {
        let ch = key.chars().next().unwrap();
        send_unicode_char(ch, target_pid)
    } else {
        send_keycode(kc, modifiers, target_pid)
    }
}

fn send_single_key(key: &str, modifiers: &[String], is_down: bool, _target_pid: Option<i32>) -> Result<(), DriverError> {
    let (keycode, _) = resolve_key(key)?;
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| DriverError::InputFailed("Failed to create CGEventSource".to_string()))?;

    let event = CGEvent::new_keyboard_event(source, keycode, is_down)
        .map_err(|_| DriverError::InputFailed("Failed to create single key event".to_string()))?;

    let mut flags: CGEventFlags = CGEventFlags::CGEventFlagNull;
    for m in modifiers {
        match m.to_lowercase().as_str() {
            "cmd" | "command" | "super" => flags |= CGEventFlags::CGEventFlagCommand,
            "ctrl" | "control" => flags |= CGEventFlags::CGEventFlagControl,
            "alt" | "option" => flags |= CGEventFlags::CGEventFlagAlternate,
            "shift" => flags |= CGEventFlags::CGEventFlagShift,
            _ => {}
        }
    }

    if flags != CGEventFlags::CGEventFlagNull {
        event.set_flags(flags);
    }

    event.post(CGEventTapLocation::HID);
    Ok(())
}

fn send_keycode(keycode: u16, modifiers: &[String], _target_pid: Option<i32>) -> Result<(), DriverError> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| DriverError::InputFailed("Failed to create CGEventSource".to_string()))?;

    let down_event = CGEvent::new_keyboard_event(source.clone(), keycode, true)
        .map_err(|_| DriverError::InputFailed("Failed to create keyboard down event".to_string()))?;

    let mut flags: CGEventFlags = CGEventFlags::CGEventFlagNull;
    for m in modifiers {
        match m.to_lowercase().as_str() {
            "cmd" | "command" | "super" => flags |= CGEventFlags::CGEventFlagCommand,
            "ctrl" | "control" => flags |= CGEventFlags::CGEventFlagControl,
            "alt" | "option" => flags |= CGEventFlags::CGEventFlagAlternate,
            "shift" => flags |= CGEventFlags::CGEventFlagShift,
            _ => {}
        }
    }

    if flags != CGEventFlags::CGEventFlagNull {
        down_event.set_flags(flags);
    }

    down_event.post(CGEventTapLocation::HID);

    std::thread::sleep(std::time::Duration::from_millis(25));

    let up_event = CGEvent::new_keyboard_event(source, keycode, false)
        .map_err(|_| DriverError::InputFailed("Failed to create keyboard up event".to_string()))?;
    if flags != CGEventFlags::CGEventFlagNull {
        up_event.set_flags(flags);
    }
    up_event.post(CGEventTapLocation::HID);

    Ok(())
}

fn resolve_key(key: &str) -> Result<(u16, bool), DriverError> {
    let kc = match key.to_lowercase().as_str() {
        "return" | "enter" => (36, false),
        "tab" => (48, false),
        "space" => (49, false),
        "delete" | "backspace" => (51, false),
        "escape" | "esc" => (53, false),
        "left" => (123, false),
        "right" => (124, false),
        "down" => (125, false),
        "up" => (126, false),
        "a" => (0, false),
        "s" => (1, false),
        "d" => (2, false),
        "f" => (3, false),
        "h" => (4, false),
        "g" => (5, false),
        "z" => (6, false),
        "x" => (7, false),
        "c" => (8, false),
        "v" => (9, false),
        "b" => (11, false),
        "q" => (12, false),
        "w" => (13, false),
        "e" => (14, false),
        "r" => (15, false),
        "y" => (16, false),
        "t" => (17, false),
        "1" => (18, false),
        "2" => (19, false),
        "3" => (20, false),
        "4" => (21, false),
        "6" => (22, false),
        "5" => (23, false),
        "=" => (24, false),
        "9" => (25, false),
        "7" => (26, false),
        "-" => (27, false),
        "8" => (28, false),
        "0" => (29, false),
        "]" => (30, false),
        "o" => (31, false),
        "u" => (32, false),
        "[" => (33, false),
        "i" => (34, false),
        "p" => (35, false),
        "l" => (37, false),
        "j" => (38, false),
        "'" => (39, false),
        "k" => (40, false),
        ";" => (41, false),
        "\\" => (42, false),
        "," => (43, false),
        "/" => (44, false),
        "n" => (45, false),
        "m" => (46, false),
        "." => (47, false),
        "`" => (50, false),
        other => {
            if let Some(ch) = other.chars().next() {
                if let Some((kc, _)) = map_char_to_keycode(ch) {
                    (kc, false)
                } else {
                    (0, true)
                }
            } else {
                return Err(DriverError::InputFailed(format!("Unknown key '{}'", key)));
            }
        }
    };
    Ok(kc)
}

fn map_char_to_keycode(ch: char) -> Option<(u16, bool)> {
    match ch {
        '0' => Some((29, false)),
        '1' => Some((18, false)),
        '2' => Some((19, false)),
        '3' => Some((20, false)),
        '4' => Some((21, false)),
        '5' => Some((23, false)),
        '6' => Some((22, false)),
        '7' => Some((26, false)),
        '8' => Some((28, false)),
        '9' => Some((25, false)),
        '+' => Some((24, true)),  // '=' with shift is '+'
        '=' => Some((24, false)),
        '-' => Some((27, false)),
        '*' => Some((28, true)),  // '8' with shift is '*'
        '/' => Some((44, false)),
        '.' => Some((47, false)),
        '(' => Some((25, true)),  // '9' with shift
        ')' => Some((29, true)),  // '0' with shift
        ' ' => Some((49, false)),
        '\n' | '\r' => Some((36, false)),
        _ => None,
    }
}
