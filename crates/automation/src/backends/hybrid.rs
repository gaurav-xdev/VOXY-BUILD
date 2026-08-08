use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::{debug, info};

use crate::error::Result;
use voxy_orchestrator::automation::{
    AutomationBackend, AutomationCapability, ElementInfo, ElementSelector, MouseButton,
    StateVerification, WindowTarget,
};
use voxy_orchestrator::OrchestratorError;
use voxy_shared::types::Rect;

struct BackendPair {
    primary: Arc<dyn AutomationBackend>,
    fallback: Arc<dyn AutomationBackend>,
}

pub struct HybridBackend {
    backends: RwLock<Option<BackendPair>>,
    name: String,
}

pub struct HybridBuilder {
    primary: Option<Arc<dyn AutomationBackend>>,
    fallback: Option<Arc<dyn AutomationBackend>>,
    config: HybridConfig,
}

#[derive(Clone)]
pub struct HybridConfig {
    pub primary_backend_name: String,
    pub fallback_on_failure: bool,
    pub latency_threshold_ms: u64,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            primary_backend_name: "windows-uia".into(),
            fallback_on_failure: true,
            latency_threshold_ms: 100,
        }
    }
}

impl HybridBuilder {
    pub fn new() -> Self {
        Self {
            primary: None,
            fallback: None,
            config: HybridConfig::default(),
        }
    }

    pub fn with_primary(mut self, backend: Arc<dyn AutomationBackend>) -> Self {
        self.primary = Some(backend.clone());
        self.fallback = Some(backend);
        self
    }

    pub fn with_backends(
        mut self,
        primary: Arc<dyn AutomationBackend>,
        fallback: Arc<dyn AutomationBackend>,
    ) -> Self {
        self.primary = Some(primary);
        self.fallback = Some(fallback);
        self
    }

    pub fn with_config(mut self, config: HybridConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> Result<HybridBackend> {
        let primary = self.primary.ok_or_else(|| {
            OrchestratorError::AutomationError("no primary backend configured".into())
        })?;
        let fallback = self.fallback.ok_or_else(|| {
            OrchestratorError::AutomationError("no fallback backend configured".into())
        })?;

        Ok(HybridBackend {
            backends: RwLock::new(Some(BackendPair { primary, fallback })),
            name: "hybrid".into(),
        })
    }
}

impl HybridBackend {
    pub fn builder() -> HybridBuilder {
        HybridBuilder::new()
    }

    fn get_pair_arcs(
        &self,
    ) -> std::result::Result<
        (Arc<dyn AutomationBackend>, Arc<dyn AutomationBackend>),
        OrchestratorError,
    > {
        let backends = self.backends.read();
        backends
            .as_ref()
            .map(|p| (p.primary.clone(), p.fallback.clone()))
            .ok_or_else(|| OrchestratorError::AutomationError("no backends available".into()))
    }
}

#[async_trait]
impl AutomationBackend for HybridBackend {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn initialize(&self) -> Result<()> {
        info!("Hybrid backend initializing with adaptive strategy");
        let (primary, fallback) = self.get_pair_arcs()?;
        primary.initialize().await?;
        fallback.initialize().await.ok();
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Hybrid backend shutting down");
        if let Ok((primary, fallback)) = self.get_pair_arcs() {
            primary.shutdown().await?;
            fallback.shutdown().await.ok();
        }
        Ok(())
    }

    async fn is_available(&self) -> bool {
        let (primary, _) = self.get_pair_arcs().unwrap_or((
            Arc::new(crate::backends::windows_uia::WindowsUiaBackend::new())
                as Arc<dyn AutomationBackend>,
            Arc::new(crate::backends::windows_uia::WindowsUiaBackend::new())
                as Arc<dyn AutomationBackend>,
        ));
        primary.is_available().await
    }

    async fn click(&self, x: i32, y: i32, button: MouseButton) -> Result<()> {
        let (primary, fallback) = self.get_pair_arcs()?;
        match primary.click(x, y, button).await {
            Ok(r) => Ok(r),
            Err(e) => {
                debug!("Primary click failed, trying fallback: {}", e);
                fallback.click(x, y, button).await
            }
        }
    }

    async fn double_click(&self, x: i32, y: i32) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.double_click(x, y).await
    }

    async fn move_mouse(&self, x: i32, y: i32) -> Result<()> {
        let (primary, fallback) = self.get_pair_arcs()?;
        match primary.move_mouse(x, y).await {
            Ok(r) => Ok(r),
            Err(e) => {
                debug!("Primary move_mouse failed, trying fallback: {}", e);
                fallback.move_mouse(x, y).await
            }
        }
    }

    async fn drag(&self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.drag(from_x, from_y, to_x, to_y).await
    }

    async fn scroll(&self, x: i32, y: i32, delta_x: i32, delta_y: i32) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.scroll(x, y, delta_x, delta_y).await
    }

    async fn type_text(&self, text: &str, interval_ms: u64) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.type_text(text, interval_ms).await
    }

    async fn key_press(&self, key: &str) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.key_press(key).await
    }

    async fn key_combination(&self, keys: &[&str]) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.key_combination(keys).await
    }

    async fn hold_key(&self, key: &str, duration_ms: u64) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.hold_key(key, duration_ms).await
    }

    async fn screenshot(
        &self,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Vec<u8>> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.screenshot(x, y, width, height).await
    }

    async fn screen_size(&self) -> Result<(u32, u32)> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.screen_size().await
    }

    async fn get_pixel_color(&self, x: i32, y: i32) -> Result<(u8, u8, u8)> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.get_pixel_color(x, y).await
    }

    async fn get_active_window(&self) -> Result<WindowTarget> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.get_active_window().await
    }

    async fn find_window(&self, title: &str, class: Option<&str>) -> Result<Vec<WindowTarget>> {
        let (primary, fallback) = self.get_pair_arcs()?;
        let result = primary.find_window(title, class).await;
        match result {
            Ok(w) if !w.is_empty() => Ok(w),
            _ => fallback.find_window(title, class).await.or(result),
        }
    }

    async fn focus_window(&self, window_id: &str) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.focus_window(window_id).await
    }

    async fn get_window_bounds(&self, window_id: &str) -> Result<Rect> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.get_window_bounds(window_id).await
    }

    async fn resize_window(&self, window_id: &str, width: u32, height: u32) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.resize_window(window_id, width, height).await
    }

    async fn move_window(&self, window_id: &str, x: i32, y: i32) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.move_window(window_id, x, y).await
    }

    async fn close_window(&self, window_id: &str) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.close_window(window_id).await
    }

    async fn minimize_window(&self, window_id: &str) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.minimize_window(window_id).await
    }

    async fn maximize_window(&self, window_id: &str) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.maximize_window(window_id).await
    }

    async fn restore_window(&self, window_id: &str) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.restore_window(window_id).await
    }

    async fn find_element(&self, selector: &ElementSelector) -> Result<Vec<ElementInfo>> {
        let (primary, fallback) = self.get_pair_arcs()?;
        let result = primary.find_element(selector).await;
        match result {
            Ok(e) if !e.is_empty() => Ok(e),
            _ => fallback.find_element(selector).await.or(result),
        }
    }

    async fn get_element_text(&self, element_id: &str) -> Result<String> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.get_element_text(element_id).await
    }

    async fn click_element(&self, element_id: &str) -> Result<()> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.click_element(element_id).await
    }

    async fn get_element_bounds(&self, element_id: &str) -> Result<Rect> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.get_element_bounds(element_id).await
    }

    async fn wait_for_element(
        &self,
        selector: &ElementSelector,
        timeout_ms: u64,
    ) -> Result<ElementInfo> {
        let (primary, _) = self.get_pair_arcs()?;
        primary.wait_for_element(selector, timeout_ms).await
    }

    async fn ocr_region(&self, image: &[u8], language: Option<&str>) -> Result<String> {
        let (_, fallback) = self.get_pair_arcs()?;
        fallback.ocr_region(image, language).await
    }

    async fn find_text_on_screen(&self, text: &str, region: Option<Rect>) -> Result<Vec<Rect>> {
        let (_, fallback) = self.get_pair_arcs()?;
        fallback.find_text_on_screen(text, region).await
    }

    async fn verify_state(&self, expected: &StateVerification) -> Result<bool> {
        let (primary, fallback) = self.get_pair_arcs()?;
        if let Ok(true) = primary.verify_state(expected).await {
            return Ok(true);
        }
        fallback.verify_state(expected).await
    }

    async fn recover(&self, error: &str) -> Result<bool> {
        let (_, fallback) = self.get_pair_arcs()?;
        fallback.recover(error).await
    }

    async fn get_backend_capabilities(&self) -> Vec<AutomationCapability> {
        let pair = self.get_pair_arcs();
        match pair {
            Ok((primary, fallback)) => {
                let mut caps = primary.get_backend_capabilities().await;
                let fallback_caps = fallback.get_backend_capabilities().await;
                for cap in fallback_caps {
                    if !caps.contains(&cap) {
                        caps.push(cap);
                    }
                }
                if !caps.contains(&AutomationCapability::Hybrid) {
                    caps.push(AutomationCapability::Hybrid);
                }
                caps
            }
            _ => vec![AutomationCapability::Hybrid],
        }
    }
}
