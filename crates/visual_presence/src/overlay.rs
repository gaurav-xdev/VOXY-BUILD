use crate::config::OverlayConfig;
use crate::error::Result;

pub struct DesktopOverlay {
    config: OverlayConfig,
    width: u32,
    height: u32,
    always_on_top: bool,
    click_through: bool,
}

impl DesktopOverlay {
    pub fn new(config: OverlayConfig) -> Result<Self> {
        let width = config.width;
        let height = config.height;
        let always_on_top = config.always_on_top;
        let click_through = config.click_through;

        Ok(Self {
            config,
            width,
            height,
            always_on_top,
            click_through,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn is_always_on_top(&self) -> bool {
        self.always_on_top
    }

    pub fn set_always_on_top(&mut self, enable: bool) {
        self.always_on_top = enable;
    }

    pub fn is_click_through(&self) -> bool {
        self.click_through
    }

    pub fn set_click_through(&mut self, enable: bool) {
        self.click_through = enable;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn config(&self) -> &OverlayConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_creation() {
        let config = OverlayConfig::default();
        let overlay = DesktopOverlay::new(config).unwrap();
        assert_eq!(overlay.width(), 400);
        assert_eq!(overlay.height(), 400);
        assert!(overlay.is_click_through());
    }

    #[test]
    fn test_overlay_resize() {
        let config = OverlayConfig::default();
        let mut overlay = DesktopOverlay::new(config).unwrap();
        overlay.resize(800, 600);
        assert_eq!(overlay.width(), 800);
        assert_eq!(overlay.height(), 600);
    }

    #[test]
    fn test_overlay_click_through_toggle() {
        let config = OverlayConfig::default();
        let mut overlay = DesktopOverlay::new(config).unwrap();
        overlay.set_click_through(false);
        assert!(!overlay.is_click_through());
    }
}
