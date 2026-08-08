pub mod ambient;
pub mod backchannel;
pub mod backend;
pub mod benchmarks;
pub mod bluetooth;
pub mod buffer;
pub mod calibration;
pub mod config;
pub mod device;
pub mod device_watcher;
pub mod diagnostics;
pub mod dsp;
pub mod endurance;
pub mod error;
pub mod gpu_dsp;
pub mod hot_swap;
pub mod latency_predictor;
pub mod metrics;
pub mod mixer;
pub mod scheduler;
pub mod spectral_denoiser;
pub mod speech_eq;
pub mod stream;
pub mod streaming_stt;
#[cfg(test)]
pub mod stress_tests;
pub mod thread_priority;
pub mod turn_detector;
pub mod voice_memory;
pub mod volume_adapter;
pub mod wasapi_improvements;
pub mod wasapi_session;
pub mod watchdog;

#[cfg(windows)]
pub mod wasapi;

pub use ambient::{AmbientAnalyzer, AmbientEnvironment};
pub use backchannel::BackchannelGenerator;
pub use backend::{AudioBackend, FallbackBackend};
pub use bluetooth::{
    BluetoothCodec, BluetoothDeviceInfo, BluetoothEvent, BluetoothManager, BluetoothProfile,
    BluetoothQualityPreference, BluetoothStrategyConfig, InMemoryBluetoothManager,
};
pub use buffer::{AudioBufferPool, RingBuffer, SpscRingBuffer};
pub use calibration::{CalibrationProfile, SelfCalibrator};
pub use config::{AudioRuntimeConfig, AudioStreamConfig};
pub use device::{AudioDeviceInfo, AudioDeviceManager, InMemoryDeviceManager};
pub use device_watcher::{DeviceChangeEvent, DeviceChangeWatcher, DeviceWatcherHandle};
pub use diagnostics::{
    AudioDiagnostics, AudioMetrics, Histogram, InMemoryDiagnostics, ResourceTracker,
    StageTimingCollector,
};
pub use dsp::{
    DspChain, DspProcessor, EchoCancellationProcessor, GainProcessor, NoiseGateProcessor,
    NoiseSuppressionProcessor, Normalizer, Resampler, SilenceDetector,
};
pub use error::{AudioError, Result};
pub use gpu_dsp::{AdaptiveNoiseSuppressor, GpuDspBackend, SpectralEchoCanceller};
pub use hot_swap::{
    HotSwapConfig, HotSwapEvent, HotSwapHandler, HotSwapManager, NoopHotSwapHandler, PipelineState,
    RecoveryAction,
};
pub use latency_predictor::{LatencyAverages, LatencyDegradation, LatencyPredictor};
pub use metrics::{
    AudioQualityMetrics, ChannelMetrics, DetectionMetrics, LatencyMetrics, MetricsCollector,
    SystemMetrics, VoiceEngineMetrics,
};
pub use mixer::{AudioMixer, ChannelState, DuckingPriority, MixerChannel};
pub use scheduler::{AiAudioScheduler, QualityMode, SchedulerThresholds, SystemSnapshot};
pub use spectral_denoiser::SpectralDenoiser;
pub use speech_eq::SpeechEq;
pub use stream::{AudioInputStream, AudioOutputStream, AudioPacket, AudioPacketStream};
pub use streaming_stt::{
    MockStreamingSttProvider, StreamingAudioBuffer, StreamingAudioChunk, StreamingSttClient,
    StreamingSttConfig, StreamingSttError, StreamingSttEvent, StreamingSttLatency,
    StreamingSttProvider, StreamingSttResult,
};
pub use thread_priority::{
    configure_audio_thread, pin_to_core, set_thread_priority, temporary_priority_boost,
    AudioThreadHandle, AudioThreadPriority,
};
pub use turn_detector::{ProsodyAnalyzer, ProsodyState, TurnDetector};
pub use voice_memory::{VoiceMemory, VoiceProfile, VolumeTendency};
pub use volume_adapter::VolumeAdapter;
pub use wasapi_improvements::{
    ClockDriftDetector, ClockDriftMetrics, UnderrunDetector, UnderrunMetrics,
    WasapiEventDrivenConfig, WasapiHealthMetrics, WasapiHealthMonitor, WasapiShareMode,
};
pub use wasapi_session::{
    AudioSessionEvent, AudioSessionInfo, AudioSessionManager, AudioSessionState,
    InMemorySessionManager as InMemoryAudioSessionManager, VolumeSnapshot, WasapiSessionConfig,
};
pub use watchdog::{CircuitBreaker, HealthStatus, HealthWatchdog, StageHealth};

#[cfg(windows)]
pub use wasapi::{WasapiBackend, WasapiDeviceManager};
