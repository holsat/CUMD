use thiserror::Error;
use crate::config::SecurityConfig;
use crate::utils::coordinates::Rect;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("SecurityPolicyViolation: Application '{0}' is explicitly blocked")]
    BlockedApplication(String),

    #[error("SecurityPolicyViolation: Application '{0}' is not in the allowed applications list")]
    ApplicationNotAllowed(String),

    #[error("SecurityPolicyViolation: Coordinates ({x}, {y}) fall outside target window bounds [{min_x}, {min_y}, {max_x}, {max_y}]")]
    CoordinatesOutOfBounds {
        x: f64,
        y: f64,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    },
}

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    config: SecurityConfig,
}

impl PolicyEngine {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    pub fn validate_app(&self, app_id: &str) -> Result<(), SecurityError> {
        let app_id_lower = app_id.to_lowercase();

        // 1. Blacklist check
        for blocked in &self.config.blocked_apps {
            if blocked.to_lowercase() == app_id_lower || app_id_lower.contains(&blocked.to_lowercase()) {
                return Err(SecurityError::BlockedApplication(app_id.to_string()));
            }
        }

        // 2. Whitelist check
        match self.config.mode {
            crate::config::SecurityMode::Allowlist => {
                for allowed in &self.config.allowed_apps {
                    if allowed.to_lowercase() == app_id_lower || app_id_lower.contains(&allowed.to_lowercase()) {
                        return Ok(());
                    }
                }
                Err(SecurityError::ApplicationNotAllowed(app_id.to_string()))
            }
            crate::config::SecurityMode::Blocklist => Ok(()),
        }
    }

    pub fn validate_point_in_window(&self, x: f64, y: f64, window_rect: &Rect) -> Result<(), SecurityError> {
        if window_rect.contains(x, y) {
            Ok(())
        } else {
            Err(SecurityError::CoordinatesOutOfBounds {
                x,
                y,
                min_x: window_rect.x,
                min_y: window_rect.y,
                max_x: window_rect.x + window_rect.width,
                max_y: window_rect.y + window_rect.height,
            })
        }
    }

    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }
}
