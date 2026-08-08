#[derive(Debug, Clone)]
pub struct AudioStreamConfig {
    pub sample_rate: u32,
    pub channels: u8,
    pub bit_depth: u8,
    pub buffer_size_frames: usize,
    pub device_id: Option<String>,
    pub latency_hint_ms: f64,
}

impl Default for AudioStreamConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bit_depth: 16,
            buffer_size_frames: 1600,
            device_id: None,
            latency_hint_ms: 100.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioRuntimeConfig {
    pub input: AudioStreamConfig,
    pub output: AudioStreamConfig,
    pub enable_diagnostics: bool,
    pub diagnostics_interval_ms: u64,
    pub enable_dsp: bool,
    pub dsp_chain: Vec<String>,
    pub platform: String,
}

impl Default for AudioRuntimeConfig {
    fn default() -> Self {
        Self {
            input: AudioStreamConfig::default(),
            output: AudioStreamConfig::default(),
            enable_diagnostics: true,
            diagnostics_interval_ms: 1000,
            enable_dsp: false,
            dsp_chain: Vec::new(),
            platform: String::new(),
        }
    }
}
