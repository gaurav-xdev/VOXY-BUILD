use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::config::GreetingConfig;
use crate::types::{Greeting, GreetingKind, TimeContext, WeatherContext};

/// Template for generating greetings.
#[derive(Debug, Clone)]
pub struct GreetingTemplate {
    pub text: String,
    pub kind: GreetingKind,
    pub time_contexts: Vec<TimeContext>,
    pub weather_contexts: Vec<WeatherContext>,
    pub min_session_duration: Duration,
}

/// Greeting intelligence engine.
pub struct GreetingEngine {
    config: GreetingConfig,
    templates: Vec<GreetingTemplate>,
    used_recently: HashSet<String>,
    greeting_count: usize,
    session_start: Instant,
}

impl GreetingEngine {
    pub fn new(config: GreetingConfig) -> Self {
        let templates = Self::build_templates();
        Self {
            config,
            templates,
            used_recently: HashSet::new(),
            greeting_count: 0,
            session_start: Instant::now(),
        }
    }

    /// Attempt to generate a greeting. Returns None if silence is appropriate.
    #[allow(clippy::too_many_arguments)]
    pub fn generate(
        &mut self,
        time: TimeContext,
        weather: WeatherContext,
        time_since_last_interaction: Duration,
        is_return: bool,
        completed_mission: bool,
        milestones: &[String],
        _now: Instant,
    ) -> Option<Greeting> {
        if self.greeting_count >= self.config.max_greetings_per_session {
            return None;
        }

        if !self.used_recently.is_empty()
            && time_since_last_interaction < self.config.min_greeting_interval
        {
            return None;
        }

        let mut candidates: Vec<(&GreetingTemplate, f64)> = self
            .templates
            .iter()
            .filter(|t| {
                (t.time_contexts.is_empty() || t.time_contexts.contains(&time))
                    && (t.weather_contexts.is_empty() || t.weather_contexts.contains(&weather))
                    && self.session_start.elapsed() >= t.min_session_duration
            })
            .map(|t| {
                let mut score = *self
                    .config
                    .context_scores
                    .get(&format!("{:?}", t.kind).to_lowercase())
                    .unwrap_or(&0.5);

                if is_return && t.kind == GreetingKind::Return {
                    score += 0.5;
                }
                if completed_mission && t.kind == GreetingKind::PostMission {
                    score += 0.2;
                }
                if !milestones.is_empty() && t.kind == GreetingKind::Milestone {
                    score += 0.2;
                }
                if time == TimeContext::Morning && t.kind == GreetingKind::Morning {
                    score += 0.1;
                }

                (t, score)
            })
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (template, score) in &candidates {
            if self.used_recently.contains(&template.text) {
                continue;
            }
            if *score < 0.3 {
                continue;
            }

            let greeting = Greeting {
                text: template.text.clone(),
                kind: template.kind,
                confidence: *score,
                context_used: vec![
                    format!("{:?}", time),
                    format!("{:?}", weather),
                    if is_return {
                        "return".to_string()
                    } else {
                        String::new()
                    },
                ]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect(),
            };

            self.used_recently.insert(template.text.clone());
            self.greeting_count += 1;
            return Some(greeting);
        }

        None
    }

    pub fn reset_session(&mut self) {
        self.used_recently.clear();
        self.greeting_count = 0;
        self.session_start = Instant::now();
    }

    pub fn greeting_count(&self) -> usize {
        self.greeting_count
    }

    fn build_templates() -> Vec<GreetingTemplate> {
        vec![
            GreetingTemplate {
                text: "Morning.".to_string(),
                kind: GreetingKind::Morning,
                time_contexts: vec![TimeContext::EarlyMorning, TimeContext::Morning],
                weather_contexts: vec![],
                min_session_duration: Duration::ZERO,
            },
            GreetingTemplate {
                text: "Welcome back.".to_string(),
                kind: GreetingKind::Return,
                time_contexts: vec![],
                weather_contexts: vec![],
                min_session_duration: Duration::ZERO,
            },
            GreetingTemplate {
                text: "Good to see you.".to_string(),
                kind: GreetingKind::Return,
                time_contexts: vec![],
                weather_contexts: vec![],
                min_session_duration: Duration::ZERO,
            },
            GreetingTemplate {
                text: "Ready when you are.".to_string(),
                kind: GreetingKind::CheckIn,
                time_contexts: vec![],
                weather_contexts: vec![],
                min_session_duration: Duration::from_secs(300),
            },
            GreetingTemplate {
                text: "Mission complete. What's next?".to_string(),
                kind: GreetingKind::PostMission,
                time_contexts: vec![],
                weather_contexts: vec![],
                min_session_duration: Duration::ZERO,
            },
            GreetingTemplate {
                text: "That's a milestone.".to_string(),
                kind: GreetingKind::Milestone,
                time_contexts: vec![],
                weather_contexts: vec![],
                min_session_duration: Duration::ZERO,
            },
            GreetingTemplate {
                text: "Afternoon.".to_string(),
                kind: GreetingKind::Morning,
                time_contexts: vec![TimeContext::Afternoon],
                weather_contexts: vec![],
                min_session_duration: Duration::ZERO,
            },
            GreetingTemplate {
                text: "Evening.".to_string(),
                kind: GreetingKind::Morning,
                time_contexts: vec![TimeContext::Evening],
                weather_contexts: vec![],
                min_session_duration: Duration::ZERO,
            },
        ]
    }
}

impl Default for GreetingEngine {
    fn default() -> Self {
        Self::new(GreetingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeting_morning() {
        let mut engine = GreetingEngine::new(GreetingConfig::default());
        let greeting = engine.generate(
            TimeContext::Morning,
            WeatherContext::Clear,
            Duration::ZERO,
            false,
            false,
            &[],
            Instant::now(),
        );
        assert!(greeting.is_some());
        let g = greeting.unwrap();
        assert_eq!(g.kind, GreetingKind::Morning);
    }

    #[test]
    fn test_greeting_max_per_session() {
        let mut engine = GreetingEngine::new(GreetingConfig::default());
        let now = Instant::now();
        let mut count = 0;
        for i in 0..10 {
            let time = match i % 4 {
                0 => TimeContext::Morning,
                1 => TimeContext::Afternoon,
                2 => TimeContext::Evening,
                _ => TimeContext::Night,
            };
            let weather = match i % 3 {
                0 => WeatherContext::Clear,
                1 => WeatherContext::Rainy,
                _ => WeatherContext::Snowy,
            };
            if engine
                .generate(
                    time,
                    weather,
                    Duration::from_secs(600),
                    false,
                    false,
                    &[],
                    now,
                )
                .is_some()
            {
                count += 1;
            }
        }
        assert_eq!(count, GreetingConfig::default().max_greetings_per_session);
    }

    #[test]
    fn test_greeting_dedup() {
        let mut engine = GreetingEngine::new(GreetingConfig::default());
        let g1 = engine.generate(
            TimeContext::Morning,
            WeatherContext::Clear,
            Duration::ZERO,
            false,
            false,
            &[],
            Instant::now(),
        );
        let g2 = engine.generate(
            TimeContext::Morning,
            WeatherContext::Clear,
            Duration::ZERO,
            false,
            false,
            &[],
            Instant::now(),
        );
        if let (Some(g1), Some(g2)) = (g1, g2) {
            assert_ne!(g1.text, g2.text);
        }
    }

    #[test]
    fn test_greeting_return() {
        let mut engine = GreetingEngine::new(GreetingConfig::default());
        let greeting = engine.generate(
            TimeContext::Afternoon,
            WeatherContext::Clear,
            Duration::from_secs(600),
            true,
            false,
            &[],
            Instant::now(),
        );
        assert!(greeting.is_some());
        assert_eq!(greeting.unwrap().kind, GreetingKind::Return);
    }
}
