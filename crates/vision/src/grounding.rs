use parking_lot::RwLock;
use std::collections::HashMap;
use tracing::info;

use crate::error::{Result, VisionError};
use crate::types::{
    GroundedTarget, GroundingMethod, SemanticElement, UiHierarchy, MIN_GROUNDING_CONFIDENCE,
};

struct GroundingState {
    stable_targets: HashMap<String, GroundedTarget>,
}

pub struct GroundingEngine {
    state: RwLock<GroundingState>,
    min_confidence: f32,
    max_targets: usize,
}

impl GroundingEngine {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(GroundingState {
                stable_targets: HashMap::new(),
            }),
            min_confidence: MIN_GROUNDING_CONFIDENCE,
            max_targets: 1000,
        }
    }

    pub fn with_min_confidence(mut self, conf: f32) -> Self {
        self.min_confidence = conf;
        self
    }

    pub fn with_max_targets(mut self, max: usize) -> Self {
        self.max_targets = max;
        self
    }

    pub fn ground_by_accessibility_id(
        &self,
        hierarchy: &UiHierarchy,
        accessibility_id: &str,
    ) -> Result<GroundedTarget> {
        let element = Self::find_by_accessibility_id(&hierarchy.root, accessibility_id)
            .ok_or_else(|| {
                VisionError::ElementNotFound(format!(
                    "element with accessibility id '{}' not found",
                    accessibility_id
                ))
            })?;

        let stable_id = format!("acc-{}", accessibility_id);
        let target = GroundedTarget {
            element: element.clone(),
            overall_confidence: element.confidence,
            methods: vec![GroundingMethod::AccessibilityIdentifier(
                accessibility_id.to_string(),
            )],
            stable_id: stable_id.clone(),
            created_at: chrono::Utc::now(),
            last_verified: Some(chrono::Utc::now()),
        };

        self.cache_target(stable_id, &target);
        Ok(target)
    }

    pub fn ground_by_text(
        &self,
        hierarchy: &UiHierarchy,
        text: &str,
    ) -> Result<Vec<GroundedTarget>> {
        let lower_text = text.to_lowercase();
        let mut results = Vec::new();

        Self::find_by_text_recursive(&hierarchy.root, &lower_text, &mut results);

        if results.is_empty() {
            return Err(VisionError::ElementNotFound(format!(
                "element with text '{}' not found",
                text
            )));
        }

        Ok(results
            .into_iter()
            .map(|element| {
                let stable_id = format!("txt-{}", element.id);
                let conf = element.confidence;
                GroundedTarget {
                    element,
                    overall_confidence: conf,
                    methods: vec![GroundingMethod::VisibleText {
                        text: text.to_string(),
                        confidence: conf,
                    }],
                    stable_id,
                    created_at: chrono::Utc::now(),
                    last_verified: Some(chrono::Utc::now()),
                }
            })
            .collect())
    }

    pub fn ground_by_spatial(
        &self,
        hierarchy: &UiHierarchy,
        reference_id: &str,
        offset_x: i32,
        offset_y: i32,
    ) -> Result<GroundedTarget> {
        let reference = Self::find_by_id(&hierarchy.root, reference_id).ok_or_else(|| {
            VisionError::ElementNotFound(format!(
                "reference element '{}' not found for spatial grounding",
                reference_id
            ))
        })?;

        let target_x = reference.bounds.x() + offset_x;
        let target_y = reference.bounds.y() + offset_y;

        let target = Self::find_by_containing_point(&hierarchy.root, target_x, target_y)
            .ok_or_else(|| {
                VisionError::ElementNotFound(format!(
                    "no element found at ({}, {}) relative to '{}'",
                    offset_x, offset_y, reference_id
                ))
            })?;

        let stable_id = format!("spatial-{}", target.id);
        let result = GroundedTarget {
            element: target.clone(),
            overall_confidence: target.confidence,
            methods: vec![GroundingMethod::SpatialRelationship {
                reference_id: reference_id.to_string(),
                offset_x,
                offset_y,
            }],
            stable_id,
            created_at: chrono::Utc::now(),
            last_verified: Some(chrono::Utc::now()),
        };

        Ok(result)
    }

    pub fn ground_by_coordinates(
        &self,
        hierarchy: &UiHierarchy,
        x: i32,
        y: i32,
        tolerance: u32,
    ) -> Result<GroundedTarget> {
        let element =
            Self::find_nearest_element(&hierarchy.root, x, y, tolerance).ok_or_else(|| {
                VisionError::ElementNotFound(format!(
                    "no element near ({}, {}) within {}px",
                    x, y, tolerance
                ))
            })?;

        let stable_id = format!("coord-{}", element.id);
        let result = GroundedTarget {
            element: element.clone(),
            overall_confidence: element.confidence,
            methods: vec![GroundingMethod::CoordinateSnap { x, y, tolerance }],
            stable_id,
            created_at: chrono::Utc::now(),
            last_verified: Some(chrono::Utc::now()),
        };

        Ok(result)
    }

    pub fn ground_by_hierarchy_path(
        &self,
        hierarchy: &UiHierarchy,
        path: &[String],
    ) -> Result<GroundedTarget> {
        let mut current = &hierarchy.root;

        for segment in path {
            let found = current
                .children
                .iter()
                .find(|c| {
                    c.element_type == crate::types::UiElementKind::Generic(segment.clone())
                        || c.id == *segment
                        || c.text.as_deref() == Some(segment.as_str())
                })
                .ok_or_else(|| {
                    VisionError::ElementNotFound(format!(
                        "path segment '{}' not found in hierarchy",
                        segment
                    ))
                })?;
            current = found;
        }

        let stable_id = format!("path-{}", current.id);
        let result = GroundedTarget {
            element: current.clone(),
            overall_confidence: current.confidence,
            methods: vec![GroundingMethod::WindowHierarchy {
                path: path.to_vec(),
            }],
            stable_id,
            created_at: chrono::Utc::now(),
            last_verified: Some(chrono::Utc::now()),
        };

        Ok(result)
    }

    pub fn combine_methods(&self, targets: Vec<GroundedTarget>) -> Result<GroundedTarget> {
        if targets.is_empty() {
            return Err(VisionError::GroundingFailed("no targets to combine".into()));
        }
        if targets.len() == 1 {
            return Ok(targets.into_iter().next().unwrap());
        }

        let best = targets
            .iter()
            .max_by(|a, b| {
                a.overall_confidence
                    .partial_cmp(&b.overall_confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| {
                VisionError::GroundingFailed("no valid target after combining".into())
            })?;

        let all_methods: Vec<GroundingMethod> =
            targets.iter().flat_map(|t| t.methods.clone()).collect();
        let avg_confidence: f32 =
            targets.iter().map(|t| t.overall_confidence).sum::<f32>() / targets.len() as f32;

        Ok(GroundedTarget {
            element: best.element.clone(),
            overall_confidence: avg_confidence,
            methods: all_methods,
            stable_id: format!("combined-{}", best.stable_id),
            created_at: chrono::Utc::now(),
            last_verified: Some(chrono::Utc::now()),
        })
    }

    pub fn get_stable_target(&self, stable_id: &str) -> Option<GroundedTarget> {
        self.state.read().stable_targets.get(stable_id).cloned()
    }

    pub fn update_target(&self, target: GroundedTarget) {
        let mut state = self.state.write();
        state
            .stable_targets
            .insert(target.stable_id.clone(), target);
        while state.stable_targets.len() > self.max_targets {
            let oldest_id = state
                .stable_targets
                .iter()
                .min_by_key(|(_, t)| t.created_at)
                .map(|(k, _)| k.clone());
            if let Some(id) = oldest_id {
                state.stable_targets.remove(&id);
            }
        }
    }

    pub fn verify_target(&self, hierarchy: &UiHierarchy, target: &GroundedTarget) -> Result<bool> {
        let found = match target.methods.first() {
            Some(GroundingMethod::AccessibilityIdentifier(id)) => {
                Self::find_by_accessibility_id(&hierarchy.root, id).is_some()
            }
            Some(GroundingMethod::VisibleText { text, .. }) => {
                let mut results = Vec::new();
                Self::find_by_text_recursive(&hierarchy.root, &text.to_lowercase(), &mut results);
                !results.is_empty()
            }
            Some(GroundingMethod::CoordinateSnap { x, y, tolerance }) => {
                Self::find_nearest_element(&hierarchy.root, *x, *y, *tolerance).is_some()
            }
            _ => false,
        };

        if found {
            if let Some(state_target) = self.state.write().stable_targets.get_mut(&target.stable_id)
            {
                state_target.last_verified = Some(chrono::Utc::now());
            }
        }

        Ok(found)
    }

    pub fn clear_cache(&self) {
        self.state.write().stable_targets.clear();
        info!("Grounding cache cleared");
    }

    fn cache_target(&self, stable_id: String, target: &GroundedTarget) {
        let mut state = self.state.write();
        state.stable_targets.insert(stable_id, target.clone());
    }

    fn find_by_accessibility_id<'a>(
        element: &'a SemanticElement,
        acc_id: &str,
    ) -> Option<&'a SemanticElement> {
        if element
            .properties
            .get("automation_id")
            .is_some_and(|v| v == acc_id)
        {
            return Some(element);
        }
        for child in &element.children {
            if let Some(found) = Self::find_by_accessibility_id(child, acc_id) {
                return Some(found);
            }
        }
        None
    }

    fn find_by_id<'a>(element: &'a SemanticElement, id: &str) -> Option<&'a SemanticElement> {
        if element.id == id {
            return Some(element);
        }
        for child in &element.children {
            if let Some(found) = Self::find_by_id(child, id) {
                return Some(found);
            }
        }
        None
    }

    fn find_by_text_recursive(
        element: &SemanticElement,
        lower_text: &str,
        results: &mut Vec<SemanticElement>,
    ) {
        if let Some(ref text) = element.text {
            if text.to_lowercase().contains(lower_text) {
                results.push(element.clone());
            }
        }
        for child in &element.children {
            Self::find_by_text_recursive(child, lower_text, results);
        }
    }

    fn find_by_containing_point(
        element: &SemanticElement,
        x: i32,
        y: i32,
    ) -> Option<&SemanticElement> {
        if element.bounds.contains(x, y) {
            for child in &element.children {
                if let Some(found) = Self::find_by_containing_point(child, x, y) {
                    return Some(found);
                }
            }
            return Some(element);
        }
        None
    }

    fn find_nearest_element(
        element: &SemanticElement,
        x: i32,
        y: i32,
        tolerance: u32,
    ) -> Option<&SemanticElement> {
        // Check children first to find the most specific match
        for child in &element.children {
            if let Some(found) = Self::find_nearest_element(child, x, y, tolerance) {
                return Some(found);
            }
        }
        if element.bounds.contains(x, y) {
            return Some(element);
        }
        let cx = element.bounds.center().0;
        let cy = element.bounds.center().1;
        let dist = (cx - x).unsigned_abs().min((cy - y).unsigned_abs());
        if dist <= tolerance {
            return Some(element);
        }
        None
    }
}

impl Default for GroundingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CaptureSource, CaptureSourceKind, Rect, UiElementKind};

    fn make_test_hierarchy() -> UiHierarchy {
        UiHierarchy {
            root: SemanticElement {
                id: "root".into(),
                element_type: UiElementKind::Window,
                text: Some("Main".into()),
                bounds: Rect::new(0, 0, 800, 600),
                confidence: 1.0,
                is_interactive: false,
                is_enabled: true,
                children: vec![
                    SemanticElement {
                        id: "btn-ok".into(),
                        element_type: UiElementKind::Button,
                        text: Some("OK".into()),
                        bounds: Rect::new(100, 100, 80, 30),
                        confidence: 0.95,
                        is_interactive: true,
                        is_enabled: true,
                        children: vec![],
                        parent_id: Some("root".into()),
                        properties: {
                            let mut m = HashMap::new();
                            m.insert("automation_id".into(), "ok-button".into());
                            m
                        },
                    },
                    SemanticElement {
                        id: "txt-search".into(),
                        element_type: UiElementKind::TextBox,
                        text: Some("Search".into()),
                        bounds: Rect::new(100, 200, 200, 25),
                        confidence: 0.9,
                        is_interactive: true,
                        is_enabled: true,
                        children: vec![],
                        parent_id: Some("root".into()),
                        properties: HashMap::new(),
                    },
                ],
                parent_id: None,
                properties: HashMap::new(),
            },
            timestamp: chrono::Utc::now(),
            source: CaptureSource {
                kind: CaptureSourceKind::FullScreen,
                display_index: Some(0),
                window_id: None,
                region: None,
            },
            processing_time_ms: 0,
        }
    }

    #[test]
    fn test_ground_by_accessibility_id() {
        let engine = GroundingEngine::new();
        let hierarchy = make_test_hierarchy();
        let target = engine
            .ground_by_accessibility_id(&hierarchy, "ok-button")
            .unwrap();
        assert_eq!(target.element.text.as_deref(), Some("OK"));
    }

    #[test]
    fn test_ground_by_text() {
        let engine = GroundingEngine::new();
        let hierarchy = make_test_hierarchy();
        let targets = engine.ground_by_text(&hierarchy, "OK").unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].element.id, "btn-ok");
    }

    #[test]
    fn test_ground_by_coordinates() {
        let engine = GroundingEngine::new();
        let hierarchy = make_test_hierarchy();
        let target = engine
            .ground_by_coordinates(&hierarchy, 140, 115, 10)
            .unwrap();
        assert_eq!(target.element.id, "btn-ok");
    }

    #[test]
    fn test_ground_by_hierarchy_path() {
        let engine = GroundingEngine::new();
        let hierarchy = make_test_hierarchy();
        let target = engine
            .ground_by_hierarchy_path(&hierarchy, &["OK".to_string()])
            .unwrap();
        assert_eq!(target.element.id, "btn-ok");
    }

    #[test]
    fn test_ground_by_spatial() {
        let engine = GroundingEngine::new();
        let hierarchy = make_test_hierarchy();
        let target = engine
            .ground_by_spatial(&hierarchy, "btn-ok", 0, 100)
            .unwrap();
        assert_eq!(target.element.id, "txt-search");
    }

    #[test]
    fn test_verify_target() {
        let engine = GroundingEngine::new();
        let hierarchy = make_test_hierarchy();
        let target = engine
            .ground_by_accessibility_id(&hierarchy, "ok-button")
            .unwrap();
        let verified = engine.verify_target(&hierarchy, &target).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_verify_target_not_found() {
        let engine = GroundingEngine::new();
        let hierarchy = make_test_hierarchy();
        let target = engine
            .ground_by_accessibility_id(&hierarchy, "nonexistent")
            .unwrap_err();
        assert!(matches!(target, VisionError::ElementNotFound(_)));
    }

    #[test]
    fn test_stable_target_caching() {
        let engine = GroundingEngine::new();
        let hierarchy = make_test_hierarchy();
        let target = engine
            .ground_by_accessibility_id(&hierarchy, "ok-button")
            .unwrap();
        let cached = engine.get_stable_target(&target.stable_id);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().element.id, "btn-ok");
    }
}
