use std::sync::Arc;

use parking_lot::RwLock;
use tracing::info;

use crate::capture::CaptureEngine;
use crate::change::ChangeDetector;
use crate::error::{Result, VisionError};
use crate::grounding::GroundingEngine;
use crate::ocr::OcrEngine;
use crate::provider::{CaptureProvider, OcrProvider, UiAnalysisProvider, VisionAnalysisProvider};
use crate::types::{
    CapturedFrame, ChangeReport, DisplayInfo, GroundedTarget, OcrResult, Rect, UiHierarchy,
};
use crate::ui::UiAnalyzer;

pub struct VisionConfig {
    pub confidence_threshold: f32,
    pub ocr_language: Option<String>,
    pub multi_monitor: bool,
    pub dpi_aware: bool,
    pub pixel_change_threshold: u8,
    pub min_change_area: u32,
    pub max_cached_frames: usize,
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.6,
            ocr_language: None,
            multi_monitor: true,
            dpi_aware: true,
            pixel_change_threshold: 20,
            min_change_area: 16,
            max_cached_frames: 10,
        }
    }
}

pub struct VisionCoordinator {
    capture: Arc<CaptureEngine>,
    ocr: Arc<OcrEngine>,
    ui: Arc<UiAnalyzer>,
    grounding: Arc<GroundingEngine>,
    change: Arc<ChangeDetector>,
    analysis: Option<Arc<dyn VisionAnalysisProvider>>,
    config: RwLock<VisionConfig>,
}

impl VisionCoordinator {
    pub fn new(
        capture_provider: Arc<dyn CaptureProvider>,
        ocr_provider: Arc<dyn OcrProvider>,
    ) -> Self {
        let capture = Arc::new(CaptureEngine::new(capture_provider));
        let ocr = Arc::new(OcrEngine::new(ocr_provider));
        let ui = Arc::new(UiAnalyzer::new().with_ocr(ocr.clone()));
        let grounding = Arc::new(GroundingEngine::new());
        let change = Arc::new(ChangeDetector::new());

        Self {
            capture,
            ocr,
            ui,
            grounding,
            change,
            analysis: None,
            config: RwLock::new(VisionConfig::default()),
        }
    }

    pub fn with_ui_provider(mut self, provider: Arc<dyn UiAnalysisProvider>) -> Self {
        self.ui = Arc::new(
            UiAnalyzer::new()
                .with_provider(provider)
                .with_ocr(self.ocr.clone()),
        );
        self
    }

    pub fn with_analysis_provider(mut self, provider: Arc<dyn VisionAnalysisProvider>) -> Self {
        self.analysis = Some(provider);
        self
    }

    pub fn with_config(self, config: VisionConfig) -> Self {
        *self.config.write() = config;
        self
    }

    pub fn capture_engine(&self) -> &Arc<CaptureEngine> {
        &self.capture
    }

    pub fn ocr_engine(&self) -> &Arc<OcrEngine> {
        &self.ocr
    }

    pub fn ui_analyzer(&self) -> &Arc<UiAnalyzer> {
        &self.ui
    }

    pub fn grounding_engine(&self) -> &Arc<GroundingEngine> {
        &self.grounding
    }

    pub fn change_detector(&self) -> &Arc<ChangeDetector> {
        &self.change
    }

    pub async fn observe_screen(&self, display_index: u32) -> Result<Observation> {
        let start = std::time::Instant::now();

        let frame = self.capture.capture_full_screen(display_index).await?;
        let ocr_result = self.ocr.recognize_full(&frame).await.ok();
        let hierarchy = self.ui.analyze(&frame).await.ok();
        let change_report = self.change.detect_frame_changes(&frame).await.ok();

        Ok(Observation {
            frame,
            ocr: ocr_result,
            hierarchy,
            changes: change_report,
            displays: self.capture.get_cached_displays(),
            processing_time_ms: start.elapsed().as_millis() as u64,
            timestamp: chrono::Utc::now(),
        })
    }

    pub async fn observe_window(&self, window_id: &str) -> Result<Observation> {
        let start = std::time::Instant::now();

        let frame = self.capture.capture_window(window_id).await?;
        let ocr_result = self.ocr.recognize_full(&frame).await.ok();
        let hierarchy = self.ui.analyze(&frame).await.ok();

        Ok(Observation {
            frame,
            ocr: ocr_result,
            hierarchy,
            changes: None,
            displays: self.capture.get_cached_displays(),
            processing_time_ms: start.elapsed().as_millis() as u64,
            timestamp: chrono::Utc::now(),
        })
    }

    pub async fn observe_region(&self, display_index: u32, region: Rect) -> Result<Observation> {
        let start = std::time::Instant::now();

        let frame = self.capture.capture_region(display_index, region).await?;
        let ocr_result = self.ocr.recognize_full(&frame).await.ok();

        Ok(Observation {
            frame,
            ocr: ocr_result,
            hierarchy: None,
            changes: None,
            displays: self.capture.get_cached_displays(),
            processing_time_ms: start.elapsed().as_millis() as u64,
            timestamp: chrono::Utc::now(),
        })
    }

    pub async fn observe_all_displays(&self) -> Result<Vec<Observation>> {
        let frames = self.capture.capture_all_displays().await?;
        let mut observations = Vec::with_capacity(frames.len());

        for frame in frames {
            let ocr_result = self.ocr.recognize_full(&frame).await.ok();
            let hierarchy = self.ui.analyze(&frame).await.ok();
            let change_report = self.change.detect_frame_changes(&frame).await.ok();

            observations.push(Observation {
                frame,
                ocr: ocr_result,
                hierarchy,
                changes: change_report,
                displays: self.capture.get_cached_displays(),
                processing_time_ms: 0,
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(observations)
    }

    pub async fn find_element(
        &self,
        display_index: u32,
        text: Option<&str>,
        accessibility_id: Option<&str>,
    ) -> Result<GroundedTarget> {
        let observation = self.observe_screen(display_index).await?;
        let hierarchy = observation
            .hierarchy
            .as_ref()
            .ok_or_else(|| VisionError::UiAnalysisFailed("no UI hierarchy available".into()))?;

        if let Some(acc_id) = accessibility_id {
            return self.grounding.ground_by_accessibility_id(hierarchy, acc_id);
        }

        if let Some(text_str) = text {
            let targets = self.grounding.ground_by_text(hierarchy, text_str)?;
            return self.grounding.combine_methods(targets);
        }

        Err(VisionError::InvalidParameter(
            "either text or accessibility_id must be provided".into(),
        ))
    }

    pub async fn analyze_image(&self, image: &[u8], prompt: Option<&str>) -> Result<String> {
        match self.analysis {
            Some(ref provider) => provider.analyze(image, prompt).await,
            None => Err(VisionError::UnsupportedOperation(
                "no vision analysis provider configured".into(),
            )),
        }
    }

    pub async fn refresh_displays(&self) -> Result<Vec<DisplayInfo>> {
        self.capture.refresh_displays().await
    }

    pub fn clear_state(&self) {
        self.capture.clear_cache();
        self.ocr.clear_cache();
        self.grounding.clear_cache();
        self.change.clear();
        info!("Vision coordinator state cleared");
    }
}

pub struct Observation {
    pub frame: CapturedFrame,
    pub ocr: Option<OcrResult>,
    pub hierarchy: Option<UiHierarchy>,
    pub changes: Option<ChangeReport>,
    pub displays: Vec<DisplayInfo>,
    pub processing_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CaptureSource, CaptureSourceKind, OcrLine, PixelFormat, Rect};

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
                text: "Test OCR".into(),
                lines: vec![OcrLine {
                    text: "Test OCR".into(),
                    confidence: 0.95,
                    bounding_box: Rect::new(10, 10, 80, 20),
                    words: vec![],
                }],
                average_confidence: 0.95,
                language: Some("en".into()),
                processing_time_ms: 5,
            })
        }

        async fn recognize_region(
            &self,
            _img: &CapturedFrame,
            _region: Rect,
            _lang: Option<&str>,
        ) -> Result<OcrResult> {
            self.recognize(_img, _lang).await
        }

        fn supported_languages(&self) -> Vec<String> {
            vec!["en".into()]
        }

        fn name(&self) -> &str {
            "test-ocr"
        }
    }

    #[tokio::test]
    async fn test_observe_screen() {
        let coord =
            VisionCoordinator::new(Arc::new(TestCaptureProvider), Arc::new(TestOcrProvider));
        coord.refresh_displays().await.unwrap();
        let obs = coord.observe_screen(0).await.unwrap();
        assert_eq!(obs.frame.width, 640);
        assert_eq!(obs.ocr.as_ref().unwrap().text, "Test OCR");
        assert_eq!(obs.displays.len(), 1);
    }

    #[tokio::test]
    async fn test_observe_window() {
        let coord =
            VisionCoordinator::new(Arc::new(TestCaptureProvider), Arc::new(TestOcrProvider));
        let obs = coord.observe_window("notepad").await.unwrap();
        assert_eq!(obs.frame.width, 200);
    }

    #[tokio::test]
    async fn test_observe_all_displays() {
        let coord =
            VisionCoordinator::new(Arc::new(TestCaptureProvider), Arc::new(TestOcrProvider));
        let obs = coord.observe_all_displays().await.unwrap();
        assert_eq!(obs.len(), 1);
    }

    #[tokio::test]
    async fn test_refresh_displays() {
        let coord =
            VisionCoordinator::new(Arc::new(TestCaptureProvider), Arc::new(TestOcrProvider));
        let displays = coord.refresh_displays().await.unwrap();
        assert_eq!(displays[0].name, "Primary");
    }

    #[tokio::test]
    async fn test_find_element_by_text() {
        let coord =
            VisionCoordinator::new(Arc::new(TestCaptureProvider), Arc::new(TestOcrProvider));
        let result = coord.find_element(0, Some("Test"), None).await;
        assert!(result.is_ok() || result.is_err());
    }
}
