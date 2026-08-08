use std::time::{Duration, Instant};

use crate::attention::AttentionState;
use crate::config::SilenceConfig;

/// Decision from silence intelligence.
#[derive(Debug, Clone, PartialEq)]
pub enum SilenceDecision {
    /// Stay silent — do not interrupt.
    Silent { reason: SilenceReason },
    /// Okay to speak.
    Speak { reason: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SilenceReason {
    UserInDeepFocus,
    InterruptCooldown,
    FocusThresholdExceeded,
    AnnoyanceThreshold,
    NoReasonToSpeak,
    IdleTooLong,
}

/// Tracks silence intelligence.
pub struct SilenceIntelligence {
    config: SilenceConfig,
    last_speech: Option<Instant>,
    last_interruption: Option<Instant>,
    consecutive_silence: Duration,
    interruption_count: usize,
}

impl SilenceIntelligence {
    pub fn new(config: SilenceConfig) -> Self {
        Self {
            config,
            last_speech: None,
            last_interruption: None,
            consecutive_silence: Duration::ZERO,
            interruption_count: 0,
        }
    }

    /// Decide whether to speak or remain silent.
    pub fn decide(
        &mut self,
        attention: &AttentionState,
        has_reason: bool,
        now: Instant,
    ) -> SilenceDecision {
        if attention.deep_focus {
            return SilenceDecision::Silent {
                reason: SilenceReason::UserInDeepFocus,
            };
        }

        let annoyance = self.estimate_annoyance(now);
        if annoyance >= self.config.annoyance_threshold {
            return SilenceDecision::Silent {
                reason: SilenceReason::AnnoyanceThreshold,
            };
        }

        if let Some(last) = self.last_interruption {
            let elapsed = now.duration_since(last);
            if elapsed < self.config.interruption_cooldown {
                return SilenceDecision::Silent {
                    reason: SilenceReason::InterruptCooldown,
                };
            }
        }

        if attention.focus_level >= self.config.focus_interrupt_threshold {
            return SilenceDecision::Silent {
                reason: SilenceReason::FocusThresholdExceeded,
            };
        }

        if !has_reason {
            return SilenceDecision::Silent {
                reason: SilenceReason::NoReasonToSpeak,
            };
        }

        SilenceDecision::Speak {
            reason: "appropriate_moment".to_string(),
        }
    }

    pub fn record_speech(&mut self, now: Instant) {
        self.last_speech = Some(now);
        self.consecutive_silence = Duration::ZERO;
    }

    pub fn record_interruption(&mut self, now: Instant) {
        self.last_interruption = Some(now);
        self.interruption_count += 1;
    }

    pub fn tick(&mut self, dt: Duration) {
        self.consecutive_silence += dt;
    }

    fn estimate_annoyance(&self, now: Instant) -> f64 {
        let recency_factor = self
            .last_interruption
            .map(|t| {
                let elapsed = now.duration_since(t).as_secs_f64();
                (1.0 - (elapsed / 300.0)).clamp(0.0, 1.0)
            })
            .unwrap_or(0.0);

        let count_factor = (self.interruption_count as f64 / 10.0).clamp(0.0, 1.0);

        (recency_factor * 0.6 + count_factor * 0.4).clamp(0.0, 1.0)
    }

    pub fn interruption_count(&self) -> usize {
        self.interruption_count
    }
}

impl Default for SilenceIntelligence {
    fn default() -> Self {
        Self::new(SilenceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attention::ActivityKind;

    fn make_attention(deep_focus: bool, focus: f64) -> AttentionState {
        AttentionState {
            activity: ActivityKind::Coding,
            focus_level: focus,
            deep_focus,
            can_interrupt: !deep_focus,
            stress_estimate: 0.0,
            state_duration: Duration::from_secs(60),
            detection_confidence: 0.8,
        }
    }

    #[test]
    fn test_silent_when_deep_focus() {
        let mut si = SilenceIntelligence::new(SilenceConfig::default());
        let att = make_attention(true, 0.9);
        let decision = si.decide(&att, true, Instant::now());
        assert_eq!(
            decision,
            SilenceDecision::Silent {
                reason: SilenceReason::UserInDeepFocus
            }
        );
    }

    #[test]
    fn test_speak_when_low_focus() {
        let mut si = SilenceIntelligence::new(SilenceConfig::default());
        let att = make_attention(false, 0.3);
        let decision = si.decide(&att, true, Instant::now());
        assert_eq!(
            decision,
            SilenceDecision::Speak {
                reason: "appropriate_moment".to_string()
            }
        );
    }

    #[test]
    fn test_silent_when_no_reason() {
        let mut si = SilenceIntelligence::new(SilenceConfig::default());
        let att = make_attention(false, 0.3);
        let decision = si.decide(&att, false, Instant::now());
        assert_eq!(
            decision,
            SilenceDecision::Silent {
                reason: SilenceReason::NoReasonToSpeak
            }
        );
    }

    #[test]
    fn test_annoyance_builds() {
        let mut si = SilenceIntelligence::new(SilenceConfig::default());
        for _ in 0..5 {
            si.record_interruption(Instant::now());
        }
        let att = make_attention(false, 0.5);
        let decision = si.decide(&att, true, Instant::now());
        assert_eq!(
            decision,
            SilenceDecision::Silent {
                reason: SilenceReason::AnnoyanceThreshold
            }
        );
    }
}
