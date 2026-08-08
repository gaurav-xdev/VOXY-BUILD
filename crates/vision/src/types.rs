pub use voxy_shared::types::Rect;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PixelFormat {
    Rgba,
    Bgra,
    Rgb,
    Bgr,
    Grayscale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSource {
    pub kind: CaptureSourceKind,
    pub display_index: Option<u32>,
    pub window_id: Option<String>,
    pub region: Option<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaptureSourceKind {
    FullScreen,
    Window,
    Region,
    VirtualScreen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedFrame {
    pub id: String,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
    pub source: CaptureSource,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub dpi_scale: f64,
    pub monitor_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub index: u32,
    pub bounds: Rect,
    pub dpi_scale: f64,
    pub is_primary: bool,
    pub name: String,
    pub is_attached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrChar {
    pub char_value: char,
    pub confidence: f32,
    pub bounding_box: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrWord {
    pub text: String,
    pub confidence: f32,
    pub bounding_box: Rect,
    pub chars: Vec<OcrChar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrLine {
    pub text: String,
    pub confidence: f32,
    pub bounding_box: Rect,
    pub words: Vec<OcrWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub lines: Vec<OcrLine>,
    pub average_confidence: f32,
    pub language: Option<String>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UiElementKind {
    Button,
    TextBox,
    Menu,
    Dialog,
    Icon,
    Toolbar,
    Tab,
    TabItem,
    List,
    ListItem,
    Tree,
    TreeNode,
    StatusBar,
    Notification,
    Label,
    CheckBox,
    RadioButton,
    ComboBox,
    DropDown,
    ScrollBar,
    Slider,
    ProgressBar,
    Image,
    Link,
    Window,
    Pane,
    Group,
    Table,
    TableCell,
    Header,
    Footer,
    Separator,
    Spinner,
    Tooltip,
    PopupMenu,
    Generic(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticElement {
    pub id: String,
    pub element_type: UiElementKind,
    pub text: Option<String>,
    pub bounds: Rect,
    pub confidence: f32,
    pub is_interactive: bool,
    pub is_enabled: bool,
    pub children: Vec<SemanticElement>,
    pub parent_id: Option<String>,
    pub properties: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHierarchy {
    pub root: SemanticElement,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: CaptureSource,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroundingMethod {
    AccessibilityIdentifier(String),
    VisibleText {
        text: String,
        confidence: f32,
    },
    IconSimilarity {
        template_id: String,
        similarity: f32,
    },
    WindowHierarchy {
        path: Vec<String>,
    },
    SpatialRelationship {
        reference_id: String,
        offset_x: i32,
        offset_y: i32,
    },
    CoordinateSnap {
        x: i32,
        y: i32,
        tolerance: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedTarget {
    pub element: SemanticElement,
    pub overall_confidence: f32,
    pub methods: Vec<GroundingMethod>,
    pub stable_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_verified: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeKind {
    WindowOpened,
    WindowClosed,
    WindowMoved,
    WindowResized,
    ElementAppeared,
    ElementDisappeared,
    ElementMoved,
    TextChanged,
    VisualChanged,
    ApplicationStateChanged,
    ContentAppeared,
    ContentDisappeared,
    RegionUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualChange {
    pub id: String,
    pub change_type: ChangeKind,
    pub region: Rect,
    pub confidence: f32,
    pub description: String,
    pub before_id: Option<String>,
    pub after_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeReport {
    pub changes: Vec<VisualChange>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub frame_before_id: String,
    pub frame_after_id: String,
    pub processing_time_ms: u64,
    pub has_significant_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinateMapping {
    pub source_rect: Rect,
    pub target_rect: Rect,
    pub source_dpi: f64,
    pub target_dpi: f64,
}

pub const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.6;
pub const MIN_OCR_CONFIDENCE: f32 = 0.3;
pub const MIN_GROUNDING_CONFIDENCE: f32 = 0.5;
