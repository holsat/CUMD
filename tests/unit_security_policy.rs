use desktop_mcp::security::policy::{PolicyEngine, SecurityError};
use desktop_mcp::config::{SecurityConfig, SecurityMode};

#[test]
fn test_allowlist_enforcement() {
    let config = SecurityConfig {
        mode: SecurityMode::Allowlist,
        enforce_focus_lock: true,
        allowed_apps: vec![
            "com.apple.calculator".to_string(),
            "com.microsoft.Word".to_string(),
            "org.libreoffice.LibreOffice".to_string(),
        ],
        blocked_apps: vec![
            "com.apple.Terminal".to_string(),
        ],
    };

    let engine = PolicyEngine::new(config);

    assert!(engine.validate_app("com.apple.calculator").is_ok());
    assert!(engine.validate_app("com.microsoft.Word").is_ok());

    // Explicitly blocked
    let err = engine.validate_app("com.apple.Terminal").unwrap_err();
    assert!(matches!(err, SecurityError::BlockedApplication(_)));

    // Not in allowlist
    let err = engine.validate_app("com.google.Chrome").unwrap_err();
    assert!(matches!(err, SecurityError::ApplicationNotAllowed(_)));
}

#[test]
fn test_blocklist_enforcement() {
    let config = SecurityConfig {
        mode: SecurityMode::Blocklist,
        enforce_focus_lock: true,
        allowed_apps: vec![],
        blocked_apps: vec![
            "com.apple.Terminal".to_string(),
            "com.1password.1password".to_string(),
        ],
    };

    let engine = PolicyEngine::new(config);

    // Unlisted is allowed in blocklist mode
    assert!(engine.validate_app("com.apple.calculator").is_ok());
    assert!(engine.validate_app("com.microsoft.Word").is_ok());

    // Blocked is denied
    assert!(matches!(
        engine.validate_app("com.apple.Terminal").unwrap_err(),
        SecurityError::BlockedApplication(_)
    ));
}

#[test]
fn test_coordinate_bounds_gating() {
    let config = SecurityConfig::default();
    let engine = PolicyEngine::new(config);

    // Window bounds: origin (100, 100), size (400, 600)
    let window_rect = desktop_mcp::utils::coordinates::Rect {
        x: 100.0,
        y: 100.0,
        width: 400.0,
        height: 600.0,
    };

    // Point inside window
    assert!(engine.validate_point_in_window(150.0, 200.0, &window_rect).is_ok());

    // Point outside window (e.g. clicking on taskbar or another window)
    let err = engine.validate_point_in_window(50.0, 50.0, &window_rect).unwrap_err();
    assert!(matches!(err, SecurityError::CoordinatesOutOfBounds { .. }));
}
