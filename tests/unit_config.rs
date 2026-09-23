use desktop_mcp::config::{Config, SecurityMode, TransportMode};

#[test]
fn test_default_config() {
    let toml_str = r#"
        [server]
        transport = "stdio"

        [security]
        mode = "allowlist"
        allowed_apps = ["com.apple.calculator", "com.microsoft.Word"]
        blocked_apps = ["com.apple.Terminal"]

        [privacy]
        isolate_window_capture = true
    "#;

    let config = Config::from_toml(toml_str).expect("Failed to parse config");
    assert_eq!(config.server.transport, TransportMode::Stdio);
    assert_eq!(config.security.mode, SecurityMode::Allowlist);
    assert!(config.privacy.isolate_window_capture);
    assert!(config.security.is_allowed("com.apple.calculator"));
    assert!(config.security.is_allowed("com.microsoft.Word"));
    assert!(!config.security.is_allowed("com.apple.Terminal"));
    assert!(!config.security.is_allowed("com.random.unlisted"));
}

#[test]
fn test_network_config() {
    let toml_str = r#"
        [server]
        transport = "network"

        [server.network]
        interface = "192.168.1.100"
        port = 9000
        auth_token = "secret-token-123"

        [security]
        mode = "blocklist"
        blocked_apps = ["com.apple.Terminal", "com.1password.1password"]

        [privacy]
        isolate_window_capture = false
    "#;

    let config = Config::from_toml(toml_str).expect("Failed to parse network config");
    assert_eq!(config.server.transport, TransportMode::Network);
    let net = config.server.network.expect("Network config missing");
    assert_eq!(net.interface, "192.168.1.100");
    assert_eq!(net.port, 9000);
    assert_eq!(net.auth_token, "secret-token-123");
    assert_eq!(config.security.mode, SecurityMode::Blocklist);
    assert!(!config.security.is_allowed("com.apple.Terminal"));
    assert!(config.security.is_allowed("com.apple.calculator"));
}
