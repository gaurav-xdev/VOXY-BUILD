use std::time::Duration;

use crate::attention::ActivityKind;
use crate::config::ConversationConfig;
use crate::types::ConversationPacing;

/// Conversation timing engine — controls pauses, thinking time, pacing.
pub struct ConversationTiming {
    config: ConversationConfig,
    exchange_count: usize,
    avg_response_length: usize,
}

impl ConversationTiming {
    pub fn new(config: ConversationConfig) -> Self {
        Self {
            config,
            exchange_count: 0,
            avg_response_length: 100,
        }
    }

    /// Calculate appropriate pacing for a response.
    pub fn calculate_pacing(
        &mut self,
        topic_complexity: f64,
        activity: Option<ActivityKind>,
        response_length: usize,
    ) -> ConversationPacing {
        let thinking = Duration::from_millis(
            (self.config.thinking_pause.as_millis() as f64
                * (1.0 + topic_complexity * self.config.complexity_multiplier)) as u64,
        )
        .min(self.config.max_pause);

        let response_delay = if let Some(act) = activity {
            if act.is_deep_focus() {
                Duration::from_secs(2)
            } else {
                Duration::from_millis(300)
            }
        } else {
            Duration::from_millis(500)
        };

        self.avg_response_length = (self.avg_response_length * self.exchange_count
            + response_length)
            / (self.exchange_count + 1).max(1);
        self.exchange_count += 1;

        ConversationPacing {
            thinking_duration: thinking,
            response_delay,
            pause_before_speaking: Duration::from_millis(200),
            estimated_response_length: response_length,
            topic_complexity,
        }
    }

    /// Should we pause before speaking?
    pub fn should_pause(&self) -> bool {
        self.exchange_count > 2
    }

    pub fn exchange_count(&self) -> usize {
        self.exchange_count
    }
}

impl Default for ConversationTiming {
    fn default() -> Self {
        Self::new(ConversationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacing_simple_topic() {
        let mut timing = ConversationTiming::new(ConversationConfig::default());
        let pacing = timing.calculate_pacing(0.2, Some(ActivityKind::Browsing), 50);
        assert!(pacing.thinking_duration < Duration::from_secs(2));
    }

    #[test]
    fn test_pacing_complex_topic() {
        let mut timing = ConversationTiming::new(ConversationConfig::default());
        let pacing = timing.calculate_pacing(0.9, Some(ActivityKind::Coding), 200);
        assert!(pacing.thinking_duration > Duration::from_millis(500));
    }

    #[test]
    fn test_exchange_count() {
        let mut timing = ConversationTiming::new(ConversationConfig::default());
        timing.calculate_pacing(0.5, None, 100);
        timing.calculate_pacing(0.5, None, 100);
        assert_eq!(timing.exchange_count(), 2);
    }
}
