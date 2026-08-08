use async_trait::async_trait;

use crate::config::{AudioConfig, VideoConfig};
use crate::error::Result;

#[async_trait]
pub trait Microphone: Send + Sync {
    fn device_id(&self) -> &str;
    fn device_name(&self) -> &str;
    fn is_available(&self) -> bool;
    async fn open(&self, config: &AudioConfig) -> Result<()>;
    async fn close(&self) -> Result<()>;
    async fn read(&self, buffer: &mut [f32]) -> Result<usize>;
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u8;
}

#[async_trait]
pub trait Speaker: Send + Sync {
    fn device_id(&self) -> &str;
    fn device_name(&self) -> &str;
    fn is_available(&self) -> bool;
    async fn open(&self, config: &AudioConfig) -> Result<()>;
    async fn close(&self) -> Result<()>;
    async fn write(&self, buffer: &[f32]) -> Result<usize>;
    async fn flush(&self) -> Result<()>;
    async fn set_volume(&self, volume: f32) -> Result<()>;
    fn volume(&self) -> f32;
    fn sample_rate(&self) -> u32;
}

#[async_trait]
pub trait Camera: Send + Sync {
    fn device_id(&self) -> &str;
    fn device_name(&self) -> &str;
    fn is_available(&self) -> bool;
    async fn open(&self, config: &VideoConfig) -> Result<()>;
    async fn close(&self) -> Result<()>;
    async fn capture_frame(&self) -> Result<Vec<u8>>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

#[async_trait]
pub trait HardwareMonitor: Send + Sync {
    async fn check_audio_inputs(&self) -> Result<Vec<Box<dyn Microphone>>>;
    async fn check_audio_outputs(&self) -> Result<Vec<Box<dyn Speaker>>>;
    async fn check_cameras(&self) -> Result<Vec<Box<dyn Camera>>>;
    async fn has_audio_input(&self) -> bool;
    async fn has_audio_output(&self) -> bool;
    async fn has_camera(&self) -> bool;
    async fn default_microphone(&self) -> Option<Box<dyn Microphone>>;
    async fn default_speaker(&self) -> Option<Box<dyn Speaker>>;
    async fn default_camera(&self) -> Option<Box<dyn Camera>>;
}

#[derive(Debug, Clone)]
pub struct HardwareInfo {
    pub audio_inputs: Vec<String>,
    pub audio_outputs: Vec<String>,
    pub cameras: Vec<String>,
    pub audio_input_available: bool,
    pub audio_output_available: bool,
    pub camera_available: bool,
}
