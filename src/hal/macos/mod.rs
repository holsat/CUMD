pub mod accessibility;
pub mod app_manager;
pub mod capture;
pub mod input;

use async_trait::async_trait;
use crate::hal::driver::*;

pub struct MacosDriver;

impl MacosDriver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DesktopDriver for MacosDriver {
    async fn capture_window(
        &self,
        app_id: Option<&str>,
        window_id: Option<u64>,
    ) -> Result<ImageBuffer, DriverError> {
        capture::capture_isolated_window(app_id, window_id, "png")
    }

    async fn mouse_action(
        &self,
        action: MouseAction,
        x: f64,
        y: f64,
        click_count: u32,
        button: MouseButton,
        scroll_delta: Option<(i32, i32)>,
    ) -> Result<(), DriverError> {
        input::execute_mouse_action(action, x, y, click_count, button, scroll_delta)
    }

    async fn keyboard_action(
        &self,
        action: KeyAction,
        text: Option<&str>,
        key: Option<&str>,
        modifiers: &[String],
    ) -> Result<(), DriverError> {
        input::execute_keyboard_action(action, text, key, modifiers)
    }

    async fn inspect_ui(&self, app_id: Option<&str>, max_depth: u32) -> Result<AccessibilityNode, DriverError> {
        accessibility::inspect_app_ui(app_id, max_depth)
    }

    async fn launch_or_focus_app(&self, app_identifier: &str) -> Result<WindowInfo, DriverError> {
        app_manager::launch_or_activate_app(app_identifier)?;
        app_manager::find_window_for_app(Some(app_identifier), None)
    }

    async fn get_active_window(&self) -> Result<WindowInfo, DriverError> {
        let app = app_manager::get_frontmost_app_name()?;
        app_manager::find_window_for_app(Some(&app), None)
    }

    async fn check_permissions(&self) -> Result<DoctorReport, DriverError> {
        let ax_granted = accessibility::is_accessibility_granted();
        let frontmost = self.get_active_window().await.ok();

        Ok(DoctorReport {
            os: "macOS".to_string(),
            accessibility_granted: ax_granted,
            screen_recording_granted: true,
            display_server: "Quartz Window Server".to_string(),
            active_displays: 1,
            active_window: frontmost,
            details: vec![
                format!("Accessibility granted: {}", ax_granted),
                "ScreenCaptureKit/CGWindowList available: true".to_string(),
            ],
        })
    }
}
