use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::error::Result;
use crate::provider::OcrProvider;
use crate::types::{
    CapturedFrame, OcrLine, OcrResult, OcrWord, Rect, DEFAULT_CONFIDENCE_THRESHOLD,
    MIN_OCR_CONFIDENCE,
};

struct OcrState {
    last_results: VecDeque<OcrResult>,
    language: Option<String>,
}

pub struct OcrEngine {
    provider: Arc<dyn OcrProvider>,
    state: RwLock<OcrState>,
    confidence_threshold: f32,
    max_cached_results: usize,
}

impl OcrEngine {
    pub fn provider(&self) -> &Arc<dyn OcrProvider> {
        &self.provider
    }

    pub fn new(provider: Arc<dyn OcrProvider>) -> Self {
        Self {
            provider,
            state: RwLock::new(OcrState {
                last_results: VecDeque::new(),
                language: None,
            }),
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
            max_cached_results: 10,
        }
    }

    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    pub fn with_language(self, language: &str) -> Self {
        self.state.write().language = Some(language.to_string());
        self
    }

    pub fn supported_languages(&self) -> Vec<String> {
        self.provider.supported_languages()
    }

    pub async fn recognize_full(&self, frame: &CapturedFrame) -> Result<OcrResult> {
        let language = self.state.read().language.clone();
        let result = self.provider.recognize(frame, language.as_deref()).await?;
        self.cache_result(&result);
        Ok(result)
    }

    pub async fn recognize_region(&self, frame: &CapturedFrame, region: Rect) -> Result<OcrResult> {
        let language = self.state.read().language.clone();
        let result = self
            .provider
            .recognize_region(frame, region, language.as_deref())
            .await?;
        self.cache_result(&result);
        Ok(result)
    }

    pub fn filter_by_confidence(&self, result: &OcrResult) -> OcrResult {
        let lines: Vec<OcrLine> = result
            .lines
            .iter()
            .filter_map(|line| {
                if line.confidence < self.confidence_threshold {
                    return None;
                }
                let words: Vec<OcrWord> = line
                    .words
                    .iter()
                    .filter(|w| w.confidence >= self.confidence_threshold)
                    .map(|w| OcrWord {
                        chars: w
                            .chars
                            .iter()
                            .filter(|c| c.confidence >= MIN_OCR_CONFIDENCE)
                            .cloned()
                            .collect(),
                        ..w.clone()
                    })
                    .collect();
                let mut filtered_text = String::new();
                for (i, w) in words.iter().enumerate() {
                    if i > 0 {
                        filtered_text.push(' ');
                    }
                    filtered_text.push_str(&w.text);
                }
                Some(OcrLine {
                    text: if filtered_text.is_empty() {
                        line.text.clone()
                    } else {
                        filtered_text
                    },
                    words,
                    ..line.clone()
                })
            })
            .collect();

        let total_conf: f32 = lines.iter().map(|l| l.confidence).sum();
        let avg_conf = if lines.is_empty() {
            0.0
        } else {
            total_conf / lines.len() as f32
        };

        let text = {
            let mut t = String::new();
            for (i, l) in lines.iter().enumerate() {
                if i > 0 {
                    t.push('\n');
                }
                t.push_str(l.text.as_str());
            }
            t
        };
        OcrResult {
            text,
            lines,
            average_confidence: avg_conf,
            language: result.language.clone(),
            processing_time_ms: result.processing_time_ms,
        }
    }

    pub fn find_text(&self, result: &OcrResult, query: &str) -> Vec<(Rect, f32)> {
        let lower_query = query.to_lowercase();
        let mut matches = Vec::new();
        for line in &result.lines {
            if line.text.to_lowercase().contains(&lower_query) {
                matches.push((line.bounding_box, line.confidence));
            }
        }
        matches
    }

    pub fn get_last_result(&self) -> Option<OcrResult> {
        self.state.read().last_results.back().cloned()
    }

    pub fn clear_cache(&self) {
        self.state.write().last_results.clear();
    }

    fn cache_result(&self, result: &OcrResult) {
        let mut state = self.state.write();
        state.last_results.push_back(result.clone());
        while state.last_results.len() > self.max_cached_results {
            state.last_results.pop_front();
        }
    }
}

pub fn merge_ocr_results(results: &[OcrResult]) -> OcrResult {
    if results.is_empty() {
        return OcrResult {
            text: String::new(),
            lines: Vec::new(),
            average_confidence: 0.0,
            language: None,
            processing_time_ms: 0,
        };
    }
    if results.len() == 1 {
        return results[0].clone();
    }

    let mut all_lines = Vec::new();
    let mut total_conf = 0.0f32;
    let mut total_time = 0u64;

    for r in results {
        all_lines.extend(r.lines.clone());
        total_conf += r.average_confidence;
        total_time += r.processing_time_ms;
    }

    let avg_conf = total_conf / results.len() as f32;
    let text = {
        let mut t = String::new();
        for (i, l) in all_lines.iter().enumerate() {
            if i > 0 {
                t.push('\n');
            }
            t.push_str(l.text.as_str());
        }
        t
    };

    OcrResult {
        text,
        lines: all_lines,
        average_confidence: avg_conf,
        language: results[0].language.clone(),
        processing_time_ms: total_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CaptureSource, CaptureSourceKind, OcrChar, PixelFormat};

    struct TestOcrProvider;

    #[async_trait::async_trait]
    impl OcrProvider for TestOcrProvider {
        async fn recognize(
            &self,
            _image: &CapturedFrame,
            _language: Option<&str>,
        ) -> Result<OcrResult> {
            Ok(OcrResult {
                text: "Hello World".into(),
                lines: vec![OcrLine {
                    text: "Hello World".into(),
                    confidence: 0.95,
                    bounding_box: Rect::new(10, 10, 100, 20),
                    words: vec![
                        OcrWord {
                            text: "Hello".into(),
                            confidence: 0.96,
                            bounding_box: Rect::new(10, 10, 50, 20),
                            chars: vec![
                                OcrChar {
                                    char_value: 'H',
                                    confidence: 0.98,
                                    bounding_box: Rect::new(10, 10, 10, 20),
                                },
                                OcrChar {
                                    char_value: 'e',
                                    confidence: 0.97,
                                    bounding_box: Rect::new(20, 10, 10, 20),
                                },
                            ],
                        },
                        OcrWord {
                            text: "World".into(),
                            confidence: 0.94,
                            bounding_box: Rect::new(60, 10, 50, 20),
                            chars: vec![],
                        },
                    ],
                }],
                average_confidence: 0.95,
                language: Some("en".into()),
                processing_time_ms: 15,
            })
        }

        async fn recognize_region(
            &self,
            _image: &CapturedFrame,
            _region: Rect,
            _language: Option<&str>,
        ) -> Result<OcrResult> {
            Ok(OcrResult {
                text: "Region Text".into(),
                lines: vec![OcrLine {
                    text: "Region Text".into(),
                    confidence: 0.9,
                    bounding_box: Rect::new(0, 0, 80, 15),
                    words: vec![],
                }],
                average_confidence: 0.9,
                language: Some("en".into()),
                processing_time_ms: 10,
            })
        }

        fn supported_languages(&self) -> Vec<String> {
            vec!["en".into(), "fr".into(), "de".into()]
        }

        fn name(&self) -> &str {
            "test-ocr"
        }
    }

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
    async fn test_ocr_engine_creates() {
        let provider = std::sync::Arc::new(TestOcrProvider);
        let engine = OcrEngine::new(provider);
        assert_eq!(engine.supported_languages().len(), 3);
    }

    #[tokio::test]
    async fn test_recognize_full() {
        let provider = std::sync::Arc::new(TestOcrProvider);
        let engine = OcrEngine::new(provider);
        let frame = make_test_frame();
        let result = engine.recognize_full(&frame).await.unwrap();
        assert_eq!(result.text, "Hello World");
        assert!(result.average_confidence > 0.9);
    }

    #[tokio::test]
    async fn test_recognize_region() {
        let provider = std::sync::Arc::new(TestOcrProvider);
        let engine = OcrEngine::new(provider);
        let frame = make_test_frame();
        let result = engine
            .recognize_region(&frame, Rect::new(10, 10, 100, 50))
            .await
            .unwrap();
        assert_eq!(result.text, "Region Text");
    }

    #[test]
    fn test_filter_by_confidence() {
        let provider = std::sync::Arc::new(TestOcrProvider);
        let engine = OcrEngine::new(provider).with_confidence_threshold(0.5);
        let result = OcrResult {
            text: "high low".into(),
            lines: vec![
                OcrLine {
                    text: "high".into(),
                    confidence: 0.9,
                    bounding_box: Rect::new(0, 0, 30, 10),
                    words: vec![OcrWord {
                        text: "high".into(),
                        confidence: 0.9,
                        bounding_box: Rect::new(0, 0, 30, 10),
                        chars: vec![],
                    }],
                },
                OcrLine {
                    text: "low".into(),
                    confidence: 0.3,
                    bounding_box: Rect::new(0, 10, 30, 10),
                    words: vec![OcrWord {
                        text: "low".into(),
                        confidence: 0.3,
                        bounding_box: Rect::new(0, 10, 30, 10),
                        chars: vec![],
                    }],
                },
            ],
            average_confidence: 0.6,
            language: None,
            processing_time_ms: 5,
        };
        let filtered = engine.filter_by_confidence(&result);
        assert_eq!(filtered.lines.len(), 1);
        assert_eq!(filtered.lines[0].text, "high");
    }

    #[test]
    fn test_find_text() {
        let provider = std::sync::Arc::new(TestOcrProvider);
        let engine = OcrEngine::new(provider);
        let result = OcrResult {
            text: "Hello World\nFoo Bar".into(),
            lines: vec![
                OcrLine {
                    text: "Hello World".into(),
                    confidence: 0.95,
                    bounding_box: Rect::new(10, 10, 100, 20),
                    words: vec![],
                },
                OcrLine {
                    text: "Foo Bar".into(),
                    confidence: 0.9,
                    bounding_box: Rect::new(10, 30, 80, 20),
                    words: vec![],
                },
            ],
            average_confidence: 0.925,
            language: None,
            processing_time_ms: 5,
        };
        let matches = engine.find_text(&result, "world");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, Rect::new(10, 10, 100, 20));
    }

    #[test]
    fn test_merge_ocr_results() {
        let r1 = OcrResult {
            text: "Line 1".into(),
            lines: vec![OcrLine {
                text: "Line 1".into(),
                confidence: 0.9,
                bounding_box: Rect::new(0, 0, 50, 10),
                words: vec![],
            }],
            average_confidence: 0.9,
            language: Some("en".into()),
            processing_time_ms: 10,
        };
        let r2 = OcrResult {
            text: "Line 2".into(),
            lines: vec![OcrLine {
                text: "Line 2".into(),
                confidence: 0.8,
                bounding_box: Rect::new(0, 10, 50, 10),
                words: vec![],
            }],
            average_confidence: 0.8,
            language: Some("en".into()),
            processing_time_ms: 15,
        };
        let merged = merge_ocr_results(&[r1, r2]);
        assert_eq!(merged.lines.len(), 2);
        assert!((merged.average_confidence - 0.85).abs() < 0.01);
    }
}
