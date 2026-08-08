use async_trait::async_trait;
use tracing::info;

use voxy_vision::error::{Result, VisionError};
use voxy_vision::provider::OcrProvider;
use voxy_vision::types::{CapturedFrame, OcrChar, OcrLine, OcrResult, OcrWord, Rect};

pub struct WindowsOcrProvider {
    name: String,
    language: String,
}

impl WindowsOcrProvider {
    pub fn new() -> Self {
        Self {
            name: "windows-ocr".into(),
            language: "en".into(),
        }
    }

    pub fn with_language(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self
    }

    fn extract_ocr_text(frame: &CapturedFrame, language: &str) -> Result<OcrResult> {
        info!("Performing OCR via simulated provider (lang: {})", language);

        let start = std::time::Instant::now();

        let raw_text = format!(
            "Windows OCR Simulation - {}x{} frame",
            frame.width, frame.height
        );

        let line = OcrLine {
            text: raw_text.clone(),
            confidence: 0.85,
            bounding_box: Rect::new(0, 0, frame.width, 20),
            words: vec![OcrWord {
                text: raw_text.clone(),
                confidence: 0.85,
                bounding_box: Rect::new(0, 0, frame.width, 20),
                chars: raw_text
                    .chars()
                    .enumerate()
                    .map(|(i, c)| OcrChar {
                        char_value: c,
                        confidence: 0.85,
                        bounding_box: Rect::new((i as u32 * 8) as i32, 0, 8, 20),
                    })
                    .collect(),
            }],
        };

        Ok(OcrResult {
            text: raw_text,
            lines: vec![line],
            average_confidence: 0.85,
            language: Some(language.to_string()),
            processing_time_ms: start.elapsed().as_millis() as u64,
        })
    }
}

impl Default for WindowsOcrProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OcrProvider for WindowsOcrProvider {
    async fn recognize(&self, frame: &CapturedFrame, language: Option<&str>) -> Result<OcrResult> {
        let lang = language.unwrap_or(&self.language);
        if cfg!(windows) {
            Self::extract_ocr_text(frame, lang)
        } else {
            Err(VisionError::UnsupportedOperation(
                "Windows OCR is only available on Windows".into(),
            ))
        }
    }

    async fn recognize_region(
        &self,
        frame: &CapturedFrame,
        region: Rect,
        language: Option<&str>,
    ) -> Result<OcrResult> {
        if !cfg!(windows) {
            return Err(VisionError::UnsupportedOperation(
                "Windows OCR is only available on Windows".into(),
            ));
        }
        let cropped = voxy_vision::capture::crop_frame(frame, region)?;
        let lang = language.unwrap_or(&self.language);
        Self::extract_ocr_text(&cropped, lang)
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "en".into(),
            "fr".into(),
            "de".into(),
            "es".into(),
            "it".into(),
            "pt".into(),
            "zh".into(),
            "ja".into(),
            "ko".into(),
            "ru".into(),
            "ar".into(),
        ]
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxy_vision::types::{CaptureSource, CaptureSourceKind, PixelFormat};

    fn make_test_frame() -> CapturedFrame {
        CapturedFrame {
            id: "ocr-test".into(),
            data: vec![0u8; 640 * 480 * 4],
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
        }
    }

    #[tokio::test]
    async fn test_provider_creates() {
        let provider = WindowsOcrProvider::new();
        assert_eq!(provider.name(), "windows-ocr");
        assert!(provider.supported_languages().len() >= 10);
    }

    #[tokio::test]
    async fn test_recognize() {
        let provider = WindowsOcrProvider::new();
        let frame = make_test_frame();
        let result = provider.recognize(&frame, Some("en")).await;
        if cfg!(windows) {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_recognize_with_default_language() {
        let provider = WindowsOcrProvider::new().with_language("fr");
        let frame = make_test_frame();
        let result = provider.recognize(&frame, None).await;
        if cfg!(windows) {
            assert!(result.is_ok());
        }
    }
}
