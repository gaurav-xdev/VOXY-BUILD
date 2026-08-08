use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisualPresenceConfig {
    pub particle: ParticleConfig,
    pub rendering: RenderingConfig,
    pub overlay: OverlayConfig,
    pub animation: AnimationConfig,
    pub audio_reactive: AudioReactiveConfig,
    pub head_tracking: HeadTrackingConfig,
    pub spatial: SpatialConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleConfig {
    pub min_particles: usize,
    pub max_particles: usize,
    pub particle_size: f32,
    pub base_speed: f32,
    pub turbulence: f32,
    pub glow_intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    pub target_fps: u32,
    pub max_cpu_percent: f32,
    pub max_gpu_percent: f32,
    pub vsync: bool,
    pub msaa_samples: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub width: u32,
    pub height: u32,
    pub always_on_top: bool,
    pub click_through: bool,
    pub dpi_aware: bool,
    pub monitor_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub blend_speed: f32,
    pub breathing_rate: f32,
    pub breathing_amplitude: f32,
    pub idle_sway_amount: f32,
    pub transition_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioReactiveConfig {
    pub enabled: bool,
    pub rms_smoothing: f32,
    pub glow_boost: f32,
    pub expansion_factor: f32,
    pub pulse_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadTrackingConfig {
    pub enabled: bool,
    pub camera_index: usize,
    pub tracking_confidence: f32,
    pub depth_scale: f32,
    pub smoothing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialConfig {
    pub cursor_influence: f32,
    pub cursor_range: f32,
    pub collapse_speed: f32,
    pub expand_speed: f32,
    pub drift_amount: f32,
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            min_particles: 5000,
            max_particles: 20000,
            particle_size: 2.0,
            base_speed: 0.5,
            turbulence: 0.3,
            glow_intensity: 0.8,
        }
    }
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            target_fps: 144,
            max_cpu_percent: 2.0,
            max_gpu_percent: 10.0,
            vsync: true,
            msaa_samples: 4,
        }
    }
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            width: 400,
            height: 400,
            always_on_top: false,
            click_through: true,
            dpi_aware: true,
            monitor_index: 0,
        }
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            blend_speed: 2.0,
            breathing_rate: 0.8,
            breathing_amplitude: 0.1,
            idle_sway_amount: 0.05,
            transition_duration_ms: 500,
        }
    }
}

impl Default for AudioReactiveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rms_smoothing: 0.8,
            glow_boost: 1.5,
            expansion_factor: 1.3,
            pulse_speed: 2.0,
        }
    }
}

impl Default for HeadTrackingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            camera_index: 0,
            tracking_confidence: 0.7,
            depth_scale: 1.0,
            smoothing: 0.3,
        }
    }
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            cursor_influence: 0.3,
            cursor_range: 200.0,
            collapse_speed: 3.0,
            expand_speed: 2.0,
            drift_amount: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = VisualPresenceConfig::default();
        assert_eq!(config.particle.min_particles, 5000);
        assert_eq!(config.particle.max_particles, 20000);
        assert_eq!(config.rendering.target_fps, 144);
        assert!(!config.head_tracking.enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = VisualPresenceConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: VisualPresenceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(
            config.particle.min_particles,
            deserialized.particle.min_particles
        );
    }
}
