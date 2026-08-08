//! Platform data types.

use serde::{Deserialize, Serialize};

/// Platform information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub version: String,
    pub hostname: Option<String>,
}

/// Window information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub process_name: String,
    pub process_id: u32,
    pub is_foreground: bool,
    pub bounds: WindowBounds,
}

/// Window bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Display information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
    pub dpi: f64,
}

/// Audio device information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_default: bool,
    pub sample_rate: u32,
    pub channels: u16,
}

/// File information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
}

/// Network information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub is_online: bool,
    pub interfaces: Vec<NetworkInterface>,
}

/// Network interface information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_addresses: Vec<String>,
    pub is_up: bool,
}

/// Process information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub executable_path: Option<String>,
}
