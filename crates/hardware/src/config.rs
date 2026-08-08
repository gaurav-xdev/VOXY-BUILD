#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u8,
    pub bit_depth: u8,
    pub buffer_size_ms: u32,
    pub device_id: Option<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bit_depth: 16,
            buffer_size_ms: 100,
            device_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub device_id: Option<String>,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            fps: 30,
            device_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HardwareConfig {
    pub audio_input: AudioConfig,
    pub audio_output: AudioConfig,
    pub video: VideoConfig,
    pub enable_hardware_monitoring: bool,
    pub monitor_interval_ms: u64,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            audio_input: AudioConfig::default(),
            audio_output: AudioConfig::default(),
            video: VideoConfig::default(),
            enable_hardware_monitoring: false,
            monitor_interval_ms: 5000,
        }
    }
}
