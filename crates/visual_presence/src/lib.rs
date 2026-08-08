pub mod animation;
pub mod audio_reactive;
pub mod config;
pub mod error;
pub mod event_integration;
pub mod head_tracking;
pub mod overlay;
pub mod particle_engine;
pub mod presence_renderer;
pub mod spatial_motion;

pub use config::VisualPresenceConfig;
pub use error::{Result, VisualPresenceError};
pub use overlay::DesktopOverlay;
pub use particle_engine::ParticleEngine;
pub use presence_renderer::PresenceRenderer;
pub use spatial_motion::SpatialMotion;

pub mod prelude {
    pub use crate::config::VisualPresenceConfig;
    pub use crate::error::{Result, VisualPresenceError};
    pub use crate::overlay::DesktopOverlay;
    pub use crate::particle_engine::ParticleEngine;
    pub use crate::presence_renderer::PresenceRenderer;
    pub use crate::spatial_motion::SpatialMotion;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_compiles() {
        let _config = VisualPresenceConfig::default();
    }
}
