use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecisionType {
    Interrupt,
    Silence,
    Remind,
    Congratulate,
    Wait,
    SuggestBreak,
    ChangeTopic,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub decision_type: DecisionType,
    pub confidence: f64,
    pub reason: String,
    pub priority: f64,
    pub context: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub is_user_speaking: bool,
    pub is_user_typing: bool,
    pub user_idle_ms: u64,
    pub current_task: Option<String>,
    pub pending_tasks: usize,
    pub recent_errors: usize,
    pub recent_completions: usize,
    pub session_duration_ms: u64,
    pub time_since_last_interaction: u64,
    pub is_meeting_active: bool,
    pub is_focus_mode: bool,
    pub battery_level: Option<f64>,
    pub cpu_usage: Option<f64>,
}

pub struct DecisionEngine {
    config: DecisionConfig,
    decision_history: Vec<Decision>,
}

#[derive(Debug, Clone)]
pub struct DecisionConfig {
    pub interrupt_threshold: f64,
    pub silence_threshold: f64,
    pub remind_threshold: f64,
    pub congratulate_threshold: f64,
    pub wait_threshold: f64,
    pub max_recent_decisions: usize,
}

impl Default for DecisionConfig {
    fn default() -> Self {
        Self {
            interrupt_threshold: 0.7,
            silence_threshold: 0.3,
            remind_threshold: 0.6,
            congratulate_threshold: 0.8,
            wait_threshold: 0.5,
            max_recent_decisions: 100,
        }
    }
}

impl DecisionEngine {
    pub fn new(config: DecisionConfig) -> Self {
        Self {
            config,
            decision_history: Vec::new(),
        }
    }

    pub fn make_decision(&mut self, context: &DecisionContext) -> Decision {
        let scores = self.score_all_options(context);

        let (best_type, best_score) = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
            .unwrap_or((DecisionType::Silence, 0.0));

        let decision = Decision {
            decision_type: best_type,
            confidence: best_score,
            reason: self.generate_reason(best_type, context),
            priority: self.calculate_priority(best_type, context),
            context: self.build_context_map(context),
            timestamp: Utc::now(),
        };

        self.decision_history.push(decision.clone());
        if self.decision_history.len() > self.config.max_recent_decisions {
            self.decision_history.swap_remove(0);
        }

        decision
    }

    pub fn decision_history(&self) -> &[Decision] {
        &self.decision_history
    }

    fn score_all_options(&self, context: &DecisionContext) -> Vec<(DecisionType, f64)> {
        vec![
            (DecisionType::Interrupt, self.score_interrupt(context)),
            (DecisionType::Silence, self.score_silence(context)),
            (DecisionType::Remind, self.score_remind(context)),
            (DecisionType::Congratulate, self.score_congratulate(context)),
            (DecisionType::Wait, self.score_wait(context)),
            (
                DecisionType::SuggestBreak,
                self.score_suggest_break(context),
            ),
            (DecisionType::ChangeTopic, self.score_change_topic(context)),
            (DecisionType::Escalate, self.score_escalate(context)),
        ]
    }

    fn score_interrupt(&self, context: &DecisionContext) -> f64 {
        let mut score: f64 = 0.0;

        if context.is_user_speaking {
            score += 0.3;
        }

        if context.user_idle_ms > 5_000 {
            score += 0.2;
        }

        if context.recent_errors > 3 {
            score += 0.3;
        }

        if context.is_meeting_active {
            score -= 0.5;
        }

        if context.is_focus_mode {
            score -= 0.3;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_silence(&self, context: &DecisionContext) -> f64 {
        let mut score: f64 = 0.5;

        if context.is_user_typing {
            score += 0.3;
        }

        if context.is_focus_mode {
            score += 0.2;
        }

        if context.user_idle_ms < 1_000 {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_remind(&self, context: &DecisionContext) -> f64 {
        let mut score: f64 = 0.0;

        if context.pending_tasks > 0 {
            score += 0.4;
        }

        if context.time_since_last_interaction > 300_000 {
            score += 0.3;
        }

        if context.user_idle_ms > 60_000 {
            score += 0.2;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_congratulate(&self, context: &DecisionContext) -> f64 {
        let mut score: f64 = 0.0;

        if context.recent_completions > 0 {
            score += 0.5;
        }

        if context.recent_completions > 3 {
            score += 0.3;
        }

        if context.recent_errors == 0 && context.recent_completions > 0 {
            score += 0.2;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_wait(&self, context: &DecisionContext) -> f64 {
        let mut score: f64 = 0.3;

        if context.is_user_speaking || context.is_user_typing {
            score += 0.4;
        }

        if context.user_idle_ms < 2_000 {
            score += 0.2;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_suggest_break(&self, context: &DecisionContext) -> f64 {
        let mut score: f64 = 0.0;

        if context.session_duration_ms > 7_200_000 {
            score += 0.6;
        } else if context.session_duration_ms > 3_600_000 {
            score += 0.3;
        }

        if let Some(battery) = context.battery_level {
            if battery < 0.2 {
                score += 0.3;
            }
        }

        if context.user_idle_ms > 300_000 {
            score += 0.2;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_change_topic(&self, context: &DecisionContext) -> f64 {
        let mut score: f64 = 0.0;

        if context.recent_errors > 5 {
            score += 0.4;
        }

        if context.user_idle_ms > 120_000 {
            score += 0.3;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_escalate(&self, context: &DecisionContext) -> f64 {
        let mut score: f64 = 0.0;

        if context.recent_errors > 10 {
            score += 0.5;
        }

        if let Some(cpu) = context.cpu_usage {
            if cpu > 0.9 {
                score += 0.3;
            }
        }

        score.clamp(0.0, 1.0)
    }

    fn calculate_priority(&self, decision_type: DecisionType, context: &DecisionContext) -> f64 {
        let base_priority: f64 = match decision_type {
            DecisionType::Escalate => 0.9,
            DecisionType::Interrupt => 0.7,
            DecisionType::Congratulate => 0.6,
            DecisionType::Remind => 0.5,
            DecisionType::SuggestBreak => 0.4,
            DecisionType::ChangeTopic => 0.3,
            DecisionType::Wait => 0.2,
            DecisionType::Silence => 0.1,
        };

        let mut priority: f64 = base_priority;

        if context.is_meeting_active {
            priority *= 0.5;
        }

        if context.is_focus_mode {
            priority *= 0.7;
        }

        priority.clamp(0.0, 1.0)
    }

    fn generate_reason(&self, decision_type: DecisionType, context: &DecisionContext) -> String {
        match decision_type {
            DecisionType::Interrupt => {
                if context.recent_errors > 3 {
                    "User may need help with errors".to_string()
                } else if context.user_idle_ms > 5_000 {
                    "User seems idle, may want assistance".to_string()
                } else {
                    "Interaction opportunity detected".to_string()
                }
            }
            DecisionType::Silence => {
                if context.is_user_typing {
                    "User is actively typing".to_string()
                } else if context.is_focus_mode {
                    "User is in focus mode".to_string()
                } else {
                    "No action needed".to_string()
                }
            }
            DecisionType::Remind => {
                if context.pending_tasks > 0 {
                    format!("{} tasks pending", context.pending_tasks)
                } else {
                    "User may have forgotten something".to_string()
                }
            }
            DecisionType::Congratulate => {
                format!("{} recent completions", context.recent_completions)
            }
            DecisionType::Wait => "Waiting for user to finish".to_string(),
            DecisionType::SuggestBreak => {
                if context.session_duration_ms > 7_200_000 {
                    "Long coding session detected".to_string()
                } else {
                    "Battery may be low".to_string()
                }
            }
            DecisionType::ChangeTopic => "User may be stuck".to_string(),
            DecisionType::Escalate => "Critical situation detected".to_string(),
        }
    }

    fn build_context_map(&self, context: &DecisionContext) -> HashMap<String, String> {
        let mut map = HashMap::new();

        if let Some(task) = &context.current_task {
            map.insert("current_task".to_string(), task.clone());
        }

        map.insert(
            "session_duration_ms".to_string(),
            context.session_duration_ms.to_string(),
        );
        map.insert(
            "pending_tasks".to_string(),
            context.pending_tasks.to_string(),
        );
        map.insert(
            "recent_errors".to_string(),
            context.recent_errors.to_string(),
        );

        map
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new(DecisionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> DecisionContext {
        DecisionContext {
            is_user_speaking: false,
            is_user_typing: false,
            user_idle_ms: 10_000,
            current_task: Some("coding".to_string()),
            pending_tasks: 2,
            recent_errors: 3,
            recent_completions: 1,
            session_duration_ms: 3_600_000,
            time_since_last_interaction: 60_000,
            is_meeting_active: false,
            is_focus_mode: false,
            battery_level: Some(0.8),
            cpu_usage: Some(0.5),
        }
    }

    #[test]
    fn test_decision_engine_creation() {
        let engine = DecisionEngine::new(DecisionConfig::default());
        assert_eq!(engine.decision_history().len(), 0);
    }

    #[test]
    fn test_make_decision() {
        let mut engine = DecisionEngine::default();
        let context = create_test_context();
        let decision = engine.make_decision(&context);
        assert!(decision.confidence > 0.0);
        assert!(!decision.reason.is_empty());
    }

    #[test]
    fn test_interrupt_when_errors() {
        let mut engine = DecisionEngine::default();
        let mut context = create_test_context();
        context.recent_errors = 5;
        context.recent_completions = 0;
        context.user_idle_ms = 10_000;
        context.is_user_speaking = true;
        let decision = engine.make_decision(&context);
        assert_eq!(decision.decision_type, DecisionType::Interrupt);
    }

    #[test]
    fn test_silence_when_typing() {
        let mut engine = DecisionEngine::default();
        let mut context = create_test_context();
        context.is_user_typing = true;
        context.is_focus_mode = true;
        let decision = engine.make_decision(&context);
        assert_eq!(decision.decision_type, DecisionType::Silence);
    }

    #[test]
    fn test_congratulate_on_completions() {
        let mut engine = DecisionEngine::default();
        let mut context = create_test_context();
        context.recent_completions = 5;
        context.recent_errors = 0;
        let decision = engine.make_decision(&context);
        assert_eq!(decision.decision_type, DecisionType::Congratulate);
    }

    #[test]
    fn test_decision_history() {
        let mut engine = DecisionEngine::default();
        let context = create_test_context();
        engine.make_decision(&context);
        engine.make_decision(&context);
        assert_eq!(engine.decision_history().len(), 2);
    }
}
