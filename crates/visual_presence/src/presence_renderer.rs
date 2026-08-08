use crate::config::VisualPresenceConfig;
use crate::error::Result;
use crate::particle_engine::ParticleEngine;
use glam::Vec3;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PresenceState {
    Idle,
    Listening,
    Thinking,
    Speaking,
    Error,
}

impl PresenceState {
    pub fn to_color(self) -> [f32; 4] {
        match self {
            Self::Idle => [0.4, 0.6, 1.0, 0.8],
            Self::Listening => [0.2, 0.8, 0.4, 0.9],
            Self::Thinking => [0.8, 0.6, 0.2, 0.85],
            Self::Speaking => [0.9, 0.3, 0.3, 0.95],
            Self::Error => [1.0, 0.2, 0.2, 0.7],
        }
    }

    pub fn to_glow(self) -> f32 {
        match self {
            Self::Idle => 0.6,
            Self::Listening => 1.2,
            Self::Thinking => 1.0,
            Self::Speaking => 1.4,
            Self::Error => 0.8,
        }
    }

    pub fn to_cohesion(self) -> f32 {
        match self {
            Self::Idle => 0.3,
            Self::Listening => 0.7,
            Self::Thinking => 0.9,
            Self::Speaking => 0.5,
            Self::Error => 0.2,
        }
    }

    pub fn to_expansion(self) -> f32 {
        match self {
            Self::Idle => 1.0,
            Self::Listening => 1.2,
            Self::Thinking => 0.8,
            Self::Speaking => 1.5,
            Self::Error => 0.6,
        }
    }
}

pub struct PresenceRenderer {
    particle_engine: ParticleEngine,
    state: PresenceState,
    target_state: PresenceState,
    transition_progress: f32,
    config: VisualPresenceConfig,
}

impl PresenceRenderer {
    pub async fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: VisualPresenceConfig,
    ) -> Result<Self> {
        let particle_engine = ParticleEngine::new(device, queue, config.particle.clone()).await?;

        Ok(Self {
            particle_engine,
            state: PresenceState::Idle,
            target_state: PresenceState::Idle,
            transition_progress: 1.0,
            config,
        })
    }

    pub fn set_state(&mut self, state: PresenceState) {
        if state != self.target_state {
            self.target_state = state;
            self.transition_progress = 0.0;
        }
    }

    pub fn update(&mut self, delta_time: f32, mouse_pos: Option<Vec3>) {
        if self.transition_progress < 1.0 {
            self.transition_progress = (self.transition_progress
                + delta_time * self.config.animation.blend_speed)
                .min(1.0);

            if self.transition_progress >= 1.0 {
                self.state = self.target_state;
            }
        }

        let target = self.target_state;
        let current = self.state;

        let cohesion = current.to_cohesion() * (1.0 - self.transition_progress)
            + target.to_cohesion() * self.transition_progress;
        let expansion = current.to_expansion() * (1.0 - self.transition_progress)
            + target.to_expansion() * self.transition_progress;
        let glow = current.to_glow() * (1.0 - self.transition_progress)
            + target.to_glow() * self.transition_progress;

        let center = mouse_pos.unwrap_or(Vec3::new(0.0, 0.0, 0.0));

        self.particle_engine
            .update(delta_time, center, cohesion, expansion, glow);
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.particle_engine.render(render_pass);
    }

    pub fn state(&self) -> PresenceState {
        self.state
    }

    pub fn target_state(&self) -> PresenceState {
        self.target_state
    }

    pub fn transition_progress(&self) -> f32 {
        self.transition_progress
    }

    pub fn particle_engine(&self) -> &ParticleEngine {
        &self.particle_engine
    }

    pub fn config(&self) -> &VisualPresenceConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_state_colors() {
        let idle = PresenceState::Idle.to_color();
        assert_eq!(idle[0], 0.4);
        assert_eq!(idle[1], 0.6);
        assert_eq!(idle[2], 1.0);

        let speaking = PresenceState::Speaking.to_color();
        assert!(speaking[0] > 0.8);
    }

    #[test]
    fn test_presence_state_transitions() {
        assert_ne!(PresenceState::Idle, PresenceState::Listening);
        assert_ne!(PresenceState::Thinking, PresenceState::Speaking);
    }

    #[test]
    fn test_state_to_glow() {
        assert!(PresenceState::Speaking.to_glow() > PresenceState::Idle.to_glow());
    }

    #[test]
    fn test_state_to_cohesion() {
        assert!(PresenceState::Thinking.to_cohesion() > PresenceState::Idle.to_cohesion());
    }

    #[test]
    fn test_state_to_expansion() {
        assert!(PresenceState::Speaking.to_expansion() > PresenceState::Idle.to_expansion());
    }
}
