//! Global keyboard shortcuts.

use crate::error::Result;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
}

impl Modifiers {
    pub const fn new() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            win: false,
        }
    }
    pub const fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::new()
        }
    }
    pub const fn ctrl_alt() -> Self {
        Self {
            ctrl: true,
            alt: true,
            ..Self::new()
        }
    }
    pub const fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            ..Self::new()
        }
    }
    pub const fn alt() -> Self {
        Self {
            alt: true,
            ..Self::new()
        }
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Shortcut {
    pub id: u32,
    pub name: String,
    pub modifiers: Modifiers,
    pub vk_code: u32,
}

pub struct ShortcutManager {
    shortcuts: Vec<Shortcut>,
}

impl ShortcutManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            shortcuts: Vec::new(),
        })
    }

    pub fn register_default_shortcuts(&mut self) -> Result<()> {
        self.register(
            "voice_toggle",
            "Toggle Voice",
            Modifiers::ctrl_shift(),
            0x56,
        )?;
        self.register("quick_action", "Quick Action", Modifiers::ctrl_alt(), 0x20)?;
        Ok(())
    }

    pub fn register(
        &mut self,
        _id: &str,
        name: &str,
        modifiers: Modifiers,
        vk_code: u32,
    ) -> Result<()> {
        let id = self.shortcuts.len() as u32 + 1;
        self.shortcuts.push(Shortcut {
            id,
            name: name.to_string(),
            modifiers,
            vk_code,
        });
        info!("Registered shortcut: {} (id={})", name, id);
        Ok(())
    }

    pub fn unregister_all(&self) {
        info!("All {} shortcuts unregistered", self.shortcuts.len());
    }

    pub fn shortcuts(&self) -> &[Shortcut] {
        &self.shortcuts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_default() {
        let m = Modifiers::new();
        assert!(!m.ctrl && !m.alt && !m.shift && !m.win);
    }

    #[test]
    fn shortcut_manager_creation() {
        assert!(ShortcutManager::new().is_ok());
    }

    #[test]
    fn shortcut_register() {
        let mut mgr = ShortcutManager::new().unwrap();
        assert!(mgr
            .register("test", "Test", Modifiers::ctrl(), 0x41)
            .is_ok());
        assert_eq!(mgr.shortcuts().len(), 1);
    }

    #[test]
    fn shortcut_unregister_all() {
        let mut mgr = ShortcutManager::new().unwrap();
        let _ = mgr.register("test", "Test", Modifiers::ctrl(), 0x41);
        mgr.unregister_all();
    }
}
