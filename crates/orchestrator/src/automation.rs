use async_trait::async_trait;

use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowTarget {
    pub id: String,
    pub title: String,
    pub class_name: Option<String>,
    pub process_id: Option<u32>,
    pub bounds: voxy_shared::types::Rect,
    pub is_visible: bool,
    pub is_focused: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ElementSelector {
    pub automation_id: Option<String>,
    pub name: Option<String>,
    pub class_name: Option<String>,
    pub control_type: Option<String>,
    pub text: Option<String>,
    pub index: Option<usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ElementInfo {
    pub id: String,
    pub name: String,
    pub control_type: String,
    pub bounds: voxy_shared::types::Rect,
    pub is_enabled: bool,
    pub is_visible: bool,
    pub text: Option<String>,
    pub children: Vec<ElementInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateVerification {
    pub window_title: Option<String>,
    pub element_present: Option<ElementSelector>,
    pub text_visible: Option<String>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AutomationCapability {
    Mouse,
    Keyboard,
    ScreenCapture,
    WindowManagement,
    ElementDetection,
    Ocr,
    StateVerification,
    Recovery,
    Hybrid,
}

#[async_trait]
pub trait AutomationBackend: Send + Sync {
    async fn name(&self) -> &str;
    async fn initialize(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
    async fn is_available(&self) -> bool;

    async fn click(&self, x: i32, y: i32, button: MouseButton) -> Result<()>;
    async fn double_click(&self, x: i32, y: i32) -> Result<()>;
    async fn move_mouse(&self, x: i32, y: i32) -> Result<()>;
    async fn drag(&self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<()>;
    async fn scroll(&self, x: i32, y: i32, delta_x: i32, delta_y: i32) -> Result<()>;

    async fn type_text(&self, text: &str, interval_ms: u64) -> Result<()>;
    async fn key_press(&self, key: &str) -> Result<()>;
    async fn key_combination(&self, keys: &[&str]) -> Result<()>;
    async fn hold_key(&self, key: &str, duration_ms: u64) -> Result<()>;

    async fn screenshot(
        &self,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Vec<u8>>;
    async fn screen_size(&self) -> Result<(u32, u32)>;
    async fn get_pixel_color(&self, x: i32, y: i32) -> Result<(u8, u8, u8)>;

    async fn get_active_window(&self) -> Result<WindowTarget>;
    async fn find_window(&self, title: &str, class: Option<&str>) -> Result<Vec<WindowTarget>>;
    async fn focus_window(&self, window_id: &str) -> Result<()>;
    async fn get_window_bounds(&self, window_id: &str) -> Result<voxy_shared::types::Rect>;
    async fn resize_window(&self, window_id: &str, width: u32, height: u32) -> Result<()>;
    async fn move_window(&self, window_id: &str, x: i32, y: i32) -> Result<()>;
    async fn close_window(&self, window_id: &str) -> Result<()>;
    async fn minimize_window(&self, window_id: &str) -> Result<()>;
    async fn maximize_window(&self, window_id: &str) -> Result<()>;
    async fn restore_window(&self, window_id: &str) -> Result<()>;

    async fn find_element(&self, selector: &ElementSelector) -> Result<Vec<ElementInfo>>;
    async fn get_element_text(&self, element_id: &str) -> Result<String>;
    async fn click_element(&self, element_id: &str) -> Result<()>;
    async fn get_element_bounds(&self, element_id: &str) -> Result<voxy_shared::types::Rect>;
    async fn wait_for_element(
        &self,
        selector: &ElementSelector,
        timeout_ms: u64,
    ) -> Result<ElementInfo>;

    async fn ocr_region(&self, image: &[u8], language: Option<&str>) -> Result<String>;
    async fn find_text_on_screen(
        &self,
        text: &str,
        region: Option<voxy_shared::types::Rect>,
    ) -> Result<Vec<voxy_shared::types::Rect>>;

    async fn verify_state(&self, expected: &StateVerification) -> Result<bool>;
    async fn recover(&self, error: &str) -> Result<bool>;
    async fn get_backend_capabilities(&self) -> Vec<AutomationCapability>;
}
