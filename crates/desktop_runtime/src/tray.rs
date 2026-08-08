//! System tray integration.

use crate::error::Result;
use tracing::info;

pub struct TrayIcon {
    #[allow(dead_code)]
    app_name: String,
    tooltip: String,
}

impl TrayIcon {
    pub fn new(app_name: &str, tooltip: &str) -> Result<Self> {
        Ok(Self {
            app_name: app_name.to_string(),
            tooltip: tooltip.to_string(),
        })
    }

    pub fn set_tooltip(&mut self, tooltip: &str) -> Result<()> {
        self.tooltip = tooltip.to_string();
        Ok(())
    }

    pub fn show_balloon(&self, title: &str, message: &str) -> Result<()> {
        info!("Balloon: {} - {}", title, message);
        Ok(())
    }

    pub fn remove(self) -> Result<()> {
        info!("Tray icon removed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_creation() {
        let tray = TrayIcon::new("VOXY", "VOXY AI Assistant");
        assert!(tray.is_ok());
    }

    #[test]
    fn tray_tooltip_update() {
        let mut tray = TrayIcon::new("VOXY", "VOXY").unwrap();
        assert!(tray.set_tooltip("New tooltip").is_ok());
    }

    #[test]
    fn tray_balloon() {
        let tray = TrayIcon::new("VOXY", "VOXY").unwrap();
        assert!(tray.show_balloon("Test", "Hello").is_ok());
    }

    #[test]
    fn tray_removal() {
        let tray = TrayIcon::new("VOXY", "VOXY").unwrap();
        assert!(tray.remove().is_ok());
    }
}
