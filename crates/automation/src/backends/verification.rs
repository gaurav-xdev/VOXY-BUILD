use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, info};

use crate::error::Result;
use voxy_orchestrator::automation::{
    AutomationBackend, AutomationCapability, ElementInfo, ElementSelector, MouseButton,
    StateVerification, WindowTarget,
};
use voxy_shared::types::Rect;

pub struct VerificationEngine {
    backend: Arc<dyn AutomationBackend>,
    name: String,
    screenshot_before: bool,
    screenshot_after: bool,
}

impl VerificationEngine {
    pub fn new(backend: Arc<dyn AutomationBackend>) -> Self {
        Self {
            backend,
            name: "verification-engine".to_string(),
            screenshot_before: true,
            screenshot_after: true,
        }
    }

    pub fn with_screenshots(mut self, before: bool, after: bool) -> Self {
        self.screenshot_before = before;
        self.screenshot_after = after;
        self
    }

    pub async fn verify_window_state(&self, expected: &StateVerification) -> Result<bool> {
        let start = std::time::Instant::now();

        if let Some(ref title) = expected.window_title {
            let windows = self.backend.find_window(title, None).await?;
            if windows.is_empty() {
                return Ok(false);
            }

            let any_focused = windows.iter().any(|w| w.is_focused);
            if !any_focused {
                if let Some(win) = windows.first() {
                    self.backend.focus_window(&win.id).await?;
                }
            }
        }

        if let Some(ref selector) = expected.element_present {
            match self
                .backend
                .wait_for_element(selector, expected.timeout_ms)
                .await
            {
                Ok(_) => {}
                Err(voxy_orchestrator::OrchestratorError::Timeout(_)) => return Ok(false),
                Err(e) => return Err(e),
            }
        }

        if let Some(ref text) = expected.text_visible {
            let result = self
                .backend
                .find_text_on_screen(text, None)
                .await
                .unwrap_or_default();
            if result.is_empty() {
                return Ok(false);
            }
        }

        let elapsed = start.elapsed().as_millis();
        debug!("verify_window_state completed in {}ms", elapsed);
        Ok(true)
    }
}

#[async_trait]
impl AutomationBackend for VerificationEngine {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn initialize(&self) -> Result<()> {
        info!("Verification engine initializing");
        self.backend.initialize().await
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Verification engine shutting down");
        Ok(())
    }

    async fn is_available(&self) -> bool {
        self.backend.is_available().await
    }

    async fn click(&self, x: i32, y: i32, button: MouseButton) -> Result<()> {
        if self.screenshot_before {
            let _ = self.backend.screenshot(None, None, None, None).await;
        }
        self.backend.click(x, y, button).await?;
        if self.screenshot_after {
            let _ = self.backend.screenshot(None, None, None, None).await;
        }
        Ok(())
    }

    async fn double_click(&self, x: i32, y: i32) -> Result<()> {
        self.click(x, y, MouseButton::Left).await?;
        self.click(x, y, MouseButton::Left).await
    }

    async fn move_mouse(&self, x: i32, y: i32) -> Result<()> {
        self.backend.move_mouse(x, y).await
    }

    async fn drag(&self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<()> {
        self.backend.drag(from_x, from_y, to_x, to_y).await
    }

    async fn scroll(&self, x: i32, y: i32, delta_x: i32, delta_y: i32) -> Result<()> {
        self.backend.scroll(x, y, delta_x, delta_y).await
    }

    async fn type_text(&self, text: &str, interval_ms: u64) -> Result<()> {
        self.backend.type_text(text, interval_ms).await
    }

    async fn key_press(&self, key: &str) -> Result<()> {
        self.backend.key_press(key).await
    }

    async fn key_combination(&self, keys: &[&str]) -> Result<()> {
        self.backend.key_combination(keys).await
    }

    async fn hold_key(&self, key: &str, duration_ms: u64) -> Result<()> {
        self.backend.hold_key(key, duration_ms).await
    }

    async fn screenshot(
        &self,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Vec<u8>> {
        self.backend.screenshot(x, y, width, height).await
    }

    async fn screen_size(&self) -> Result<(u32, u32)> {
        self.backend.screen_size().await
    }

    async fn get_pixel_color(&self, x: i32, y: i32) -> Result<(u8, u8, u8)> {
        self.backend.get_pixel_color(x, y).await
    }

    async fn get_active_window(&self) -> Result<WindowTarget> {
        self.backend.get_active_window().await
    }

    async fn find_window(&self, title: &str, class: Option<&str>) -> Result<Vec<WindowTarget>> {
        self.backend.find_window(title, class).await
    }

    async fn focus_window(&self, window_id: &str) -> Result<()> {
        self.backend.focus_window(window_id).await
    }

    async fn get_window_bounds(&self, window_id: &str) -> Result<Rect> {
        self.backend.get_window_bounds(window_id).await
    }

    async fn resize_window(&self, window_id: &str, width: u32, height: u32) -> Result<()> {
        self.backend.resize_window(window_id, width, height).await
    }

    async fn move_window(&self, window_id: &str, x: i32, y: i32) -> Result<()> {
        self.backend.move_window(window_id, x, y).await
    }

    async fn close_window(&self, window_id: &str) -> Result<()> {
        self.backend.close_window(window_id).await
    }

    async fn minimize_window(&self, window_id: &str) -> Result<()> {
        self.backend.minimize_window(window_id).await
    }

    async fn maximize_window(&self, window_id: &str) -> Result<()> {
        self.backend.maximize_window(window_id).await
    }

    async fn restore_window(&self, window_id: &str) -> Result<()> {
        self.backend.restore_window(window_id).await
    }

    async fn find_element(&self, selector: &ElementSelector) -> Result<Vec<ElementInfo>> {
        self.backend.find_element(selector).await
    }

    async fn get_element_text(&self, element_id: &str) -> Result<String> {
        self.backend.get_element_text(element_id).await
    }

    async fn click_element(&self, element_id: &str) -> Result<()> {
        self.backend.click_element(element_id).await
    }

    async fn get_element_bounds(&self, element_id: &str) -> Result<Rect> {
        self.backend.get_element_bounds(element_id).await
    }

    async fn wait_for_element(
        &self,
        selector: &ElementSelector,
        timeout_ms: u64,
    ) -> Result<ElementInfo> {
        self.backend.wait_for_element(selector, timeout_ms).await
    }

    async fn ocr_region(&self, image: &[u8], language: Option<&str>) -> Result<String> {
        self.backend.ocr_region(image, language).await
    }

    async fn find_text_on_screen(&self, text: &str, region: Option<Rect>) -> Result<Vec<Rect>> {
        self.backend.find_text_on_screen(text, region).await
    }

    async fn verify_state(&self, expected: &StateVerification) -> Result<bool> {
        self.verify_window_state(expected).await
    }

    async fn recover(&self, _error: &str) -> Result<bool> {
        Err(crate::error::unsupported_err(
            "recovery delegated to RecoveryEngine",
        ))
    }

    async fn get_backend_capabilities(&self) -> Vec<AutomationCapability> {
        let mut caps = self.backend.get_backend_capabilities().await;
        caps.push(AutomationCapability::StateVerification);
        caps
    }
}
