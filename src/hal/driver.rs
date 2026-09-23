use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::utils::coordinates::Rect;

#[derive(Error, Debug)]
pub enum DriverError {
    #[error("OS API error: {0}")]
    OsError(String),

    #[error("Application not found or failed to launch: {0}")]
    AppNotFound(String),

    #[error("No active window found for application: {0}")]
    NoActiveWindow(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Window capture failed: {0}")]
    CaptureFailed(String),

    #[error("Input injection failed: {0}")]
    InputFailed(String),

    #[error("Accessibility error: {0}")]
    AccessibilityError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseAction {
    Move,
    Click,
    DoubleClick,
    TripleClick,
    RightClick,
    Drag,
    Scroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl Default for MouseButton {
    fn default() -> Self {
        MouseButton::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyAction {
    Type,
    PressKey,
    KeyDown,
    KeyUp,
    Hotkey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBuffer {
    pub base64_data: String,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub format: String,
    pub window_title: String,
    pub app_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub window_id: u64,
    pub app_id: String,
    pub app_name: String,
    pub title: String,
    pub bounds: Rect,
    pub is_active: bool,
    pub pid: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityNode {
    pub role: String,
    pub title: Option<String>,
    pub value: Option<String>,
    pub bounds: Option<Rect>,
    pub children: Vec<AccessibilityNode>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub os: String,
    pub accessibility_granted: bool,
    pub screen_recording_granted: bool,
    pub display_server: String,
    pub active_displays: u32,
    pub active_window: Option<WindowInfo>,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub display_id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_virtual: bool,
    pub is_main: bool,
    pub refresh_rate: f64,
}

#[async_trait]
pub trait DesktopDriver: Send + Sync {
    async fn capture_window(&self, app_id: Option<&str>, window_id: Option<u64>) -> Result<ImageBuffer, DriverError>;
    async fn mouse_action(
        &self,
        action: MouseAction,
        x: f64,
        y: f64,
        click_count: u32,
        button: MouseButton,
        scroll_delta: Option<(i32, i32)>,
        target_app: Option<&str>,
        window_id: Option<u64>,
        coordinate_space: Option<&str>,
    ) -> Result<(), DriverError>;
    async fn keyboard_action(
        &self,
        action: KeyAction,
        text: Option<&str>,
        key: Option<&str>,
        modifiers: &[String],
        target_app: Option<&str>,
        window_id: Option<u64>,
    ) -> Result<(), DriverError>;
    async fn inspect_ui(&self, app_id: Option<&str>, max_depth: u32) -> Result<AccessibilityNode, DriverError>;
    async fn launch_or_focus_app(&self, app_identifier: &str) -> Result<WindowInfo, DriverError>;
    async fn terminate_app(&self, app_identifier: &str) -> Result<(), DriverError>;
    async fn get_active_window(&self) -> Result<WindowInfo, DriverError>;
    async fn check_permissions(&self) -> Result<DoctorReport, DriverError>;
    async fn create_virtual_display(&self, width: u32, height: u32, name: Option<&str>) -> Result<DisplayInfo, DriverError>;
    async fn destroy_virtual_display(&self, display_id: u32) -> Result<(), DriverError>;
    async fn list_displays(&self) -> Result<Vec<DisplayInfo>, DriverError>;
}
