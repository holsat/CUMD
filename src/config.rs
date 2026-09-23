use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportMode {
    Stdio,
    UnixSocket,
    Network,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnixSocketConfig {
    #[serde(default = "default_socket_path")]
    pub path: PathBuf,
    #[serde(default = "default_socket_chmod")]
    pub chmod: String,
}

fn default_socket_path() -> PathBuf {
    PathBuf::from("/tmp/desktop-mcp.sock")
}

fn default_socket_chmod() -> String {
    "0600".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_network_interface")]
    pub interface: String,
    #[serde(default = "default_network_port")]
    pub port: u16,
    #[serde(default = "default_network_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub auth_token: String,
    #[serde(default)]
    pub enable_tls: bool,
    #[serde(default)]
    pub cert_file: Option<PathBuf>,
    #[serde(default)]
    pub key_file: Option<PathBuf>,
}

fn default_network_interface() -> String {
    "127.0.0.1".to_string()
}

fn default_network_port() -> u16 {
    8443
}

fn default_network_protocol() -> String {
    "sse".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub transport: TransportMode,
    pub unix_socket: Option<UnixSocketConfig>,
    pub network: Option<NetworkConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SecurityMode {
    #[default]
    Allowlist,
    Blocklist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub mode: SecurityMode,
    #[serde(default = "default_true")]
    pub enforce_focus_lock: bool,
    #[serde(default)]
    pub allowed_apps: Vec<String>,
    #[serde(default)]
    pub blocked_apps: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            mode: SecurityMode::Allowlist,
            enforce_focus_lock: true,
            allowed_apps: vec![
                "com.apple.calculator".to_string(),
                "Calculator".to_string(),
                "com.microsoft.Word".to_string(),
                "Microsoft Word".to_string(),
                "Word".to_string(),
                "org.libreoffice.LibreOffice".to_string(),
                "LibreOffice".to_string(),
                "com.apple.Pages".to_string(),
                "Pages".to_string(),
                "com.apple.TextEdit".to_string(),
                "TextEdit".to_string(),
                "kcalc".to_string(),
                "gnome-calculator".to_string(),
                "org.kde.kcalc".to_string(),
                "org.gnome.Calculator".to_string(),
                "libreoffice-writer".to_string(),
                "onlyoffice-desktopeditors".to_string(),
            ],
            blocked_apps: vec![
                "com.apple.Terminal".to_string(),
                "com.googlecode.iterm2".to_string(),
                "org.gnome.Terminal".to_string(),
                "com.1password.1password".to_string(),
                "org.keepassxc.keepassxc".to_string(),
                "com.apple.keychainaccess".to_string(),
            ],
        }
    }
}

impl SecurityConfig {
    pub fn is_allowed(&self, app_id: &str) -> bool {
        let app_id_lower = app_id.to_lowercase();

        // Blacklist check takes precedence in all modes
        for blocked in &self.blocked_apps {
            if blocked.to_lowercase() == app_id_lower || app_id_lower.contains(&blocked.to_lowercase()) {
                return false;
            }
        }

        match self.mode {
            SecurityMode::Allowlist => {
                for allowed in &self.allowed_apps {
                    if allowed.to_lowercase() == app_id_lower || app_id_lower.contains(&allowed.to_lowercase()) {
                        return true;
                    }
                }
                false
            }
            SecurityMode::Blocklist => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    #[serde(default = "default_true")]
    pub isolate_window_capture: bool,
    #[serde(default = "default_true")]
    pub mask_sensitive_text_fields: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            isolate_window_capture: true,
            mask_sensitive_text_fields: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub privacy: PrivacyConfig,
}

impl Config {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_toml(&content)?)
    }
}
