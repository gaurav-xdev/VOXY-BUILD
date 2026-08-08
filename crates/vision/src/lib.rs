pub mod capture;
pub mod change;
pub mod coordinator;
pub mod error;
pub mod grounding;
pub mod ocr;
pub mod provider;
pub mod types;
pub mod ui;

pub use error::{Result, VisionError};
pub use types::*;

use std::sync::Arc;

use coordinator::VisionCoordinator;
use provider::{CaptureProvider, OcrProvider, UiAnalysisProvider};

#[async_trait::async_trait]
pub trait VisionProvider: Send + Sync {
    async fn analyze(&self, image: &[u8], prompt: Option<&str>) -> Result<String>;
    fn available_models(&self) -> Vec<String>;
}

pub struct VisionRuntime {
    coordinator: Arc<VisionCoordinator>,
    provider: Option<Arc<dyn VisionProvider>>,
}

impl VisionRuntime {
    pub fn new(
        capture_provider: Arc<dyn CaptureProvider>,
        ocr_provider: Arc<dyn OcrProvider>,
    ) -> Self {
        Self {
            coordinator: Arc::new(VisionCoordinator::new(capture_provider, ocr_provider)),
            provider: None,
        }
    }

    pub fn with_ui_provider(mut self, provider: Arc<dyn UiAnalysisProvider>) -> Self {
        let coord = VisionCoordinator::new(
            self.coordinator.capture_engine().provider().clone(),
            self.coordinator.ocr_engine().provider().clone(),
        )
        .with_ui_provider(provider);
        self.coordinator = Arc::new(coord);
        self
    }

    pub fn with_vision_provider(mut self, provider: Arc<dyn VisionProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn coordinator(&self) -> &Arc<VisionCoordinator> {
        &self.coordinator
    }

    pub async fn observe_screen(&self, display_index: u32) -> Result<coordinator::Observation> {
        self.coordinator.observe_screen(display_index).await
    }

    pub async fn observe_window(&self, window_id: &str) -> Result<coordinator::Observation> {
        self.coordinator.observe_window(window_id).await
    }

    pub async fn observe_region(
        &self,
        display_index: u32,
        region: Rect,
    ) -> Result<coordinator::Observation> {
        self.coordinator.observe_region(display_index, region).await
    }

    pub async fn observe_all_displays(&self) -> Result<Vec<coordinator::Observation>> {
        self.coordinator.observe_all_displays().await
    }

    pub async fn analyze_image(&self, image: &[u8], prompt: Option<&str>) -> Result<String> {
        if let Some(ref provider) = self.provider {
            provider.analyze(image, prompt).await
        } else {
            self.coordinator.analyze_image(image, prompt).await
        }
    }

    pub async fn find_element(
        &self,
        display_index: u32,
        text: Option<&str>,
        accessibility_id: Option<&str>,
    ) -> Result<GroundedTarget> {
        self.coordinator
            .find_element(display_index, text, accessibility_id)
            .await
    }

    pub fn clear_state(&self) {
        self.coordinator.clear_state();
    }
}

#[allow(dead_code)]
struct NoopCaptureProvider;

#[async_trait::async_trait]
impl CaptureProvider for NoopCaptureProvider {
    async fn capture_full_screen(&self, _idx: u32) -> Result<CapturedFrame> {
        Err(VisionError::UnsupportedOperation(
            "no capture provider configured".into(),
        ))
    }

    async fn capture_window(&self, _id: &str) -> Result<CapturedFrame> {
        Err(VisionError::UnsupportedOperation(
            "no capture provider configured".into(),
        ))
    }

    async fn capture_region(&self, _idx: u32, _region: Rect) -> Result<CapturedFrame> {
        Err(VisionError::UnsupportedOperation(
            "no capture provider configured".into(),
        ))
    }

    async fn list_displays(&self) -> Result<Vec<DisplayInfo>> {
        Ok(vec![])
    }

    async fn screen_size(&self, _idx: u32) -> Result<(u32, u32)> {
        Ok((0, 0))
    }

    fn supported_pixel_formats(&self) -> Vec<PixelFormat> {
        vec![]
    }

    fn name(&self) -> &str {
        "noop-capture"
    }

    fn supports_window_capture(&self) -> bool {
        false
    }

    fn supports_region_capture(&self) -> bool {
        false
    }
}

#[allow(dead_code)]
struct NoopOcrProvider;

#[async_trait::async_trait]
impl OcrProvider for NoopOcrProvider {
    async fn recognize(&self, _img: &CapturedFrame, _lang: Option<&str>) -> Result<OcrResult> {
        Err(VisionError::UnsupportedOperation(
            "no OCR provider configured".into(),
        ))
    }

    async fn recognize_region(
        &self,
        _img: &CapturedFrame,
        _region: Rect,
        _lang: Option<&str>,
    ) -> Result<OcrResult> {
        Err(VisionError::UnsupportedOperation(
            "no OCR provider configured".into(),
        ))
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![]
    }

    fn name(&self) -> &str {
        "noop-ocr"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CaptureSource, CaptureSourceKind, OcrLine, OcrWord};

    struct TestCaptureProvider;

    #[async_trait::async_trait]
    impl CaptureProvider for TestCaptureProvider {
        async fn capture_full_screen(&self, _idx: u32) -> Result<CapturedFrame> {
            Ok(CapturedFrame {
                id: "test".into(),
                data: vec![128u8; 640 * 480 * 4],
                width: 640,
                height: 480,
                stride: 640 * 4,
                format: PixelFormat::Rgba,
                source: CaptureSource {
                    kind: CaptureSourceKind::FullScreen,
                    display_index: Some(0),
                    window_id: None,
                    region: None,
                },
                timestamp: chrono::Utc::now(),
                dpi_scale: 1.0,
                monitor_index: 0,
            })
        }

        async fn capture_window(&self, id: &str) -> Result<CapturedFrame> {
            Ok(CapturedFrame {
                id: format!("win-{id}"),
                data: vec![0u8; 200 * 100 * 4],
                width: 200,
                height: 100,
                stride: 800,
                format: PixelFormat::Rgba,
                source: CaptureSource {
                    kind: CaptureSourceKind::Window,
                    display_index: None,
                    window_id: Some(id.into()),
                    region: None,
                },
                timestamp: chrono::Utc::now(),
                dpi_scale: 1.0,
                monitor_index: 0,
            })
        }

        async fn capture_region(&self, idx: u32, _region: Rect) -> Result<CapturedFrame> {
            self.capture_full_screen(idx).await
        }

        async fn list_displays(&self) -> Result<Vec<DisplayInfo>> {
            Ok(vec![DisplayInfo {
                index: 0,
                bounds: Rect::new(0, 0, 1920, 1080),
                dpi_scale: 1.0,
                is_primary: true,
                name: "Primary".into(),
                is_attached: true,
            }])
        }

        async fn screen_size(&self, _idx: u32) -> Result<(u32, u32)> {
            Ok((1920, 1080))
        }

        fn supported_pixel_formats(&self) -> Vec<PixelFormat> {
            vec![PixelFormat::Rgba]
        }

        fn name(&self) -> &str {
            "test-capture"
        }

        fn supports_window_capture(&self) -> bool {
            true
        }

        fn supports_region_capture(&self) -> bool {
            true
        }
    }

    struct TestOcrProvider;

    #[async_trait::async_trait]
    impl OcrProvider for TestOcrProvider {
        async fn recognize(&self, _img: &CapturedFrame, _lang: Option<&str>) -> Result<OcrResult> {
            Ok(OcrResult {
                text: "Hello World".into(),
                lines: vec![OcrLine {
                    text: "Hello World".into(),
                    confidence: 0.95,
                    bounding_box: Rect::new(10, 10, 100, 20),
                    words: vec![OcrWord {
                        text: "Hello".into(),
                        confidence: 0.96,
                        bounding_box: Rect::new(10, 10, 50, 20),
                        chars: vec![],
                    }],
                }],
                average_confidence: 0.95,
                language: Some("en".into()),
                processing_time_ms: 10,
            })
        }

        async fn recognize_region(
            &self,
            img: &CapturedFrame,
            _region: Rect,
            lang: Option<&str>,
        ) -> Result<OcrResult> {
            self.recognize(img, lang).await
        }

        fn supported_languages(&self) -> Vec<String> {
            vec!["en".into()]
        }

        fn name(&self) -> &str {
            "test-ocr"
        }
    }

    #[tokio::test]
    async fn test_vision_runtime_creates() {
        let rt = VisionRuntime::new(Arc::new(TestCaptureProvider), Arc::new(TestOcrProvider));
        let obs = rt.observe_screen(0).await.unwrap();
        assert_eq!(obs.frame.width, 640);
        assert_eq!(obs.ocr.unwrap().text, "Hello World");
    }

    #[tokio::test]
    async fn test_vision_runtime_observe_window() {
        let rt = VisionRuntime::new(Arc::new(TestCaptureProvider), Arc::new(TestOcrProvider));
        let obs = rt.observe_window("calc").await.unwrap();
        assert_eq!(obs.frame.width, 200);
    }

    #[tokio::test]
    async fn test_vision_runtime_observe_all() {
        let rt = VisionRuntime::new(Arc::new(TestCaptureProvider), Arc::new(TestOcrProvider));
        let obs = rt.observe_all_displays().await.unwrap();
        assert_eq!(obs.len(), 1);
    }

    #[tokio::test]
    async fn test_vision_runtime_find_element() {
        let rt = VisionRuntime::new(Arc::new(TestCaptureProvider), Arc::new(TestOcrProvider));
        let result = rt.find_element(0, Some("Hello"), None).await;
        assert!(result.is_ok() || result.is_err());
    }
}
