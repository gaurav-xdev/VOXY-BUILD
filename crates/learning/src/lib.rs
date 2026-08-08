pub mod behavior;
pub mod calibration;
pub mod config;
pub mod engine;
pub mod error;
pub mod event;
pub mod feedback;
pub mod policies;
pub mod preferences;
pub mod reinforcement;
pub mod strategy;
pub mod types;

pub use behavior::BehaviorAdaptation;
pub use calibration::ConfidenceCalibration;
pub use config::LearningConfig;
pub use engine::{LearningEngine, LearningStats};
pub use error::{LearningError, Result};
pub use event::LearningEvent;
pub use feedback::FeedbackProcessing;
pub use policies::{AdaptiveThresholds, LearningPolicy};
pub use preferences::PreferenceEvolution;
pub use reinforcement::ReinforcementHook;
pub use strategy::StrategyEvolution;
pub use types::{
    AdaptiveThreshold, BehaviorPattern, CalibrationSample, FeedbackEntry, PreferenceEntry,
    ReinforcementSignal, StrategyEffectiveness, UserPreferenceProfile,
};

pub mod prelude {
    pub use crate::behavior::BehaviorAdaptation;
    pub use crate::calibration::ConfidenceCalibration;
    pub use crate::config::LearningConfig;
    pub use crate::engine::{LearningEngine, LearningStats};
    pub use crate::error::{LearningError, Result};
    pub use crate::event::LearningEvent;
    pub use crate::feedback::FeedbackProcessing;
    pub use crate::policies::{AdaptiveThresholds, LearningPolicy};
    pub use crate::preferences::PreferenceEvolution;
    pub use crate::reinforcement::ReinforcementHook;
    pub use crate::strategy::StrategyEvolution;
    pub use crate::types::{
        AdaptiveThreshold, BehaviorPattern, CalibrationSample, FeedbackEntry, PreferenceEntry,
        ReinforcementSignal, StrategyEffectiveness, UserPreferenceProfile,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;
    use voxy_memory::types::{MemoryId, MemoryItem, MemoryState, MemoryType};

    fn make_memory_item() -> MemoryItem {
        MemoryItem {
            id: MemoryId("mem_1".to_string()),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content: json!("test content"),
            importance: 0.8,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 5,
            context_tags: vec!["tag1".to_string()],
            source: "test".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }
    }

    fn make_preference_entry() -> PreferenceEntry {
        PreferenceEntry {
            id: "pref_1".to_string(),
            category: "theme".to_string(),
            key: "dark_mode".to_string(),
            value: json!(true),
            confidence: 0.85,
            source: "explicit".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_count: 10,
            is_stable: true,
            metadata: HashMap::new(),
        }
    }

    fn make_behavior_pattern() -> BehaviorPattern {
        BehaviorPattern {
            id: "b1".to_string(),
            pattern_type: "app_launch".to_string(),
            description: "Opens browser at 9 AM".to_string(),
            trigger_context: vec!["morning".to_string(), "weekday".to_string()],
            frequency: 0.9,
            effectiveness: 0.75,
            last_observed: Utc::now(),
            adaptation_count: 3,
            is_active: true,
        }
    }

    fn make_reinforcement_signal() -> ReinforcementSignal {
        ReinforcementSignal {
            id: "r1".to_string(),
            action_id: "act_1".to_string(),
            action_description: "Opened email client".to_string(),
            reward: 1.0,
            context: HashMap::new(),
            timestamp: Utc::now(),
            source: "user".to_string(),
        }
    }

    fn make_feedback_entry() -> FeedbackEntry {
        FeedbackEntry {
            id: "fb_1".to_string(),
            user_id: "user_1".to_string(),
            target_id: "tgt_1".to_string(),
            target_type: "action".to_string(),
            sentiment: 0.9,
            comment: Some("Great work".to_string()),
            context: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    fn make_calibration_sample() -> CalibrationSample {
        CalibrationSample {
            id: "cs_1".to_string(),
            estimator_id: "est_1".to_string(),
            predicted_confidence: 0.8,
            actual_correctness: true,
            timestamp: Utc::now(),
            context: HashMap::new(),
        }
    }

    fn make_strategy_effectiveness() -> StrategyEffectiveness {
        StrategyEffectiveness {
            strategy_id: "strat_1".to_string(),
            strategy_name: "greedy".to_string(),
            effectiveness_score: 0.85,
            trial_count: 100,
            success_count: 85,
            average_duration_ms: 250.0,
            last_updated: Utc::now(),
        }
    }

    fn make_adaptive_threshold() -> AdaptiveThreshold {
        AdaptiveThreshold {
            id: "th_1".to_string(),
            name: "confidence_threshold".to_string(),
            current_value: 0.7,
            default_value: 0.5,
            min_value: 0.0,
            max_value: 1.0,
            adjustment_rate: 0.05,
            last_adjusted: Utc::now(),
            adjustment_count: 5,
            reason: Some("calibration".to_string()),
        }
    }

    #[test]
    fn test_preference_entry_creation() {
        let p = make_preference_entry();
        assert_eq!(p.id, "pref_1");
        assert_eq!(p.category, "theme");
        assert_eq!(p.key, "dark_mode");
        assert!(p.is_stable);
        assert!((p.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_behavior_pattern_creation() {
        let b = make_behavior_pattern();
        assert_eq!(b.id, "b1");
        assert_eq!(b.pattern_type, "app_launch");
        assert!(b.is_active);
        assert!((b.frequency - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reinforcement_signal_creation() {
        let rs = make_reinforcement_signal();
        assert_eq!(rs.id, "r1");
        assert_eq!(rs.action_id, "act_1");
        assert!((rs.reward - 1.0).abs() < f64::EPSILON);
        assert_eq!(rs.source, "user");
    }

    #[test]
    fn test_feedback_entry_creation() {
        let fb = make_feedback_entry();
        assert_eq!(fb.id, "fb_1");
        assert_eq!(fb.user_id, "user_1");
        assert_eq!(fb.comment, Some("Great work".to_string()));
        assert!((fb.sentiment - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calibration_sample_creation() {
        let cs = make_calibration_sample();
        assert_eq!(cs.id, "cs_1");
        assert_eq!(cs.estimator_id, "est_1");
        assert!(cs.actual_correctness);
        assert!((cs.predicted_confidence - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_strategy_effectiveness_creation() {
        let se = make_strategy_effectiveness();
        assert_eq!(se.strategy_id, "strat_1");
        assert_eq!(se.strategy_name, "greedy");
        assert_eq!(se.trial_count, 100);
        assert_eq!(se.success_count, 85);
    }

    #[test]
    fn test_adaptive_threshold_creation() {
        let at = make_adaptive_threshold();
        assert_eq!(at.id, "th_1");
        assert!((at.current_value - 0.7).abs() < f64::EPSILON);
        assert!((at.default_value - 0.5).abs() < f64::EPSILON);
        assert_eq!(at.reason, Some("calibration".to_string()));
    }

    #[test]
    fn test_user_preference_profile_creation() {
        let pref = make_preference_entry();
        let bp = make_behavior_pattern();
        let at = make_adaptive_threshold();
        let profile = UserPreferenceProfile {
            preferences: vec![pref],
            behavior_patterns: vec![bp],
            adaptive_thresholds: vec![at],
            learning_rate: 0.1,
            adaptation_level: 0.5,
            last_updated: Utc::now(),
        };
        assert_eq!(profile.preferences.len(), 1);
        assert_eq!(profile.behavior_patterns.len(), 1);
        assert_eq!(profile.adaptive_thresholds.len(), 1);
        assert!((profile.learning_rate - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_learning_config_default() {
        let cfg = LearningConfig::default();
        assert!(cfg.enable_preference_evolution);
        assert!(cfg.enable_behavior_adaptation);
        assert!(cfg.enable_reinforcement);
        assert!(cfg.enable_confidence_calibration);
        assert!(cfg.enable_strategy_evolution);
        assert_eq!(cfg.feedback_window_size, 50);
        assert_eq!(cfg.min_feedback_for_adjustment, 5);
        assert!((cfg.learning_rate - 0.1).abs() < f64::EPSILON);
        assert_eq!(cfg.adaptation_cooldown_seconds, 60);
        assert_eq!(cfg.max_preference_history, 1000);
        assert_eq!(cfg.calibration_samples_needed, 20);
        assert_eq!(cfg.strategy_review_interval_seconds, 3600);
    }

    #[test]
    fn test_learning_event_preference_updated_display() {
        let ev = LearningEvent::PreferenceUpdated {
            preference_id: "pref_1".to_string(),
            category: "theme".to_string(),
            old_value: "false".to_string(),
            new_value: "true".to_string(),
            confidence: 0.9,
        };
        let s = format!("{}", ev);
        assert!(s.contains("PreferenceUpdated"));
        assert!(s.contains("pref_1"));
        assert!(s.contains("theme"));
    }

    #[test]
    fn test_learning_event_behavior_adapted_display() {
        let ev = LearningEvent::BehaviorAdapted {
            behavior_id: "b1".to_string(),
            adaptation: "increase_frequency".to_string(),
            trigger: "morning".to_string(),
        };
        let s = format!("{}", ev);
        assert!(s.contains("BehaviorAdapted"));
        assert!(s.contains("b1"));
    }

    #[test]
    fn test_learning_event_reinforcement_applied_display() {
        let ev = LearningEvent::ReinforcementApplied {
            hook_id: "hook_1".to_string(),
            action: "open_app".to_string(),
            reward: 1.0,
        };
        let s = format!("{}", ev);
        assert!(s.contains("ReinforcementApplied"));
        assert!(s.contains("hook_1"));
    }

    #[test]
    fn test_learning_event_strategy_evolved_display() {
        let ev = LearningEvent::StrategyEvolved {
            strategy_id: "strat_1".to_string(),
            old_effectiveness: 0.5,
            new_effectiveness: 0.8,
        };
        let s = format!("{}", ev);
        assert!(s.contains("StrategyEvolved"));
        assert!(s.contains("strat_1"));
    }

    #[test]
    fn test_learning_error_invalid_config_display() {
        let err = LearningError::InvalidConfig("bad config".to_string());
        let s = format!("{}", err);
        assert!(s.contains("Invalid configuration"));
        assert!(s.contains("bad config"));
    }

    #[test]
    fn test_learning_error_preference_error_display() {
        let err = LearningError::PreferenceError("not found".to_string());
        let s = format!("{}", err);
        assert!(s.contains("Preference error"));
        assert!(s.contains("not found"));
    }

    #[test]
    fn test_learning_error_timeout_display() {
        let err = LearningError::Timeout("operation timed out".to_string());
        let s = format!("{}", err);
        assert!(s.contains("Timeout"));
        assert!(s.contains("operation timed out"));
    }

    #[test]
    fn test_learning_stats_creation() {
        let stats = LearningStats {
            preferences_tracked: 10,
            behavior_patterns: 5,
            reinforcement_signals: 20,
            calibration_samples: 15,
            feedback_processed: 30,
            strategies_active: 3,
            thresholds_managed: 7,
            is_learning: true,
            last_learning_event: Some(Utc::now()),
        };
        assert_eq!(stats.preferences_tracked, 10);
        assert_eq!(stats.behavior_patterns, 5);
        assert_eq!(stats.reinforcement_signals, 20);
        assert!(stats.is_learning);
        assert!(stats.last_learning_event.is_some());
    }

    #[test]
    fn test_memory_item_type_accessible() {
        let mi = make_memory_item();
        assert_eq!(mi.id.0, "mem_1");
        assert_eq!(mi.memory_type, MemoryType::Episodic);
        assert_eq!(mi.state, MemoryState::Active);
        assert!((mi.importance - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_preference_entry_default_metadata() {
        let p = make_preference_entry();
        assert!(p.metadata.is_empty());
    }

    #[test]
    fn test_feedback_entry_empty_context() {
        let fb = make_feedback_entry();
        assert!(fb.context.is_empty());
        assert_eq!(fb.target_type, "action");
    }

    #[test]
    fn test_calibration_sample_correctness_field() {
        let cs = make_calibration_sample();
        assert!(cs.actual_correctness);
        let cs2 = CalibrationSample {
            actual_correctness: false,
            ..make_calibration_sample()
        };
        assert!(!cs2.actual_correctness);
    }

    #[test]
    fn test_strategy_effectiveness_duration() {
        let se = make_strategy_effectiveness();
        assert!((se.average_duration_ms - 250.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_adaptive_threshold_min_max() {
        let at = make_adaptive_threshold();
        assert!((at.min_value - 0.0).abs() < f64::EPSILON);
        assert!((at.max_value - 1.0).abs() < f64::EPSILON);
        assert!((at.adjustment_rate - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_behavior_pattern_multiple_triggers() {
        let b = make_behavior_pattern();
        assert_eq!(b.trigger_context.len(), 2);
        assert!(b.trigger_context.contains(&"morning".to_string()));
    }

    #[test]
    fn test_reinforcement_signal_with_context() {
        let mut ctx = HashMap::new();
        ctx.insert("app".to_string(), "chrome".to_string());
        let rs = ReinforcementSignal {
            context: ctx.clone(),
            ..make_reinforcement_signal()
        };
        assert_eq!(rs.context.get("app"), Some(&"chrome".to_string()));
    }

    #[test]
    fn test_feedback_entry_no_comment() {
        let fb = FeedbackEntry {
            comment: None,
            ..make_feedback_entry()
        };
        assert!(fb.comment.is_none());
    }

    #[test]
    fn test_preference_entry_evidence_count() {
        let p = make_preference_entry();
        assert_eq!(p.evidence_count, 10);
        let p2 = PreferenceEntry {
            evidence_count: 0,
            ..make_preference_entry()
        };
        assert_eq!(p2.evidence_count, 0);
    }

    #[test]
    fn test_user_preference_profile_learning_rate() {
        let profile = UserPreferenceProfile {
            preferences: vec![],
            behavior_patterns: vec![],
            adaptive_thresholds: vec![],
            learning_rate: 0.05,
            adaptation_level: 0.0,
            last_updated: Utc::now(),
        };
        assert!((profile.learning_rate - 0.05).abs() < f64::EPSILON);
        assert!((profile.adaptation_level - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_learning_stats_not_learning() {
        let stats = LearningStats {
            is_learning: false,
            last_learning_event: None,
            preferences_tracked: 0,
            behavior_patterns: 0,
            reinforcement_signals: 0,
            calibration_samples: 0,
            feedback_processed: 0,
            strategies_active: 0,
            thresholds_managed: 0,
        };
        assert!(!stats.is_learning);
        assert!(stats.last_learning_event.is_none());
    }

    #[test]
    fn test_learning_event_confidence_calibrated_display() {
        let ev = LearningEvent::ConfidenceCalibrated {
            estimator_id: "est_1".to_string(),
            old_threshold: 0.6,
            new_threshold: 0.75,
            samples: 30,
        };
        let s = format!("{}", ev);
        assert!(s.contains("ConfidenceCalibrated"));
        assert!(s.contains("est_1"));
        assert!(s.contains("0.75"));
    }

    #[test]
    fn test_learning_event_threshold_adjusted_display() {
        let ev = LearningEvent::ThresholdAdjusted {
            threshold_id: "th_1".to_string(),
            old_value: 0.5,
            new_value: 0.55,
            reason: "performance_drop".to_string(),
        };
        let s = format!("{}", ev);
        assert!(s.contains("ThresholdAdjusted"));
        assert!(s.contains("th_1"));
        assert!(s.contains("0.55"));
    }

    #[test]
    fn test_memory_item_related_ids() {
        let mi = make_memory_item();
        assert!(mi.related_ids.is_empty());
        assert!(mi.parent_id.is_none());
    }

    #[test]
    fn test_adaptive_threshold_adjustment_count() {
        let at = make_adaptive_threshold();
        assert_eq!(at.adjustment_count, 5);
        let at2 = AdaptiveThreshold {
            adjustment_count: 0,
            ..make_adaptive_threshold()
        };
        assert_eq!(at2.adjustment_count, 0);
    }
}
