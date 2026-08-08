use std::fmt;

#[derive(Debug)]
pub enum VisionError {
    CaptureFailed(String),
    OcrFailed(String),
    AnalysisFailed(String),
    ProviderError(String),
    UiAnalysisFailed(String),
    GroundingFailed(String),
    ChangeDetectionFailed(String),
    CoordinateMappingFailed(String),
    ElementNotFound(String),
    Timeout(String),
    UnsupportedOperation(String),
    InsufficientConfidence { actual: f32, required: f32 },
    InvalidParameter(String),
    PipelineBusy,
    ResourceExhausted(String),
    IntegrationError(String),
}

impl fmt::Display for VisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CaptureFailed(msg) => write!(f, "Capture failed: {}", msg),
            Self::OcrFailed(msg) => write!(f, "OCR failed: {}", msg),
            Self::AnalysisFailed(msg) => write!(f, "Analysis failed: {}", msg),
            Self::ProviderError(msg) => write!(f, "Provider error: {}", msg),
            Self::UiAnalysisFailed(msg) => write!(f, "UI analysis failed: {}", msg),
            Self::GroundingFailed(msg) => write!(f, "Grounding failed: {}", msg),
            Self::ChangeDetectionFailed(msg) => write!(f, "Change detection failed: {}", msg),
            Self::CoordinateMappingFailed(msg) => write!(f, "Coordinate mapping failed: {}", msg),
            Self::ElementNotFound(msg) => write!(f, "Element not found: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Self::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            Self::InsufficientConfidence { actual, required } => {
                write!(f, "Insufficient confidence: {}/{}", actual, required)
            }
            Self::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            Self::PipelineBusy => write!(f, "Pipeline is busy"),
            Self::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
            Self::IntegrationError(msg) => write!(f, "Integration error: {}", msg),
        }
    }
}

impl std::error::Error for VisionError {}

pub type Result<T> = std::result::Result<T, VisionError>;
