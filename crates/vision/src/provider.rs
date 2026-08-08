use async_trait::async_trait;

use crate::error::Result;
use crate::types::{
    CapturedFrame, DisplayInfo, OcrResult, PixelFormat, Rect, SemanticElement, UiHierarchy,
};

#[async_trait]
pub trait CaptureProvider: Send + Sync {
    async fn capture_full_screen(&self, display_index: u32) -> Result<CapturedFrame>;
    async fn capture_window(&self, window_id: &str) -> Result<CapturedFrame>;
    async fn capture_region(&self, display_index: u32, region: Rect) -> Result<CapturedFrame>;
    async fn list_displays(&self) -> Result<Vec<DisplayInfo>>;
    async fn screen_size(&self, display_index: u32) -> Result<(u32, u32)>;
    fn supported_pixel_formats(&self) -> Vec<PixelFormat>;
    fn name(&self) -> &str;
    fn supports_window_capture(&self) -> bool;
    fn supports_region_capture(&self) -> bool;
}

#[async_trait]
pub trait OcrProvider: Send + Sync {
    async fn recognize(&self, image: &CapturedFrame, language: Option<&str>) -> Result<OcrResult>;
    async fn recognize_region(
        &self,
        image: &CapturedFrame,
        region: Rect,
        language: Option<&str>,
    ) -> Result<OcrResult>;
    fn supported_languages(&self) -> Vec<String>;
    fn name(&self) -> &str;
}

#[async_trait]
pub trait UiAnalysisProvider: Send + Sync {
    async fn extract_hierarchy(&self, image: &CapturedFrame) -> Result<UiHierarchy>;
    async fn find_elements_by_type(
        &self,
        image: &CapturedFrame,
        element_type: &str,
    ) -> Result<Vec<SemanticElement>>;
    async fn find_text_in_image(
        &self,
        image: &CapturedFrame,
        text: &str,
    ) -> Result<Vec<SemanticElement>>;
    fn name(&self) -> &str;
}

#[async_trait]
pub trait VisionAnalysisProvider: Send + Sync {
    async fn analyze(&self, image: &[u8], prompt: Option<&str>) -> Result<String>;
    fn available_models(&self) -> Vec<String>;
    fn name(&self) -> &str;
}
