use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{Result, VisionError};
use crate::ocr::OcrEngine;
use crate::provider::UiAnalysisProvider;
use crate::types::{
    CapturedFrame, OcrResult, SemanticElement, UiElementKind, UiHierarchy,
    DEFAULT_CONFIDENCE_THRESHOLD,
};

pub struct UiAnalyzer {
    provider: Option<Arc<dyn UiAnalysisProvider>>,
    ocr_engine: Option<Arc<OcrEngine>>,
    confidence_threshold: f32,
}

impl UiAnalyzer {
    pub fn new() -> Self {
        Self {
            provider: None,
            ocr_engine: None,
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
        }
    }

    pub fn with_provider(mut self, provider: Arc<dyn UiAnalysisProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn with_ocr(mut self, ocr: Arc<OcrEngine>) -> Self {
        self.ocr_engine = Some(ocr);
        self
    }

    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    pub async fn analyze(&self, frame: &CapturedFrame) -> Result<UiHierarchy> {
        let start = std::time::Instant::now();

        let mut hierarchy = if let Some(ref provider) = self.provider {
            provider.extract_hierarchy(frame).await?
        } else {
            UiHierarchy {
                root: SemanticElement {
                    id: "root".into(),
                    element_type: UiElementKind::Window,
                    text: None,
                    bounds: crate::types::Rect::new(0, 0, frame.width, frame.height),
                    confidence: 1.0,
                    is_interactive: false,
                    is_enabled: true,
                    children: Vec::new(),
                    parent_id: None,
                    properties: HashMap::new(),
                },
                timestamp: chrono::Utc::now(),
                source: frame.source.clone(),
                processing_time_ms: 0,
            }
        };

        if let Some(ref ocr) = self.ocr_engine {
            if let Ok(ocr_result) = ocr.recognize_full(frame).await {
                Self::annotate_with_ocr(&mut hierarchy, &ocr_result);
            }
        }

        hierarchy.processing_time_ms = start.elapsed().as_millis() as u64;
        hierarchy.timestamp = chrono::Utc::now();

        Ok(hierarchy)
    }

    pub async fn find_elements_by_type(
        &self,
        frame: &CapturedFrame,
        element_type: &UiElementKind,
    ) -> Result<Vec<SemanticElement>> {
        if let Some(ref provider) = self.provider {
            let type_str = format!("{:?}", element_type);
            provider.find_elements_by_type(frame, &type_str).await
        } else {
            Err(VisionError::UnsupportedOperation(
                "no UI analysis provider configured".into(),
            ))
        }
    }

    pub async fn find_text_in_image(
        &self,
        frame: &CapturedFrame,
        text: &str,
    ) -> Result<Vec<SemanticElement>> {
        if let Some(ref provider) = self.provider {
            provider.find_text_in_image(frame, text).await
        } else if let Some(ref ocr) = self.ocr_engine {
            let ocr_result = ocr.recognize_full(frame).await?;
            let matches = ocr.find_text(&ocr_result, text);
            if matches.is_empty() {
                return Err(VisionError::ElementNotFound(format!(
                    "text '{}' not found in image",
                    text
                )));
            }
            Ok(matches
                .into_iter()
                .map(|(rect, conf)| SemanticElement {
                    id: uuid::Uuid::new_v4().to_string(),
                    element_type: UiElementKind::Label,
                    text: Some(text.to_string()),
                    bounds: rect,
                    confidence: conf,
                    is_interactive: false,
                    is_enabled: true,
                    children: Vec::new(),
                    parent_id: None,
                    properties: HashMap::new(),
                })
                .collect())
        } else {
            Err(VisionError::UnsupportedOperation(
                "no UI analysis or OCR provider configured".into(),
            ))
        }
    }

    pub fn classify_element(
        &self,
        control_type: &str,
        name: &str,
        bounds: crate::types::Rect,
    ) -> UiElementKind {
        let lower_type = control_type.to_lowercase();
        let lower_name = name.to_lowercase();

        match lower_type.as_str() {
            "button" | "pushbutton" => UiElementKind::Button,
            "edit" | "textbox" | "text" | "richedit" => UiElementKind::TextBox,
            "menubar" | "menuitem" | "menu" => UiElementKind::Menu,
            "dialog" | "window" if lower_name.contains("dialog") => UiElementKind::Dialog,
            "toolbar" => UiElementKind::Toolbar,
            "tab" | "tabitem" => UiElementKind::Tab,
            "list" | "listbox" => UiElementKind::List,
            "listitem" => UiElementKind::ListItem,
            "tree" => UiElementKind::Tree,
            "treenode" | "treeitem" => UiElementKind::TreeNode,
            "statusbar" | "status" => UiElementKind::StatusBar,
            "checkbox" => UiElementKind::CheckBox,
            "radiobutton" | "radio" => UiElementKind::RadioButton,
            "combobox" | "dropdown" => UiElementKind::ComboBox,
            "scrollbar" => UiElementKind::ScrollBar,
            "slider" | "trackbar" => UiElementKind::Slider,
            "progressbar" | "progress" => UiElementKind::ProgressBar,
            "image" | "picture" => UiElementKind::Image,
            "link" | "hyperlink" => UiElementKind::Link,
            "label" | "static" => UiElementKind::Label,
            "table" | "datagrid" => UiElementKind::Table,
            "pane" | "panel" | "groupbox" | "group" => UiElementKind::Pane,
            _ => {
                if lower_name.contains("notification") || lower_name.contains("toast") {
                    UiElementKind::Notification
                } else if lower_name.contains("icon") || bounds.width() < 48 && bounds.height() < 48
                {
                    UiElementKind::Icon
                } else {
                    UiElementKind::Generic(control_type.to_string())
                }
            }
        }
    }

    fn annotate_with_ocr(hierarchy: &mut UiHierarchy, ocr_result: &OcrResult) {
        Self::annotate_element_with_ocr(&mut hierarchy.root, ocr_result);
    }

    fn annotate_element_with_ocr(element: &mut SemanticElement, ocr_result: &OcrResult) {
        if element.text.is_none() {
            for line in &ocr_result.lines {
                if element.bounds.intersects(&line.bounding_box) {
                    element.text = Some(line.text.clone());
                    element.confidence = element.confidence.min(line.confidence);
                    break;
                }
            }
        }
        for child in &mut element.children {
            Self::annotate_element_with_ocr(child, ocr_result);
        }
    }
}

impl Default for UiAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CaptureSource, CaptureSourceKind, PixelFormat};

    struct TestUiProvider;

    #[async_trait::async_trait]
    impl UiAnalysisProvider for TestUiProvider {
        async fn extract_hierarchy(&self, frame: &CapturedFrame) -> Result<UiHierarchy> {
            Ok(UiHierarchy {
                root: SemanticElement {
                    id: "root".into(),
                    element_type: UiElementKind::Window,
                    text: Some("Main Window".into()),
                    bounds: crate::types::Rect::new(0, 0, frame.width, frame.height),
                    confidence: 1.0,
                    is_interactive: false,
                    is_enabled: true,
                    children: vec![SemanticElement {
                        id: "btn1".into(),
                        element_type: UiElementKind::Button,
                        text: Some("Click Me".into()),
                        bounds: crate::types::Rect::new(10, 10, 80, 30),
                        confidence: 0.95,
                        is_interactive: true,
                        is_enabled: true,
                        children: vec![],
                        parent_id: Some("root".into()),
                        properties: HashMap::new(),
                    }],
                    parent_id: None,
                    properties: HashMap::new(),
                },
                timestamp: chrono::Utc::now(),
                source: frame.source.clone(),
                processing_time_ms: 5,
            })
        }

        async fn find_elements_by_type(
            &self,
            _frame: &CapturedFrame,
            element_type: &str,
        ) -> Result<Vec<SemanticElement>> {
            Ok(vec![SemanticElement {
                id: "found".into(),
                element_type: if element_type == "Button" {
                    UiElementKind::Button
                } else {
                    UiElementKind::Generic(element_type.into())
                },
                text: None,
                bounds: crate::types::Rect::new(0, 0, 10, 10),
                confidence: 0.8,
                is_interactive: true,
                is_enabled: true,
                children: vec![],
                parent_id: None,
                properties: HashMap::new(),
            }])
        }

        async fn find_text_in_image(
            &self,
            _frame: &CapturedFrame,
            text: &str,
        ) -> Result<Vec<SemanticElement>> {
            Ok(vec![SemanticElement {
                id: "text-found".into(),
                element_type: UiElementKind::Label,
                text: Some(text.into()),
                bounds: crate::types::Rect::new(10, 10, 50, 20),
                confidence: 0.9,
                is_interactive: false,
                is_enabled: true,
                children: vec![],
                parent_id: None,
                properties: HashMap::new(),
            }])
        }

        fn name(&self) -> &str {
            "test-ui"
        }
    }

    fn make_test_frame() -> CapturedFrame {
        CapturedFrame {
            id: "ui-test".into(),
            data: vec![0u8; 100 * 100 * 4],
            width: 100,
            height: 100,
            stride: 400,
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
    async fn test_ui_analyzer_no_provider() {
        let analyzer = UiAnalyzer::new();
        let frame = make_test_frame();
        let hierarchy = analyzer.analyze(&frame).await.unwrap();
        assert_eq!(hierarchy.root.element_type, UiElementKind::Window);
    }

    #[tokio::test]
    async fn test_ui_analyzer_with_provider() {
        let provider = std::sync::Arc::new(TestUiProvider);
        let analyzer = UiAnalyzer::new().with_provider(provider);
        let frame = make_test_frame();
        let hierarchy = analyzer.analyze(&frame).await.unwrap();
        assert_eq!(hierarchy.root.children.len(), 1);
        assert_eq!(
            hierarchy.root.children[0].element_type,
            UiElementKind::Button
        );
    }

    #[tokio::test]
    async fn test_find_elements_by_type() {
        let provider = std::sync::Arc::new(TestUiProvider);
        let analyzer = UiAnalyzer::new().with_provider(provider);
        let frame = make_test_frame();
        let elements = analyzer
            .find_elements_by_type(&frame, &UiElementKind::Button)
            .await
            .unwrap();
        assert_eq!(elements.len(), 1);
    }

    #[test]
    fn test_classify_element() {
        let analyzer = UiAnalyzer::new();
        let bounds = crate::types::Rect::new(0, 0, 10, 10);
        assert_eq!(
            analyzer.classify_element("button", "OK", bounds),
            UiElementKind::Button
        );
        assert_eq!(
            analyzer.classify_element("edit", "Search", bounds),
            UiElementKind::TextBox
        );
        assert_eq!(
            analyzer.classify_element("checkbox", "Option", bounds),
            UiElementKind::CheckBox
        );
        assert_eq!(
            analyzer.classify_element("unknown", "notification area", bounds),
            UiElementKind::Notification
        );
    }
}
