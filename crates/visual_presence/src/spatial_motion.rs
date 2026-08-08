use crate::config::SpatialConfig;
use glam::Vec3;

pub struct SpatialMotion {
    config: SpatialConfig,
    cursor_pos: Vec3,
    target_pos: Vec3,
    velocity: Vec3,
    breathing_offset: f32,
    breathing_phase: f32,
}

impl SpatialMotion {
    pub fn new(config: SpatialConfig) -> Self {
        Self {
            config,
            cursor_pos: Vec3::ZERO,
            target_pos: Vec3::ZERO,
            velocity: Vec3::ZERO,
            breathing_offset: 0.0,
            breathing_phase: 0.0,
        }
    }

    pub fn update_cursor(&mut self, pos: Vec3) {
        self.cursor_pos = pos;
    }

    pub fn update(&mut self, delta_time: f32, is_speaking: bool, is_thinking: bool) {
        self.breathing_phase += delta_time * 0.8;

        let to_cursor = self.cursor_pos - self.target_pos;
        let dist = to_cursor.length();

        if dist < self.config.cursor_range {
            let influence = 1.0 - (dist / self.config.cursor_range);
            let attraction =
                to_cursor.normalize_or_zero() * influence * self.config.cursor_influence;
            self.target_pos += attraction * delta_time * 60.0;
        }

        if is_speaking {
            self.breathing_offset = self.breathing_phase.sin() * 0.3;
        } else if is_thinking {
            self.breathing_offset = (self.breathing_phase * 2.0).sin() * 0.2;
        } else {
            self.breathing_offset = self.breathing_phase.sin() * 0.1;
        }

        self.target_pos.y += self.breathing_offset;

        let desired_velocity = (self.cursor_pos - self.target_pos) * self.config.drift_amount;
        self.velocity = self.velocity.lerp(desired_velocity, delta_time * 10.0);

        self.target_pos += self.velocity * delta_time;

        self.target_pos.x = self.target_pos.x.clamp(-300.0, 300.0);
        self.target_pos.y = self.target_pos.y.clamp(-300.0, 300.0);
        self.target_pos.z = self.target_pos.z.clamp(-100.0, 100.0);
    }

    pub fn position(&self) -> Vec3 {
        self.target_pos
    }

    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }

    pub fn breathing_offset(&self) -> f32 {
        self.breathing_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_motion_creation() {
        let config = SpatialConfig::default();
        let motion = SpatialMotion::new(config);
        assert_eq!(motion.position(), Vec3::ZERO);
    }

    #[test]
    fn test_cursor_update() {
        let config = SpatialConfig::default();
        let mut motion = SpatialMotion::new(config);
        motion.update_cursor(Vec3::new(100.0, 50.0, 0.0));
        assert_eq!(motion.cursor_pos, Vec3::new(100.0, 50.0, 0.0));
    }

    #[test]
    fn test_breathing_update() {
        let config = SpatialConfig::default();
        let mut motion = SpatialMotion::new(config);
        motion.update(0.016, false, false);
        assert!(motion.breathing_offset().abs() < 1.0);
    }

    #[test]
    fn test_position_clamping() {
        let config = SpatialConfig::default();
        let mut motion = SpatialMotion::new(config);
        motion.target_pos = Vec3::new(500.0, 500.0, 0.0);
        motion.update(0.016, false, false);
        assert!(motion.position().x <= 300.0);
        assert!(motion.position().y <= 300.0);
    }
}
