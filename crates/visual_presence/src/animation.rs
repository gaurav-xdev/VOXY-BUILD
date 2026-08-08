use crate::config::AnimationConfig;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    Idle,
    Listening,
    Thinking,
    Speaking,
    Transitioning,
}

pub struct AnimationGraph {
    config: AnimationConfig,
    current_state: AnimationState,
    target_state: AnimationState,
    blend_progress: f32,
    breath_phase: f32,
    sway_phase: f32,
    idle_time: f32,
}

impl AnimationGraph {
    pub fn new(config: AnimationConfig) -> Self {
        Self {
            config,
            current_state: AnimationState::Idle,
            target_state: AnimationState::Idle,
            blend_progress: 1.0,
            breath_phase: 0.0,
            sway_phase: 0.0,
            idle_time: 0.0,
        }
    }

    pub fn transition_to(&mut self, state: AnimationState) {
        if state != self.target_state {
            self.target_state = state;
            self.blend_progress = 0.0;
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.breath_phase += delta_time * self.config.breathing_rate;
        self.sway_phase += delta_time * 0.5;
        self.idle_time += delta_time;

        if self.blend_progress < 1.0 {
            self.blend_progress = (self.blend_progress
                + delta_time / (self.config.transition_duration_ms as f32 / 1000.0))
                .min(1.0);

            if self.blend_progress >= 1.0 {
                self.current_state = self.target_state;
            }
        }
    }

    pub fn breathing_offset(&self) -> f32 {
        let base = self.breath_phase.sin() * self.config.breathing_amplitude;
        match self.current_state {
            AnimationState::Idle => base,
            AnimationState::Listening => base * 1.2,
            AnimationState::Thinking => base * 0.8,
            AnimationState::Speaking => base * 1.5,
            AnimationState::Transitioning => base,
        }
    }

    pub fn sway_offset(&self) -> f32 {
        self.sway_phase.sin() * self.config.idle_sway_amount
    }

    pub fn current_state(&self) -> AnimationState {
        self.current_state
    }

    pub fn target_state(&self) -> AnimationState {
        self.target_state
    }

    pub fn blend_progress(&self) -> f32 {
        self.blend_progress
    }

    pub fn is_transitioning(&self) -> bool {
        self.blend_progress < 1.0
    }

    pub fn idle_time(&self) -> f32 {
        self.idle_time
    }

    pub fn reset_idle_timer(&mut self) {
        self.idle_time = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_graph_creation() {
        let config = AnimationConfig::default();
        let graph = AnimationGraph::new(config);
        assert_eq!(graph.current_state(), AnimationState::Idle);
        assert!(!graph.is_transitioning());
    }

    #[test]
    fn test_transition() {
        let config = AnimationConfig::default();
        let mut graph = AnimationGraph::new(config);
        graph.transition_to(AnimationState::Listening);
        assert!(graph.is_transitioning());
        assert_eq!(graph.target_state(), AnimationState::Listening);
    }

    #[test]
    fn test_breathing_offset() {
        let config = AnimationConfig::default();
        let mut graph = AnimationGraph::new(config);
        graph.update(0.016);
        let offset = graph.breathing_offset();
        assert!(offset.abs() < 1.0);
    }

    #[test]
    fn test_sway_offset() {
        let config = AnimationConfig::default();
        let mut graph = AnimationGraph::new(config);
        graph.update(0.016);
        let offset = graph.sway_offset();
        assert!(offset.abs() < 1.0);
    }

    #[test]
    fn test_idle_timer() {
        let config = AnimationConfig::default();
        let mut graph = AnimationGraph::new(config);
        graph.update(1.0);
        assert!(graph.idle_time() > 0.9);
        graph.reset_idle_timer();
        assert_eq!(graph.idle_time(), 0.0);
    }
}
