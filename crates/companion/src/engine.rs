use std::time::{Duration, Instant};

use chrono::Timelike;

use crate::attention::AttentionModel;
use crate::config::CompanionConfig;
use crate::conversation::ConversationTiming;
use crate::greeting::GreetingEngine;
use crate::journey::SharedJourney;
use crate::memory::MemoryMoments;
use crate::micro::MicroEngine;
use crate::mission::MissionCompanion;
use crate::personality::CompanionPersonality;
use crate::presence::PresenceState;
use crate::presence::PresenceSystem;
use crate::score::PresenceScoreEngine;
use crate::silence::SilenceIntelligence;
use crate::types::{CompanionInput, CompanionOutput, MemoryKind, UserPresence};

/// The Companion Intelligence Engine.
///
/// Orchestrates all subsystems to produce a single update output each cycle.
/// Designed for <0.2ms update cycles on modern hardware.
pub struct CompanionEngine {
    config: CompanionConfig,
    personality: CompanionPersonality,
    presence: PresenceSystem,
    attention: AttentionModel,
    greeting: GreetingEngine,
    silence: SilenceIntelligence,
    micro: MicroEngine,
    mission: MissionCompanion,
    journey: SharedJourney,
    memory: MemoryMoments,
    conversation: ConversationTiming,
    score: PresenceScoreEngine,
    update_count: u64,
    last_update: Option<Instant>,
}

impl CompanionEngine {
    pub fn new(config: CompanionConfig, personality: CompanionPersonality) -> Self {
        Self {
            greeting: GreetingEngine::new(config.greeting.clone()),
            silence: SilenceIntelligence::new(config.silence.clone()),
            micro: MicroEngine::new(config.micro.clone()),
            memory: MemoryMoments::new(config.memory.clone()),
            conversation: ConversationTiming::new(config.conversation.clone()),
            score: PresenceScoreEngine::new(config.score_weights.clone()),
            presence: PresenceSystem::new(config.presence.clone()),
            attention: AttentionModel::new(),
            mission: MissionCompanion::new(),
            journey: SharedJourney::new(50),
            config,
            personality,
            update_count: 0,
            last_update: None,
        }
    }

    /// Run one update cycle. This is the main entry point.
    ///
    /// Target: <0.2ms per call.
    pub fn update(&mut self, input: &CompanionInput) -> CompanionOutput {
        let cycle_start = Instant::now();
        let dt = self
            .last_update
            .map(|t| cycle_start.duration_since(t))
            .unwrap_or(Duration::from_millis(100));
        self.last_update = Some(cycle_start);
        self.update_count += 1;

        // 1. Attention model
        let attention = self.attention.update(
            input.current_activity,
            input.idle_duration,
            input.stress_estimate,
            Some(input.focus_level),
            cycle_start,
        );

        // 2. Presence system
        let presence_snapshot = self.presence.tick(dt, &input.user_presence);

        // 3. Presence score
        let score_breakdown = self.score.compute(input, &attention);

        // 4. Silence intelligence
        let has_greeting = input
            .last_greeting
            .map(|t| cycle_start.duration_since(t) > Duration::from_secs(300))
            .unwrap_or(true);
        let has_mission_complete = matches!(
            &input.mission_state,
            crate::types::MissionState::Completed { .. }
        );
        let has_milestone = !input.recent_milestones.is_empty();
        let has_reason = has_greeting || has_mission_complete || has_milestone;

        let silence_decision = self.silence.decide(&attention, has_reason, cycle_start);
        let is_silent = matches!(
            silence_decision,
            crate::silence::SilenceDecision::Silent { .. }
        );

        // 5. Generate outputs based on silence decision
        let greeting = if !is_silent && has_greeting {
            let time_ctx = crate::types::TimeContext::from_hour(input.now.hour());
            let is_return = matches!(
                &input.user_presence,
                UserPresence::Active if input.idle_duration > Duration::from_secs(120)
            );
            self.greeting.generate(
                time_ctx,
                input.weather,
                input.time_since_last_interaction,
                is_return,
                has_milestone,
                &input.recent_milestones,
                cycle_start,
            )
        } else {
            None
        };

        let micro_interaction = if !is_silent && greeting.is_none() {
            self.micro.generate(
                attention.focus_level,
                input.completed_tasks_today,
                input.pending_tasks,
                has_milestone,
                cycle_start,
            )
        } else {
            None
        };

        let memory_moment = if !is_silent && greeting.is_none() && micro_interaction.is_none() {
            self.memory.generate(
                &input.active_goals.join(" "),
                input.completed_tasks_today,
                &input.recent_milestones,
                cycle_start,
            )
        } else {
            None
        };

        // 6. Personality expression
        let activity_name = input
            .current_activity
            .map(|a| format!("{:?}", a))
            .unwrap_or_else(|| "Unknown".to_string());
        let expression = self.personality.express(
            attention.focus_level,
            &activity_name,
            presence_snapshot.energy,
        );

        // 7. Conversation pacing
        let pacing = if greeting.is_some() || micro_interaction.is_some() || memory_moment.is_some()
        {
            Some(self.conversation.calculate_pacing(
                attention.focus_level,
                input.current_activity,
                50,
            ))
        } else {
            None
        };

        // 8. Display text
        let display = greeting
            .as_ref()
            .map(|g| g.text.clone())
            .or_else(|| micro_interaction.as_ref().map(|m| m.text.clone()))
            .or_else(|| memory_moment.as_ref().map(|m| m.text.clone()));

        // 9. Record speech if we're displaying something
        if display.is_some() {
            self.silence.record_speech(cycle_start);
        }

        let update_latency_us = cycle_start.elapsed().as_micros() as u64;

        CompanionOutput {
            display,
            expression,
            presence_score: score_breakdown.total,
            pacing,
            greeting,
            micro_interaction,
            memory_moment,
            mission_state: input.mission_state.clone(),
            silence: is_silent,
            update_latency_us,
        }
    }

    /// Access the shared journey for recording milestones.
    pub fn journey(&self) -> &SharedJourney {
        &self.journey
    }

    /// Access the shared journey mutably for recording milestones.
    pub fn journey_mut(&mut self) -> &mut SharedJourney {
        &mut self.journey
    }

    /// Access the mission companion.
    pub fn mission(&self) -> &MissionCompanion {
        &self.mission
    }

    /// Access the mission companion mutably.
    pub fn mission_mut(&mut self) -> &mut MissionCompanion {
        &mut self.mission
    }

    /// Record a milestone in both journey and memory.
    pub fn record_milestone(&mut self, text: &str) {
        self.journey.record(text, MemoryKind::Milestone);
        self.memory.record(text, MemoryKind::Milestone, 0.9);
    }

    pub fn update_count(&self) -> u64 {
        self.update_count
    }

    pub fn presence_state(&self) -> PresenceState {
        self.presence.state()
    }

    pub fn personality(&self) -> &CompanionPersonality {
        &self.personality
    }

    pub fn config(&self) -> &CompanionConfig {
        &self.config
    }
}

impl Default for CompanionEngine {
    fn default() -> Self {
        Self::new(CompanionConfig::default(), CompanionPersonality::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SessionId, WeatherContext};
    use chrono::Utc;

    fn make_input() -> CompanionInput {
        CompanionInput {
            now: Utc::now(),
            session_id: SessionId::new(),
            user_presence: UserPresence::Active,
            current_activity: Some(crate::attention::ActivityKind::Coding),
            time_since_last_interaction: Duration::from_secs(600),
            conversation_count_this_session: 2,
            total_session_duration: Duration::from_secs(3600),
            active_goals: vec!["Build context engine".to_string()],
            recent_milestones: vec!["Context Fusion complete".to_string()],
            weather: WeatherContext::Clear,
            stress_estimate: 0.2,
            idle_duration: Duration::from_secs(10),
            pending_tasks: 3,
            completed_tasks_today: 5,
            last_greeting: None,
            last_micro_interaction: None,
            last_memory_reference: None,
            mission_state: crate::types::MissionState::Idle,
            focus_level: 0.7,
        }
    }

    #[test]
    fn test_engine_update() {
        let mut engine = CompanionEngine::default();
        let output = engine.update(&make_input());
        assert!(output.update_latency_us < 200);
        assert!(output.presence_score > 0.0);
    }

    #[test]
    fn test_engine_records_milestone() {
        let mut engine = CompanionEngine::default();
        engine.record_milestone("Test milestone");
        assert_eq!(engine.journey().entry_count(), 1);
    }

    #[test]
    fn test_engine_update_count() {
        let mut engine = CompanionEngine::default();
        engine.update(&make_input());
        engine.update(&make_input());
        assert_eq!(engine.update_count(), 2);
    }

    #[test]
    fn test_engine_latency() {
        let mut engine = CompanionEngine::default();
        let output = engine.update(&make_input());
        assert!(
            output.update_latency_us < 2000,
            "Update took {}us, target is <200us (generous <2ms for test)",
            output.update_latency_us
        );
    }
}
