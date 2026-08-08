use parking_lot::RwLock;
use std::collections::HashMap;
use tracing::info;

use crate::error::Result;
use crate::types::{
    CapturedFrame, ChangeKind, ChangeReport, SemanticElement, UiHierarchy, VisualChange,
};

use crate::capture::compute_frame_diff;

struct ChangeState {
    previous_frames: HashMap<u32, CapturedFrame>,
    previous_hierarchies: HashMap<String, UiHierarchy>,
}

pub struct ChangeDetector {
    state: RwLock<ChangeState>,
    pixel_threshold: u8,
    min_change_area: u32,
    max_cached_frames: usize,
}

impl ChangeDetector {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(ChangeState {
                previous_frames: HashMap::new(),
                previous_hierarchies: HashMap::new(),
            }),
            pixel_threshold: 20,
            min_change_area: 16,
            max_cached_frames: 20,
        }
    }

    pub fn with_pixel_threshold(mut self, threshold: u8) -> Self {
        self.pixel_threshold = threshold;
        self
    }

    pub fn with_min_change_area(mut self, area: u32) -> Self {
        self.min_change_area = area;
        self
    }

    pub async fn detect_frame_changes(&self, current: &CapturedFrame) -> Result<ChangeReport> {
        let start = std::time::Instant::now();
        let mut changes = Vec::new();
        let before_id;
        let monitor = current.monitor_index;

        {
            let state = self.state.read();
            before_id = state.previous_frames.get(&monitor).map(|f| f.id.clone());
        }

        if let Some(ref prev_id) = before_id {
            if let Some(previous) = self.get_previous_frame(monitor) {
                let diff_regions = compute_frame_diff(&previous, current, self.pixel_threshold);

                for region in &diff_regions {
                    let area = region.width() * region.height();
                    if area >= self.min_change_area {
                        changes.push(VisualChange {
                            id: uuid::Uuid::new_v4().to_string(),
                            change_type: ChangeKind::RegionUpdated,
                            region: *region,
                            confidence: 0.8,
                            description: format!(
                                "Region updated: {}x{} at ({}, {})",
                                region.width(),
                                region.height(),
                                region.x(),
                                region.y()
                            ),
                            before_id: Some(prev_id.clone()),
                            after_id: Some(current.id.clone()),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            }
        }

        self.store_frame(current);

        let has_significant = !changes.is_empty();
        Ok(ChangeReport {
            changes,
            timestamp: chrono::Utc::now(),
            frame_before_id: before_id.unwrap_or_default(),
            frame_after_id: current.id.clone(),
            processing_time_ms: start.elapsed().as_millis() as u64,
            has_significant_changes: has_significant,
        })
    }

    pub fn detect_hierarchy_changes(
        &self,
        source_key: &str,
        current: &UiHierarchy,
    ) -> Result<Vec<VisualChange>> {
        let mut changes = Vec::new();
        let previous = {
            let state = self.state.read();
            state.previous_hierarchies.get(source_key).cloned()
        };

        if let Some(ref prev) = previous {
            Self::diff_hierarchies(prev, current, &mut changes);
        }

        {
            let mut state = self.state.write();
            state
                .previous_hierarchies
                .insert(source_key.to_string(), current.clone());
            while state.previous_hierarchies.len() > self.max_cached_frames {
                let oldest = state
                    .previous_hierarchies
                    .keys()
                    .next()
                    .unwrap_or(&String::new())
                    .clone();
                state.previous_hierarchies.remove(&oldest);
            }
        }

        Ok(changes)
    }

    pub fn has_previous_frame(&self, monitor_index: u32) -> bool {
        self.state
            .read()
            .previous_frames
            .contains_key(&monitor_index)
    }

    pub fn get_previous_frame(&self, monitor_index: u32) -> Option<CapturedFrame> {
        self.state
            .read()
            .previous_frames
            .get(&monitor_index)
            .cloned()
    }

    pub fn clear(&self) {
        let mut state = self.state.write();
        state.previous_frames.clear();
        state.previous_hierarchies.clear();
        info!("Change detector state cleared");
    }

    fn store_frame(&self, frame: &CapturedFrame) {
        let mut state = self.state.write();
        state
            .previous_frames
            .insert(frame.monitor_index, frame.clone());
        while state.previous_frames.len() > self.max_cached_frames {
            let oldest = *state.previous_frames.keys().next().unwrap_or(&0);
            state.previous_frames.remove(&oldest);
        }
    }

    fn diff_hierarchies(
        prev: &UiHierarchy,
        current: &UiHierarchy,
        changes: &mut Vec<VisualChange>,
    ) {
        Self::diff_elements(&prev.root, &current.root, String::new(), changes);
    }

    fn diff_elements(
        prev: &SemanticElement,
        current: &SemanticElement,
        parent_path: String,
        changes: &mut Vec<VisualChange>,
    ) {
        if prev.bounds != current.bounds {
            changes.push(VisualChange {
                id: uuid::Uuid::new_v4().to_string(),
                change_type: ChangeKind::ElementMoved,
                region: current.bounds,
                confidence: 0.85,
                description: format!(
                    "Element '{}' moved from {:?} to {:?}",
                    prev.text.as_deref().unwrap_or(&prev.id),
                    prev.bounds,
                    current.bounds
                ),
                before_id: Some(prev.id.clone()),
                after_id: Some(current.id.clone()),
                timestamp: chrono::Utc::now(),
            });
        }

        if prev.text != current.text {
            changes.push(VisualChange {
                id: uuid::Uuid::new_v4().to_string(),
                change_type: ChangeKind::TextChanged,
                region: current.bounds,
                confidence: 0.9,
                description: format!(
                    "Text changed from '{}' to '{}'",
                    prev.text.as_deref().unwrap_or(""),
                    current.text.as_deref().unwrap_or("")
                ),
                before_id: Some(prev.id.clone()),
                after_id: Some(current.id.clone()),
                timestamp: chrono::Utc::now(),
            });
        }

        let prev_ids: std::collections::HashSet<&str> =
            prev.children.iter().map(|c| c.id.as_str()).collect();
        let curr_ids: std::collections::HashSet<&str> =
            current.children.iter().map(|c| c.id.as_str()).collect();

        for child in &current.children {
            if !prev_ids.contains(child.id.as_str()) {
                changes.push(VisualChange {
                    id: uuid::Uuid::new_v4().to_string(),
                    change_type: ChangeKind::ElementAppeared,
                    region: child.bounds,
                    confidence: 0.8,
                    description: format!(
                        "New element '{}' appeared",
                        child.text.as_deref().unwrap_or(&child.id)
                    ),
                    before_id: None,
                    after_id: Some(child.id.clone()),
                    timestamp: chrono::Utc::now(),
                });
            }
        }

        for child in &prev.children {
            if !curr_ids.contains(child.id.as_str()) {
                changes.push(VisualChange {
                    id: uuid::Uuid::new_v4().to_string(),
                    change_type: ChangeKind::ElementDisappeared,
                    region: child.bounds,
                    confidence: 0.8,
                    description: format!(
                        "Element '{}' disappeared",
                        child.text.as_deref().unwrap_or(&child.id)
                    ),
                    before_id: Some(child.id.clone()),
                    after_id: None,
                    timestamp: chrono::Utc::now(),
                });
            }
        }

        let path_prefix = if parent_path.is_empty() {
            current.id.clone()
        } else {
            format!("{}/{}", parent_path, current.id)
        };

        for prev_child in &prev.children {
            if let Some(curr_child) = current.children.iter().find(|c| c.id == prev_child.id) {
                Self::diff_elements(prev_child, curr_child, path_prefix.clone(), changes);
            }
        }
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CaptureSource, CaptureSourceKind, PixelFormat, Rect, UiElementKind};

    fn make_test_frame(id: &str, fill_byte: u8) -> CapturedFrame {
        CapturedFrame {
            id: id.into(),
            data: vec![fill_byte; 100 * 100 * 4],
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
    async fn test_no_changes_on_first_frame() {
        let detector = ChangeDetector::new();
        let frame = make_test_frame("first", 128);
        let report = detector.detect_frame_changes(&frame).await.unwrap();
        assert!(!report.has_significant_changes);
    }

    #[tokio::test]
    async fn test_detects_pixel_changes() {
        let detector = ChangeDetector::new();
        let frame1 = make_test_frame("first", 128);
        detector.detect_frame_changes(&frame1).await.unwrap();

        let mut frame2 = make_test_frame("second", 128);
        frame2.data[0] = 255;
        frame2.data[1] = 0;
        frame2.data[2] = 0;
        let report = detector.detect_frame_changes(&frame2).await.unwrap();
        assert!(report.has_significant_changes);
    }

    #[test]
    fn test_hierarchy_no_changes() {
        let detector = ChangeDetector::new();
        let hierarchy = UiHierarchy {
            root: SemanticElement {
                id: "root".into(),
                element_type: UiElementKind::Window,
                text: None,
                bounds: Rect::new(0, 0, 100, 100),
                confidence: 1.0,
                is_interactive: false,
                is_enabled: true,
                children: vec![],
                parent_id: None,
                properties: std::collections::HashMap::new(),
            },
            timestamp: chrono::Utc::now(),
            source: CaptureSource {
                kind: CaptureSourceKind::FullScreen,
                display_index: Some(0),
                window_id: None,
                region: None,
            },
            processing_time_ms: 0,
        };
        detector
            .detect_hierarchy_changes("test", &hierarchy)
            .unwrap();
        let result = detector
            .detect_hierarchy_changes("test", &hierarchy)
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_detects_element_appeared() {
        let detector = ChangeDetector::new();
        let prev = UiHierarchy {
            root: SemanticElement {
                id: "root".into(),
                element_type: UiElementKind::Window,
                text: None,
                bounds: Rect::new(0, 0, 100, 100),
                confidence: 1.0,
                is_interactive: false,
                is_enabled: true,
                children: vec![],
                parent_id: None,
                properties: std::collections::HashMap::new(),
            },
            timestamp: chrono::Utc::now(),
            source: CaptureSource {
                kind: CaptureSourceKind::FullScreen,
                display_index: Some(0),
                window_id: None,
                region: None,
            },
            processing_time_ms: 0,
        };
        detector.detect_hierarchy_changes("test", &prev).unwrap();

        let current = UiHierarchy {
            root: SemanticElement {
                children: vec![SemanticElement {
                    id: "new-btn".into(),
                    element_type: UiElementKind::Button,
                    text: Some("New".into()),
                    bounds: Rect::new(10, 10, 50, 20),
                    confidence: 0.9,
                    is_interactive: true,
                    is_enabled: true,
                    children: vec![],
                    parent_id: Some("root".into()),
                    properties: std::collections::HashMap::new(),
                }],
                ..prev.root.clone()
            },
            ..prev.clone()
        };
        let changes = detector.detect_hierarchy_changes("test", &current).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeKind::ElementAppeared);
    }
}
