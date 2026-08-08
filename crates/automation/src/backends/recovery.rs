use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, info, warn};

use crate::error::Result;
use voxy_orchestrator::automation::{
    AutomationBackend, AutomationCapability, ElementInfo, ElementSelector, MouseButton,
    StateVerification, WindowTarget,
};
use voxy_shared::types::Rect;

pub struct RecoveryEngine {
    backend: Arc<dyn AutomationBackend>,
    name: String,
}

impl RecoveryEngine {
    pub fn new(backend: Arc<dyn AutomationBackend>) -> Self {
        Self {
            backend,
            name: "recovery-engine".to_string(),
        }
    }

    pub async fn recover_from_error(&self, error: &str, context: &RecoveryContext) -> Result<bool> {
        info!("Attempting recovery from error: {}", error);

        match context.strategy {
            RecoveryStrategy::Retry => {
                for attempt in 0..context.max_retries {
                    debug!("Retry attempt {}/{}", attempt + 1, context.max_retries);
                    if let Ok(_) = context
                        .retry_operation
                        .as_ref()
                        .map(|op| op())
                        .unwrap_or(Ok(()))
                    {
                        return Ok(true);
                    }
                    if attempt + 1 < context.max_retries {
                        let delay = context.retry_delay_ms * (1 << attempt);
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    }
                }
                Err(crate::error::action_err(format!(
                    "recovery failed after {} retries: {}",
                    context.max_retries, error
                )))
            }
            RecoveryStrategy::FocusWindow => {
                let active = self.backend.get_active_window().await?;
                self.backend.focus_window(&active.id).await?;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                Ok(true)
            }
            RecoveryStrategy::RestoreWindow => {
                let active = self.backend.get_active_window().await?;
                self.backend.restore_window(&active.id).await?;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                Ok(true)
            }
            RecoveryStrategy::ResetInput => {
                for key in &["CONTROL", "ALT", "SHIFT", "WIN"] {
                    let _ = self.backend.key_press(key).await;
                }
                Ok(true)
            }
            RecoveryStrategy::WaitAndRetry => {
                let delay_ms = context.retry_delay_ms.min(5000);
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                if let Some(ref op) = context.retry_operation {
                    op().map(|_| true)
                } else {
                    Ok(true)
                }
            }
            RecoveryStrategy::Screenshot | RecoveryStrategy::Diagnostic => {
                let _ = self.backend.screenshot(None, None, None, None).await;
                warn!(
                    "Recovery strategy {:?} captured diagnostic - manual intervention may be needed",
                    context.strategy
                );
                Ok(false)
            }
            RecoveryStrategy::Abort => Err(crate::error::action_err(format!(
                "aborting due to unrecoverable error: {}",
                error
            ))),
        }
    }
}

#[derive(Debug)]
pub enum RecoveryStrategy {
    Retry,
    FocusWindow,
    RestoreWindow,
    ResetInput,
    WaitAndRetry,
    Screenshot,
    Diagnostic,
    Abort,
}

pub struct RecoveryContext {
    pub strategy: RecoveryStrategy,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub retry_operation: Option<Box<dyn Fn() -> Result<()> + Send + Sync>>,
}

impl RecoveryContext {
    pub fn retry(max_retries: u32, delay_ms: u64) -> Self {
        Self {
            strategy: RecoveryStrategy::Retry,
            max_retries,
            retry_delay_ms: delay_ms,
            retry_operation: None,
        }
    }

    pub fn focus_window() -> Self {
        Self {
            strategy: RecoveryStrategy::FocusWindow,
            max_retries: 1,
            retry_delay_ms: 0,
            retry_operation: None,
        }
    }

    pub fn abort() -> Self {
        Self {
            strategy: RecoveryStrategy::Abort,
            max_retries: 0,
            retry_delay_ms: 0,
            retry_operation: None,
        }
    }
}

#[async_trait]
impl AutomationBackend for RecoveryEngine {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn initialize(&self) -> Result<()> {
        info!("Recovery engine initializing");
        self.backend.initialize().await
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Recovery engine shutting down");
        Ok(())
    }

    async fn is_available(&self) -> bool {
        self.backend.is_available().await
    }

    async fn click(&self, x: i32, y: i32, button: MouseButton) -> Result<()> {
        let result = self.backend.click(x, y, button.clone()).await;
        if result.is_err() {
            let ctx = RecoveryContext::focus_window();
            self.recover_from_error("click failed", &ctx).await?;
            return self.backend.click(x, y, button).await;
        }
        result
    }

    async fn double_click(&self, x: i32, y: i32) -> Result<()> {
        self.click(x, y, MouseButton::Left).await?;
        self.click(x, y, MouseButton::Left).await
    }

    async fn move_mouse(&self, x: i32, y: i32) -> Result<()> {
        let result = self.backend.move_mouse(x, y).await;
        if result.is_err() {
            let ctx = RecoveryContext::retry(2, 100);
            self.recover_from_error("move_mouse failed", &ctx).await?;
            return self.backend.move_mouse(x, y).await;
        }
        result
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
        let result = self.backend.focus_window(window_id).await;
        if result.is_err() {
            let ctx = RecoveryContext::retry(2, 200);
            self.recover_from_error("focus_window failed", &ctx).await?;
            return self.backend.focus_window(window_id).await;
        }
        result
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
        let result = self.backend.click_element(element_id).await;
        if result.is_err() {
            let ctx = RecoveryContext::retry(2, 300);
            self.recover_from_error("click_element failed", &ctx)
                .await?;
            return self.backend.click_element(element_id).await;
        }
        result
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
        self.backend.verify_state(expected).await
    }

    async fn recover(&self, error: &str) -> Result<bool> {
        info!("Recovery engine handling error: {}", error);

        let ctx = RecoveryContext {
            strategy: RecoveryStrategy::WaitAndRetry,
            max_retries: 3,
            retry_delay_ms: 1000,
            retry_operation: None,
        };
        self.recover_from_error(error, &ctx).await
    }

    async fn get_backend_capabilities(&self) -> Vec<AutomationCapability> {
        let mut caps = self.backend.get_backend_capabilities().await;
        caps.push(AutomationCapability::Recovery);
        caps
    }
}
