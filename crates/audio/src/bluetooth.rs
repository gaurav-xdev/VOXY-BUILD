//! Bluetooth audio strategy for the voice pipeline.
//!
//! Provides Bluetooth device management:
//! - Profile detection (A2DP, HSP, HFP)
//! - Quality preference and codec selection
//! - Audio routing when Bluetooth profiles change
//! - Latency and bandwidth awareness
//!
//! All platform-specific code is isolated behind `#[cfg(windows)]`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tokio::sync::mpsc;

/// Bluetooth audio profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BluetoothProfile {
    /// Advanced Audio Distribution Profile (stereo music).
    A2dp,
    /// Headset Profile (mono voice, bidirectional).
    HspHandsFree,
    /// Hands-Free Profile (enhanced voice with wider bandwidth).
    HfpHandsFree,
    /// Low Energy Audio (new in Bluetooth 5.2+).
    LeAudio,
    /// Unknown or unsupported profile.
    Unknown,
}

impl BluetoothProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::A2dp => "A2DP",
            Self::HspHandsFree => "HSP",
            Self::HfpHandsFree => "HFP",
            Self::LeAudio => "LE Audio",
            Self::Unknown => "Unknown",
        }
    }

    /// Whether this profile supports bidirectional audio.
    pub fn is_bidirectional(&self) -> bool {
        matches!(
            self,
            Self::HspHandsFree | Self::HfpHandsFree | Self::LeAudio
        )
    }

    /// Typical sample rate for this profile.
    pub fn typical_sample_rate(&self) -> u32 {
        match self {
            Self::A2dp => 44100,
            Self::HspHandsFree => 16000,
            Self::HfpHandsFree => 8000,
            Self::LeAudio => 48000,
            Self::Unknown => 16000,
        }
    }

    /// Typical latency (ms) for this profile.
    pub fn typical_latency_ms(&self) -> u32 {
        match self {
            Self::A2dp => 150,
            Self::HspHandsFree => 100,
            Self::HfpHandsFree => 120,
            Self::LeAudio => 30,
            Self::Unknown => 150,
        }
    }
}

/// Bluetooth audio codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BluetoothCodec {
    /// Sub-Band Codec (standard A2DP).
    Sbc,
    /// Low Complexity Sub-Band Codec (A2DP optional).
    Ldac,
    /// Apple proprietary (not available on Windows typically).
    Aac,
    /// Lossless (aptX Lossless).
    AptxLossless,
    /// Low Energy Audio codec.
    Lc3,
    /// Unknown codec.
    Unknown,
}

impl BluetoothCodec {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sbc => "SBC",
            Self::Ldac => "LDAC",
            Self::Aac => "AAC",
            Self::AptxLossless => "aptX Lossless",
            Self::Lc3 => "LC3",
            Self::Unknown => "Unknown",
        }
    }

    /// Bandwidth efficiency score (0-100).
    pub fn bandwidth_score(&self) -> u32 {
        match self {
            Self::AptxLossless => 100,
            Self::Ldac => 90,
            Self::Lc3 => 85,
            Self::Aac => 70,
            Self::Sbc => 50,
            Self::Unknown => 40,
        }
    }
}

/// Bluetooth device information.
#[derive(Debug, Clone)]
pub struct BluetoothDeviceInfo {
    /// Device name.
    pub name: String,
    /// Device address (MAC or identifier).
    pub address: String,
    /// Active profile.
    pub profile: BluetoothProfile,
    /// Current codec (if A2DP).
    pub codec: Option<BluetoothCodec>,
    /// Battery level (0-100) if available.
    pub battery_level: Option<u32>,
    /// Signal strength (0-100) if available.
    pub signal_strength: Option<u32>,
    /// Connection quality score (0-100).
    pub quality_score: u32,
    /// Latency estimate (ms).
    pub latency_ms: u32,
    /// Whether this device is currently active for audio.
    pub is_active: bool,
    /// Timestamp of last update.
    pub last_seen: Instant,
}

/// Quality preference for Bluetooth audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothQualityPreference {
    /// Prefer highest quality regardless of latency.
    HighQuality,
    /// Balance quality and latency.
    Balanced,
    /// Prefer lowest latency for voice applications.
    LowLatency,
    /// Prefer lowest power consumption.
    PowerSaving,
}

/// Events from Bluetooth manager.
#[derive(Debug, Clone)]
pub enum BluetoothEvent {
    /// Device connected.
    DeviceConnected {
        name: String,
        profile: BluetoothProfile,
    },
    /// Device disconnected.
    DeviceDisconnected { name: String },
    /// Profile changed on an active device.
    ProfileChanged {
        name: String,
        old_profile: BluetoothProfile,
        new_profile: BluetoothProfile,
    },
    /// Quality score changed.
    QualityChanged { name: String, new_score: u32 },
    /// Recommended routing changed.
    RoutingChanged { from: String, to: String },
}

/// Configuration for Bluetooth strategy.
#[derive(Debug, Clone)]
pub struct BluetoothStrategyConfig {
    /// Quality preference.
    pub quality_preference: BluetoothQualityPreference,
    /// Profiles preferred for voice (in priority order).
    pub voice_profiles: Vec<BluetoothProfile>,
    /// Profiles preferred for music (in priority order).
    pub music_profiles: Vec<BluetoothProfile>,
    /// Minimum quality score to consider device usable.
    pub min_quality_score: u32,
    /// Maximum acceptable latency for voice (ms).
    pub max_voice_latency_ms: u32,
    /// Polling interval for quality monitoring.
    pub quality_poll_interval: Duration,
    /// Whether to automatically reroute on profile change.
    pub auto_reroute: bool,
}

impl Default for BluetoothStrategyConfig {
    fn default() -> Self {
        Self {
            quality_preference: BluetoothQualityPreference::Balanced,
            voice_profiles: vec![
                BluetoothProfile::LeAudio,
                BluetoothProfile::HfpHandsFree,
                BluetoothProfile::HspHandsFree,
            ],
            music_profiles: vec![BluetoothProfile::A2dp, BluetoothProfile::LeAudio],
            min_quality_score: 50,
            max_voice_latency_ms: 200,
            quality_poll_interval: Duration::from_secs(5),
            auto_reroute: true,
        }
    }
}

/// Trait for platform-agnostic Bluetooth management.
#[async_trait::async_trait]
pub trait BluetoothManager: Send + Sync {
    /// Initialize the Bluetooth manager.
    async fn initialize(&self, config: &BluetoothStrategyConfig) -> Result<(), String>;

    /// Scan for Bluetooth devices.
    async fn scan_devices(&self) -> Result<Vec<BluetoothDeviceInfo>, String>;

    /// Get the currently active Bluetooth audio device.
    async fn active_device(&self) -> Option<BluetoothDeviceInfo>;

    /// Get device quality score.
    async fn device_quality(&self, address: &str) -> Option<u32>;

    /// Check if a device supports the given profile.
    async fn supports_profile(
        &self,
        address: &str,
        profile: BluetoothProfile,
    ) -> Result<bool, String>;

    /// Recommend the best device for voice calls.
    async fn recommend_voice_device(&self) -> Option<BluetoothDeviceInfo>;

    /// Recommend the best device for music playback.
    async fn recommend_music_device(&self) -> Option<BluetoothDeviceInfo>;

    /// Take the event receiver.
    async fn take_events(&self) -> Option<mpsc::Receiver<BluetoothEvent>>;
}

/// In-memory (mock) Bluetooth manager for testing.
pub struct InMemoryBluetoothManager {
    initialized: AtomicBool,
    config: RwLock<Option<BluetoothStrategyConfig>>,
    devices: RwLock<Vec<BluetoothDeviceInfo>>,
    #[allow(dead_code)]
    event_tx: mpsc::Sender<BluetoothEvent>,
    event_rx: RwLock<Option<mpsc::Receiver<BluetoothEvent>>>,
}

impl InMemoryBluetoothManager {
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel(64);
        Self {
            initialized: AtomicBool::new(false),
            config: RwLock::new(None),
            devices: RwLock::new(Vec::new()),
            event_tx,
            event_rx: RwLock::new(Some(event_rx)),
        }
    }

    /// Add a mock device for testing.
    pub fn add_device(&self, device: BluetoothDeviceInfo) {
        self.devices.write().push(device);
    }
}

impl Default for InMemoryBluetoothManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl BluetoothManager for InMemoryBluetoothManager {
    async fn initialize(&self, config: &BluetoothStrategyConfig) -> Result<(), String> {
        self.initialized.store(true, Ordering::SeqCst);
        *self.config.write() = Some(config.clone());
        Ok(())
    }

    async fn scan_devices(&self) -> Result<Vec<BluetoothDeviceInfo>, String> {
        Ok(self.devices.read().clone())
    }

    async fn active_device(&self) -> Option<BluetoothDeviceInfo> {
        self.devices.read().iter().find(|d| d.is_active).cloned()
    }

    async fn device_quality(&self, address: &str) -> Option<u32> {
        self.devices
            .read()
            .iter()
            .find(|d| d.address == address)
            .map(|d| d.quality_score)
    }

    async fn supports_profile(
        &self,
        address: &str,
        profile: BluetoothProfile,
    ) -> Result<bool, String> {
        let devices = self.devices.read();
        let device = devices
            .iter()
            .find(|d| d.address == address)
            .ok_or_else(|| format!("Device not found: {}", address))?;
        Ok(device.profile == profile)
    }

    async fn recommend_voice_device(&self) -> Option<BluetoothDeviceInfo> {
        let config = self.config.read();
        let cfg = config.as_ref()?;
        let devices = self.devices.read();

        // Find best bidirectional device that meets latency requirements
        let mut best: Option<&BluetoothDeviceInfo> = None;
        let mut best_score = 0u32;

        for device in devices.iter() {
            if !device.profile.is_bidirectional() {
                continue;
            }
            if device.latency_ms > cfg.max_voice_latency_ms {
                continue;
            }
            if device.quality_score < cfg.min_quality_score {
                continue;
            }

            let score = device.quality_score + (100 - device.latency_ms.min(100));
            if score > best_score {
                best_score = score;
                best = Some(device);
            }
        }

        best.cloned()
    }

    async fn recommend_music_device(&self) -> Option<BluetoothDeviceInfo> {
        let devices = self.devices.read();

        // Find highest quality A2DP/LE Audio device
        devices
            .iter()
            .filter(|d| !d.profile.is_bidirectional() || d.profile == BluetoothProfile::LeAudio)
            .max_by_key(|d| d.quality_score)
            .cloned()
    }

    async fn take_events(&self) -> Option<mpsc::Receiver<BluetoothEvent>> {
        self.event_rx.write().take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bt_device(
        name: &str,
        address: &str,
        profile: BluetoothProfile,
        quality: u32,
        latency_ms: u32,
        active: bool,
    ) -> BluetoothDeviceInfo {
        BluetoothDeviceInfo {
            name: name.to_string(),
            address: address.to_string(),
            profile,
            codec: None,
            battery_level: Some(80),
            signal_strength: Some(90),
            quality_score: quality,
            latency_ms,
            is_active: active,
            last_seen: Instant::now(),
        }
    }

    #[test]
    fn bluetooth_profile_basics() {
        assert_eq!(BluetoothProfile::A2dp.name(), "A2DP");
        assert!(!BluetoothProfile::A2dp.is_bidirectional());
        assert!(BluetoothProfile::HfpHandsFree.is_bidirectional());
        assert!(BluetoothProfile::LeAudio.is_bidirectional());
        assert_eq!(BluetoothProfile::A2dp.typical_sample_rate(), 44100);
    }

    #[test]
    fn bluetooth_codec_basics() {
        assert_eq!(BluetoothCodec::Sbc.name(), "SBC");
        assert!(
            BluetoothCodec::AptxLossless.bandwidth_score() > BluetoothCodec::Sbc.bandwidth_score()
        );
    }

    #[test]
    fn bluetooth_config_defaults() {
        let config = BluetoothStrategyConfig::default();
        assert_eq!(
            config.quality_preference,
            BluetoothQualityPreference::Balanced
        );
        assert!(config.voice_profiles.contains(&BluetoothProfile::LeAudio));
        assert!(config.music_profiles.contains(&BluetoothProfile::A2dp));
    }

    #[tokio::test]
    async fn bt_manager_init() {
        let mgr = InMemoryBluetoothManager::new();
        assert!(!mgr.initialized.load(Ordering::SeqCst));

        let config = BluetoothStrategyConfig::default();
        mgr.initialize(&config).await.unwrap();
        assert!(mgr.initialized.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn bt_scan_devices() {
        let mgr = InMemoryBluetoothManager::new();
        let config = BluetoothStrategyConfig::default();
        mgr.initialize(&config).await.unwrap();

        mgr.add_device(make_bt_device(
            "Jabra Elite",
            "AA:BB:CC",
            BluetoothProfile::HfpHandsFree,
            85,
            100,
            true,
        ));
        mgr.add_device(make_bt_device(
            "Sony WH-1000",
            "DD:EE:FF",
            BluetoothProfile::A2dp,
            90,
            150,
            false,
        ));

        let devices = mgr.scan_devices().await.unwrap();
        assert_eq!(devices.len(), 2);
    }

    #[tokio::test]
    async fn bt_active_device() {
        let mgr = InMemoryBluetoothManager::new();
        let config = BluetoothStrategyConfig::default();
        mgr.initialize(&config).await.unwrap();

        mgr.add_device(make_bt_device(
            "Jabra Elite",
            "AA:BB:CC",
            BluetoothProfile::HfpHandsFree,
            85,
            100,
            true,
        ));

        let active = mgr.active_device().await.unwrap();
        assert_eq!(active.name, "Jabra Elite");
    }

    #[tokio::test]
    async fn bt_no_active_device() {
        let mgr = InMemoryBluetoothManager::new();
        let config = BluetoothStrategyConfig::default();
        mgr.initialize(&config).await.unwrap();

        assert!(mgr.active_device().await.is_none());
    }

    #[tokio::test]
    async fn bt_device_quality() {
        let mgr = InMemoryBluetoothManager::new();
        let config = BluetoothStrategyConfig::default();
        mgr.initialize(&config).await.unwrap();

        mgr.add_device(make_bt_device(
            "Jabra Elite",
            "AA:BB:CC",
            BluetoothProfile::HfpHandsFree,
            85,
            100,
            true,
        ));

        let quality = mgr.device_quality("AA:BB:CC").await;
        assert_eq!(quality, Some(85));

        let quality = mgr.device_quality("XX:YY:ZZ").await;
        assert_eq!(quality, None);
    }

    #[tokio::test]
    async fn bt_supports_profile() {
        let mgr = InMemoryBluetoothManager::new();
        let config = BluetoothStrategyConfig::default();
        mgr.initialize(&config).await.unwrap();

        mgr.add_device(make_bt_device(
            "Jabra Elite",
            "AA:BB:CC",
            BluetoothProfile::HfpHandsFree,
            85,
            100,
            true,
        ));

        assert!(mgr
            .supports_profile("AA:BB:CC", BluetoothProfile::HfpHandsFree)
            .await
            .unwrap());
        assert!(!mgr
            .supports_profile("AA:BB:CC", BluetoothProfile::A2dp)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn bt_recommend_voice_device() {
        let mgr = InMemoryBluetoothManager::new();
        let config = BluetoothStrategyConfig::default();
        mgr.initialize(&config).await.unwrap();

        // Good voice device
        mgr.add_device(make_bt_device(
            "Jabra Elite",
            "AA:BB:CC",
            BluetoothProfile::HfpHandsFree,
            85,
            100,
            false,
        ));
        // A2DP only - should not be recommended for voice
        mgr.add_device(make_bt_device(
            "Sony WH-1000",
            "DD:EE:FF",
            BluetoothProfile::A2dp,
            90,
            150,
            false,
        ));
        // Good LE Audio device
        mgr.add_device(make_bt_device(
            "AirPods Pro",
            "11:22:33",
            BluetoothProfile::LeAudio,
            80,
            30,
            false,
        ));

        let recommended = mgr.recommend_voice_device().await;
        assert!(recommended.is_some());
        let dev = recommended.unwrap();
        // Should prefer low latency LeAudio or HFP
        assert!(dev.profile.is_bidirectional());
        assert!(dev.latency_ms <= config.max_voice_latency_ms);
    }

    #[tokio::test]
    async fn bt_recommend_music_device() {
        let mgr = InMemoryBluetoothManager::new();
        let config = BluetoothStrategyConfig::default();
        mgr.initialize(&config).await.unwrap();

        mgr.add_device(make_bt_device(
            "Jabra Elite",
            "AA:BB:CC",
            BluetoothProfile::HfpHandsFree,
            85,
            100,
            false,
        ));
        mgr.add_device(make_bt_device(
            "Sony WH-1000",
            "DD:EE:FF",
            BluetoothProfile::A2dp,
            90,
            150,
            false,
        ));

        let recommended = mgr.recommend_music_device().await;
        assert!(recommended.is_some());
        assert_eq!(recommended.unwrap().name, "Sony WH-1000");
    }

    #[tokio::test]
    async fn bt_supports_profile_not_found() {
        let mgr = InMemoryBluetoothManager::new();
        let config = BluetoothStrategyConfig::default();
        mgr.initialize(&config).await.unwrap();

        let result = mgr
            .supports_profile("nonexistent", BluetoothProfile::HfpHandsFree)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn bt_take_events() {
        let mgr = InMemoryBluetoothManager::new();
        let rx = mgr.take_events().await;
        assert!(rx.is_some());
        assert!(mgr.take_events().await.is_none());
    }
}
