use std::process::Command;
use crate::hal::driver::{DriverError, KeyAction, MouseAction, MouseButton};
use core_graphics::event::*;
use core_graphics::event_source::*;
use core_graphics::geometry::CGPoint;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventKeyboardSetUnicodeString(
        event: *mut std::ffi::c_void,
        string_length: libc::c_ulong,
        unicode_string: *const u16,
    );
}

pub fn execute_mouse_action(
    action: MouseAction,
    x: f64,
    y: f64,
    click_count: u32,
    button: MouseButton,
    scroll_delta: Option<(i32, i32)>,
) -> Result<(), DriverError> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| DriverError::InputFailed("Failed to create CGEventSource".to_string()))?;

    let point = CGPoint::new(x, y);

    match action {
        MouseAction::Move => {
            let event = CGEvent::new_mouse_event(
                source,
                CGEventType::MouseMoved,
                point,
                CGMouseButton::Left,
            ).map_err(|_| DriverError::InputFailed("Failed to create MouseMoved event".to_string()))?;
            event.post(CGEventTapLocation::HID);
        }
        MouseAction::Click | MouseAction::DoubleClick | MouseAction::TripleClick => {
            let (down_type, up_type, cg_button) = match button {
                MouseButton::Right => (CGEventType::RightMouseDown, CGEventType::RightMouseUp, CGMouseButton::Right),
                _ => (CGEventType::LeftMouseDown, CGEventType::LeftMouseUp, CGMouseButton::Left),
            };

            let clicks = match action {
                MouseAction::DoubleClick => 2,
                MouseAction::TripleClick => 3,
                _ => click_count.max(1),
            };

            const K_CG_MOUSE_EVENT_CLICK_STATE: u32 = 1;

            // Mouse down
            let down_event = CGEvent::new_mouse_event(source.clone(), down_type, point, cg_button)
                .map_err(|_| DriverError::InputFailed("Failed to create mouse down event".to_string()))?;
            down_event.set_integer_value_field(K_CG_MOUSE_EVENT_CLICK_STATE, clicks as i64);
            down_event.post(CGEventTapLocation::HID);

            std::thread::sleep(std::time::Duration::from_millis(50));

            // Mouse up
            let up_event = CGEvent::new_mouse_event(source, up_type, point, cg_button)
                .map_err(|_| DriverError::InputFailed("Failed to create mouse up event".to_string()))?;
            up_event.set_integer_value_field(K_CG_MOUSE_EVENT_CLICK_STATE, clicks as i64);
            up_event.post(CGEventTapLocation::HID);
        }
        MouseAction::RightClick => {
            let down_event = CGEvent::new_mouse_event(source.clone(), CGEventType::RightMouseDown, point, CGMouseButton::Right)
                .map_err(|_| DriverError::InputFailed("Failed to create right mouse down".to_string()))?;
            down_event.post(CGEventTapLocation::HID);

            std::thread::sleep(std::time::Duration::from_millis(50));

            let up_event = CGEvent::new_mouse_event(source, CGEventType::RightMouseUp, point, CGMouseButton::Right)
                .map_err(|_| DriverError::InputFailed("Failed to create right mouse up".to_string()))?;
            up_event.post(CGEventTapLocation::HID);
        }
        MouseAction::Drag => {
            let drag_event = CGEvent::new_mouse_event(source, CGEventType::LeftMouseDragged, point, CGMouseButton::Left)
                .map_err(|_| DriverError::InputFailed("Failed to create mouse drag event".to_string()))?;
            drag_event.post(CGEventTapLocation::HID);
        }
        MouseAction::Scroll => {
            if let Some((_dx, dy)) = scroll_delta {
                let script = format!(
                    "tell application \"System Events\" to tell window 1 of (first process whose frontmost is true) to scroll up by {}",
                    -dy
                );
                let _ = Command::new("osascript").args(["-e", &script]).status();
            }
        }
    }

    Ok(())
}

pub fn execute_keyboard_action(
    action: KeyAction,
    text: Option<&str>,
    key: Option<&str>,
    modifiers: &[String],
) -> Result<(), DriverError> {
    match action {
        KeyAction::Type => {
            if let Some(t) = text {
                type_text_quartz(t)?;
            }
        }
        KeyAction::Hotkey | KeyAction::PressKey => {
            if let Some(k) = key {
                press_key_quartz(k, modifiers)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn type_text_quartz(text: &str) -> Result<(), DriverError> {
    for ch in text.chars() {
        if let Some((keycode, shift)) = map_char_to_keycode(ch) {
            let mut mods = Vec::new();
            if shift {
                mods.push("shift".to_string());
            }
            send_keycode(keycode, &mods)?;
        } else {
            // Send arbitrary Unicode character using CGEventKeyboardSetUnicodeString
            send_unicode_char(ch)?;
        }
        std::thread::sleep(std::time::Duration::from_millis(40));
    }
    Ok(())
}

fn send_unicode_char(ch: char) -> Result<(), DriverError> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| DriverError::InputFailed("Failed to create CGEventSource".to_string()))?;

    let utf16_chars: Vec<u16> = ch.encode_utf16(&mut [0; 2]).to_vec();

    let down_event = CGEvent::new_keyboard_event(source.clone(), 0, true)
        .map_err(|_| DriverError::InputFailed("Failed to create down event".to_string()))?;

    unsafe {
        use foreign_types::ForeignType;
        CGEventKeyboardSetUnicodeString(
            down_event.as_ptr() as *mut std::ffi::c_void,
            utf16_chars.len() as libc::c_ulong,
            utf16_chars.as_ptr(),
        );
    }
    down_event.post(CGEventTapLocation::HID);

    std::thread::sleep(std::time::Duration::from_millis(20));

    let up_event = CGEvent::new_keyboard_event(source, 0, false)
        .map_err(|_| DriverError::InputFailed("Failed to create up event".to_string()))?;
    unsafe {
        use foreign_types::ForeignType;
        CGEventKeyboardSetUnicodeString(
            up_event.as_ptr() as *mut std::ffi::c_void,
            utf16_chars.len() as libc::c_ulong,
            utf16_chars.as_ptr(),
        );
    }
    up_event.post(CGEventTapLocation::HID);

    Ok(())
}

fn press_key_quartz(key: &str, modifiers: &[String]) -> Result<(), DriverError> {
    let keycode = match key.to_lowercase().as_str() {
        "return" | "enter" => 36,
        "tab" => 48,
        "space" => 49,
        "delete" | "backspace" => 51,
        "escape" | "esc" => 53,
        "left" => 123,
        "right" => 124,
        "down" => 125,
        "up" => 126,
        "c" => 8,
        "v" => 9,
        "s" => 1,
        "b" => 11,
        "i" => 34,
        "e" => 14,
        "a" => 0,
        other => {
            if let Some(ch) = other.chars().next() {
                if let Some((kc, _)) = map_char_to_keycode(ch) {
                    kc
                } else {
                    return send_unicode_char(ch);
                }
            } else {
                return Err(DriverError::InputFailed(format!("Unknown key '{}'", key)));
            }
        }
    };

    send_keycode(keycode, modifiers)
}

fn send_keycode(keycode: u16, modifiers: &[String]) -> Result<(), DriverError> {
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
