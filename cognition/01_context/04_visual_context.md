# Visual Context

## Purpose

The Visual Context module provides the COS with understanding of what is on the user's screen. It captures screenshots, performs OCR, detects UI elements, tracks changes, and resolves user intents to visual targets. This module wraps the `vision` crate's capabilities (`CapturedFrame`, `OcrResult`, `SemanticElement`, `UiHierarchy`, `GroundedTarget`, `VisualChange`, `ChangeReport`) and extends them with a managed pipeline, permission boundaries, and vision model routing. It answers: *What is the user looking at? What text is visible? What UI elements are available? Where should I click?*

## Responsibilities

1. **Screen capture**: Capture screenshots of the active display or specific windows
2. **OCR**: Extract text from screenshots
3. **UI element detection**: Detect interactive UI elements (buttons, text fields, menus)
4. **UI hierarchy**: Build a tree of UI elements with properties
5. **Visual grounding**: Resolve user intent ("click the submit button") to a specific screen location
6. **Change detection**: Detect changes between screenshots
7. **Clipboard integration**: Read/write clipboard content
8. **Selected text detection**: Detect text selected by the user
9. **Permission management**: Enforce boundaries on what visual data is accessed
10. **Vision model routing**: Route visual queries to appropriate vision models

## Internal Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                       VISUAL CONTEXT                                 │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    INPUT SOURCES                              │   │
│  │                                                               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐   │   │
│  │  │ Screen  │  │  OCR    │  │  UI     │  │  Clipboard   │   │   │
│  │  │ Capture │  │ Engine  │  │ Detector│  │  Monitor     │   │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬───────┘   │   │
│  │       │            │            │               │            │   │
│  └───────┼────────────┼────────────┼───────────────┼────────────┘   │
│          │            │            │               │                 │
│          ▼            ▼            ▼               ▼                 │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              VisualContextManager                             │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Screen Pipeline                                        │  │   │
│  │  │  Capture → Preprocess → Analyze → Store                │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  OCR Pipeline                                          │  │   │
│  │  │  Capture → OCR → Text Extraction → Index               │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  UI Detection Pipeline                                 │  │   │
│  │  │  Capture → Element Detection → Hierarchy Build         │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Grounding Pipeline                                    │  │   │
│  │  │  Intent → UI Hierarchy → Target Resolution → Action    │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              VisualSnapshot                                   │   │
│  │  Point-in-time view of all visual context                     │   │
│  │  Consumed by: Grounding, Cognition, Automation               │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Visual Signals

```rust
pub struct VisualSignal {
    /// Signal identifier
    pub id: String,
    
    /// Signal type
    pub signal_type: VisualSignalType,
    
    /// Signal timestamp
    pub observed_at: DateTime<Utc>,
    
    /// Signal confidence
    pub confidence: f64,
    
    /// Signal data
    pub data: serde_json::Value,
}

pub enum VisualSignalType {
    /// Screen captured
    ScreenCapture {
        frame: CapturedFrame,
        source: CaptureSource,
    },
    
    /// OCR result
    OcrResult {
        result: OcrResult,
        source_frame_id: String,
    },
    
    /// UI hierarchy detected
    UiHierarchy {
        hierarchy: UiHierarchy,
        source_frame_id: String,
    },
    
    /// UI element detected
    UiElement {
        element: SemanticElement,
        frame_id: String,
    },
    
    /// Visual change detected
    VisualChange {
        change: VisualChange,
        from_frame_id: String,
        to_frame_id: String,
    },
    
    /// Selected text detected
    SelectedText {
        text: String,
        application: String,
        frame_id: String,
    },
    
    /// Clipboard content changed
    ClipboardChange {
        content: String,
        content_type: ClipboardContentType,
    },
    
    /// User intent to interact with screen
    VisualIntent {
        description: String,
        qualifiers: Vec<String>,
    },
}

pub enum CaptureSource {
    /// Full screen capture
    FullScreen,
    
    /// Specific window capture
    Window(String),
    
    /// Specific region capture
    Region(Region),
    
    /// Focused element capture
    FocusedElement,
    
    /// OCR capture (optimized for text)
    OcrCapture,
}

pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub enum ClipboardContentType {
    Text,
    Image,
    RichText,
    File,
    Unknown,
}
```

## Outputs

### Visual Snapshot

```rust
pub struct VisualSnapshot {
    /// Snapshot identifier
    pub id: String,
    
    /// Capture timestamp
    pub captured_at: DateTime<Utc>,
    
    /// Frame identifier
    pub frame_id: String,
    
    /// Capture source
    pub source: CaptureSource,
    
    /// Frame dimensions
    pub width: u32,
    pub height: u32,
    
    /// DPI
    pub dpi: u32,
    
    /// Extracted text (if OCR performed)
    pub ocr_text: Option<String>,
    
    /// OCR results with line/word positions
    pub ocr_results: Vec<OcrResultInfo>,
    
    /// UI hierarchy
    pub ui_hierarchy: Option<UiHierarchyInfo>,
    
    /// Detected UI elements
    pub ui_elements: Vec<UiElementInfo>,
    
    /// Selected text
    pub selected_text: Option<String>,
    
    /// Clipboard content
    pub clipboard: Option<ClipboardInfo>,
    
    /// Recent changes (since last snapshot)
    pub recent_changes: Vec<ChangeInfo>,
    
    /// Visual features extracted
    pub visual_features: Option<VisualFeatures>,
    
    /// Data freshness
    pub freshness: u64,
    
    /// Confidence in snapshot
    pub confidence: f64,
}

pub struct OcrResultInfo {
    /// Full OCR text
    pub full_text: String,
    
    /// Text lines with positions
    pub lines: Vec<TextLine>,
    
    /// Words with positions
    pub words: Vec<TextWord>,
    
    /// OCR confidence
    pub confidence: f64,
    
    /// Language detected
    pub language: Option<String>,
}

pub struct TextLine {
    /// Line text
    pub text: String,
    
    /// Line bounding box
    pub bbox: BoundingBox,
    
    /// Line confidence
    pub confidence: f64,
}

pub struct TextWord {
    /// Word text
    pub text: String,
    
    /// Word bounding box
    pub bbox: BoundingBox,
    
    /// Word confidence
    pub confidence: f64,
}

pub struct BoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub struct UiHierarchyInfo {
    /// Root element
    pub root: UiElementInfo,
    
    /// Total element count
    pub element_count: u32,
    
    /// Interactive element count
    pub interactive_count: u32,
    
    /// Hierarchy depth
    pub depth: u32,
    
    /// Capture timestamp
    pub captured_at: DateTime<Utc>,
}

pub struct UiElementInfo {
    /// Element identifier
    pub id: String,
    
    /// Element type
    pub element_type: UiElementType,
    
    /// Element role (accessibility role)
    pub role: Option<String>,
    
    /// Element name/title
    pub name: Option<String>,
    
    /// Element value
    pub value: Option<String>,
    
    /// Element description
    pub description: Option<String>,
    
    /// Element bounding box
    pub bbox: BoundingBox,
    
    /// Is enabled
    pub enabled: bool,
    
    /// Is visible
    pub visible: bool,
    
    /// Is focused
    pub focused: bool,
    
    /// Is selected
    pub selected: bool,
    
    /// Children
    pub children: Vec<UiElementInfo>,
}

pub enum UiElementType {
    Button,
    TextField,
    TextArea,
    CheckBox,
    RadioButton,
    ComboBox,
    ListBox,
    Menu,
    MenuItem,
    Tab,
    Link,
    Image,
    Text,
    Container,
    Window,
    Dialog,
    Toolbar,
    StatusBar,
    ScrollBar,
    ProgressBar,
    Slider,
    Other(String),
}

pub struct ChangeInfo {
    /// Change type
    pub change_type: ChangeType,
    
    /// Change region
    pub region: BoundingBox,
    
    /// Change description
    pub description: String,
    
    /// Change confidence
    pub confidence: f64,
    
    /// Change timestamp
    pub timestamp: DateTime<Utc>,
}

pub enum ChangeType {
    TextAppeared,
    TextDisappeared,
    TextChanged,
    ElementAppeared,
    ElementDisappeared,
    ElementStateChanged,
    LayoutChanged,
    ColorChanged,
    ContentChanged,
}

pub struct ClipboardInfo {
    /// Clipboard content
    pub content: String,
    
    /// Content type
    pub content_type: ClipboardContentType,
    
    /// Content length
    pub length: usize,
    
    /// Last changed timestamp
    pub changed_at: DateTime<Utc>,
}

pub struct VisualFeatures {
    /// Dominant colors
    pub dominant_colors: Vec<ColorInfo>,
    
    /// Brightness level
    pub brightness: f64,
    
    /// Complexity score
    pub complexity: f64,
    
    /// Has text content
    pub has_text: bool,
    
    /// Has images
    pub has_images: bool,
    
    /// Has video
    pub has_video: bool,
    
    /// Has code
    pub has_code: bool,
}

pub struct ColorInfo {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub percentage: f64,
}
```

## State Transitions

```
┌─────────────────────────────────────────────────────────────────────┐
│                  VISUAL CONTEXT STATE MACHINE                        │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │   INITIALIZING   │                                               │
│  └────────┬─────────┘                                               │
│           │ (capture permissions granted)                            │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   MONITORING     │◀─────────────────────────────────────┐        │
│  └────────┬─────────┘                                       │        │
│           │ (capture requested)                             │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   CAPTURING      │                                       │        │
│  └────────┬─────────┘                                       │        │
│           │ (capture complete)                              │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   PROCESSING     │                                       │        │
│  │   (OCR + UI)     │                                       │        │
│  └────────┬─────────┘                                       │        │
│           │ (processing complete)                           │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   ANALYZING      │                                       │        │
│  │   (changes,      │                                       │        │
│  │    features)     │                                       │        │
│  └────────┬─────────┘                                       │        │
│           │ (analysis complete)                             │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   PUBLISHING     │──────────────────────────────────────┘        │
│  └──────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Decision Logic

### When to Capture

```rust
fn should_capture(
    signal: &VisualSignal,
    current_snapshot: &Option<VisualSnapshot>,
    config: &VisualConfig,
) -> bool {
    // Always capture on explicit request
    if matches!(signal.signal_type, VisualSignalType::VisualIntent { .. }) {
        return true;
    }
    
    // Capture if no current snapshot
    if current_snapshot.is_none() {
        return true;
    }
    
    // Capture if snapshot is stale
    if let Some(snapshot) = current_snapshot {
        let age = Utc::now() - snapshot.captured_at;
        if age > Duration::from_secs(config.max_staleness_seconds) {
            return true;
        }
    }
    
    // Capture on window change
    if matches!(signal.signal_type, VisualSignalType::ScreenCapture { .. }) {
        return true;
    }
    
    false
}
```

### When to Perform OCR

```rust
fn should_perform_ocr(
    frame: &CapturedFrame,
    config: &VisualConfig,
) -> bool {
    // Always OCR if explicitly requested
    if config.force_ocr {
        return true;
    }
    
    // OCR if frame has text-like characteristics
    if has_text_characteristics(frame) {
        return true;
    }
    
    // OCR if application is text-heavy
    if is_text_heavy_application(frame) {
        return true;
    }
    
    false
}
```

### Vision Model Routing

```rust
fn route_vision_query(
    query: &VisualQuery,
    available_models: &[VisionModel],
) -> VisionModel {
    match query.query_type {
        VisualQueryType::Ocr => {
            // Use dedicated OCR model
            available_models.iter()
                .find(|m| m.supports_ocr)
                .unwrap_or(&available_models[0])
                .clone()
        }
        VisualQueryType::ElementDetection => {
            // Use UI element detection model
            available_models.iter()
                .find(|m| m.supports_ui_detection)
                .unwrap_or(&available_models[0])
                .clone()
        }
        VisualQueryType::SceneUnderstanding => {
            // Use general vision model
            available_models.iter()
                .find(|m| m.supports_scene_understanding)
                .unwrap_or(&available_models[0])
                .clone()
        }
        VisualQueryType::Grounding => {
            // Use grounding model
            available_models.iter()
                .find(|m| m.supports_grounding)
                .unwrap_or(&available_models[0])
                .clone()
        }
    }
}
```

## Failure Modes

| Mode | Detection | Recovery | Prevention |
|------|-----------|----------|------------|
| Capture permission denied | Permission error | Inform user, request permission | Request permissions at startup |
| OCR failure | Low confidence, empty result | Retry with different model, use cached | Fallback OCR models |
| UI detection failure | Empty hierarchy | Use OCR-only mode | Multiple detection backends |
| Stale snapshot | Freshness threshold exceeded | Capture new frame | Adaptive capture frequency |
| Large frame memory | Memory pressure | Reduce resolution, skip full-frame | Resolution limits |
| Vision model timeout | Timeout error | Use fallback model, return partial | Model timeout configuration |

### Recovery Strategy

```rust
impl VisualContextManager {
    async fn recover_from_capture_failure(&self, error: &CaptureError) -> Option<VisualSnapshot> {
        match error {
            CaptureError::PermissionDenied => {
                tracing::warn!("Screen capture permission denied");
                // Inform user, don't retry
                None
            }
            CaptureError::Timeout => {
                tracing::warn!("Screen capture timed out, retrying");
                self.retry_capture().await
            }
            CaptureError::DeviceBusy => {
                tracing::warn!("Capture device busy, waiting and retrying");
                tokio::time::sleep(Duration::from_millis(500)).await;
                self.retry_capture().await
            }
            CaptureError::OutOfMemory => {
                tracing::warn!("Out of memory during capture, reducing resolution");
                self.capture_with_reduced_resolution().await
            }
        }
    }
}
```

## Privacy Considerations

1. **Screen capture**: Screenshots are processed locally, never transmitted. Users must explicitly grant screen capture permission.
2. **OCR text**: Extracted text is processed in-memory, not stored persistently unless user requests.
3. **UI elements**: UI element information is used only for grounding and interaction. Not logged.
4. **Clipboard**: Clipboard content is read only when explicitly requested by user action. Not stored.
5. **Selected text**: Selected text is read only when user initiates an action. Not stored.
6. **Visual features**: Visual features are used only for context. Not associated with user identity.
7. **Permission boundaries**: Visual context is only captured when user is actively interacting with VOXY.
8. **No screenshots stored**: Screenshots are processed and immediately discarded. Only extracted metadata is retained.

## Security Considerations

1. **Permission model**: Screen capture requires explicit OS-level permission. Permission is checked before every capture.
2. **Secure processing**: Visual data is processed in secure memory, not written to disk.
3. **No remote transmission**: Visual data never leaves the device without explicit user consent.
4. **Integrity verification**: Captured frames are verified for integrity before processing.
5. **Access control**: Only authorized COS components can access visual context.
6. **Audit logging**: Visual context access is auditable.

## Future Extensibility

1. **Multi-monitor support**: Simultaneous capture from multiple displays
2. **Video capture**: Real-time video understanding for streaming content
3. **AR/VR support**: Context from augmented/virtual reality displays
4. **Accessibility mode**: Enhanced OCR for accessibility features
5. **Multi-language OCR**: Support for additional languages
6. **Handwriting recognition**: OCR for handwritten text
7. **Diagram understanding**: Understand flowcharts, diagrams, charts
8. **Code understanding**: Visual understanding of code structure

## Examples

### Example 1: OCR on Code Editor

```
Capture: FullScreen, VSCode window active
OCR Result: "fn main() { println!(\"Hello, world!\"); }"
UI Elements: [Button("Run"), Button("Debug"), TextField("search")]
Visual Features: has_code=true, has_text=true, brightness=0.3
VisualSnapshot used by: Grounding → "Run the code" → click Button("Run")
```

### Example 2: Change Detection

```
Frame 1: Button("Submit") disabled, form fields empty
Frame 2: Button("Submit") enabled, form fields filled
Changes: [ElementStateChanged(Button("Submit"), disabled→enabled)]
Interpretation: User completed a form
```

### Example 3: Visual Grounding

```
User Intent: "Click the blue button at the top right"
UI Hierarchy: [Button("Save", bbox=(1200,50,100,30), color=blue)]
Grounding Result: ResolvedTarget::Region(bbox=(1200,50,100,30))
Action: Click at coordinates (1250, 65)
```

## Engineering Notes

- Screen capture uses platform-specific APIs (Windows: DXGI Desktop Duplication, Linux: X11/Wayland, macOS: Core Graphics)
- OCR uses the `vision` crate's built-in OCR engine with Tesseract fallback
- UI detection uses accessibility APIs (Windows: UI Automation, Linux: AT-SPI, macOS: Accessibility)
- Visual grounding uses the `grounding` crate to resolve intents to screen targets
- Frame processing is async and parallelized (capture + OCR + UI detection)
- Frames are stored in a ring buffer (default: 5 frames) to minimize memory usage
- Change detection uses frame differencing with configurable sensitivity
- Clipboard monitoring uses OS clipboard APIs with polling fallback
- Vision model routing is configurable per-query-type
- All timestamps use `chrono::DateTime<Utc>` for consistency
