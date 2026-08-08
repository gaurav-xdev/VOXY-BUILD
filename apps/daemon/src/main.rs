use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod background;
mod benchmark;
mod shutdown;

use voxy_automation::{HybridBackend, WindowsUiaBackend};
use voxy_cognition::{CognitionConfig, CognitiveEngine, InMemoryCognitiveEngine, IntentInput};
use voxy_cognitive_orchestrator::bridge::CognitiveBridge;
use voxy_cognitive_orchestrator::config::OrchestratorConfig;
use voxy_companion_intelligence::{
    ExperienceBridge, ExperienceInput, IntelligenceConfig, MomentContext, MomentEngine,
};
use voxy_desktop_runtime::{DesktopRuntime, RuntimeConfig};
use voxy_kokoro::KokoroTtsEngine;
use voxy_ollama::OllamaProvider;
use voxy_orchestrator::automation::AutomationBackend;
use voxy_provider_core::LlmProvider;
use voxy_runtime_guard::{GuardConfig, RuntimeGuard};
use voxy_security::{
    sanitize_context, sanitize_llm_output, sanitize_user_input, AuditEventType, AuditLog,
    CapabilityRegistry, GuardianConfig, GuardianEngine, PolicyEngine, RecoveryMode,
    SystemPromptBuilder,
};
use voxy_voice::VoicePipeline;
use chrono::Timelike;
use voxy_world_model::{DesktopEventBridge, WorldModelConfig};

fn setup_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,opencode_debug=debug".into()),
        )
        .try_init();
}

struct ConversationMemory {
    turns: VecDeque<(String, String)>,
    last_app: Option<String>,
    mention_count: usize,
}

impl ConversationMemory {
    fn new() -> Self {
        Self {
            turns: VecDeque::with_capacity(20),
            last_app: None,
            mention_count: 0,
        }
    }

    fn add_turn(&mut self, role: &str, text: &str) {
        if self.turns.len() >= 20 {
            self.turns.pop_front();
        }
        self.turns.push_back((role.to_string(), text.to_string()));
        self.mention_count = self.mention_count.wrapping_add(1);
    }

    fn conversation_history(&self) -> String {
        self.turns
            .iter()
            .map(|(role, text)| format!("{}: {}", role, text))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[allow(dead_code)]
    fn last_user_message(&self) -> Option<&str> {
        self.turns
            .iter()
            .rev()
            .find(|(role, _)| role == "user")
            .map(|(_, text)| text.as_str())
    }

    fn is_follow_up(&self, text: &str) -> bool {
        let follow_words = [
            "it", "that", "this", "there", "again", "another", "also", "too",
        ];
        let first = text.split_whitespace().next().unwrap_or("");
        follow_words.contains(&first)
    }
}

struct VoiceMetrics {
    stt_count: AtomicU64,
    total_automation_ns: AtomicU64,
    total_tts_ns: AtomicU64,
    max_automation_ns: AtomicU64,
    max_tts_ns: AtomicU64,
    restart_count: AtomicU64,
    start_time: Instant,
}

impl VoiceMetrics {
    fn new() -> Self {
        Self {
            stt_count: AtomicU64::new(0),
            total_automation_ns: AtomicU64::new(0),
            total_tts_ns: AtomicU64::new(0),
            max_automation_ns: AtomicU64::new(0),
            max_tts_ns: AtomicU64::new(0),
            restart_count: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    fn _record_stt(&self) {
        self.stt_count.fetch_add(1, Ordering::Relaxed);
    }

    fn _record_tts(&self, dur: Duration) {
        let ns = dur.as_nanos() as u64;
        self.total_tts_ns.fetch_add(ns, Ordering::Relaxed);
        let mut max = self.max_tts_ns.load(Ordering::Relaxed);
        while ns > max {
            match self
                .max_tts_ns
                .compare_exchange(max, ns, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => break,
                Err(m) => max = m,
            }
        }
    }

    fn record_automation(&self, dur: Duration) {
        let ns = dur.as_nanos() as u64;
        self.total_automation_ns.fetch_add(ns, Ordering::Relaxed);
        let mut max = self.max_automation_ns.load(Ordering::Relaxed);
        while ns > max {
            match self.max_automation_ns.compare_exchange(
                max,
                ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(m) => max = m,
            }
        }
    }

    fn report(&self) -> String {
        let elapsed = self.start_time.elapsed();
        let sc = self.stt_count.load(Ordering::Relaxed);
        let auto_sum = self.total_automation_ns.load(Ordering::Relaxed);
        let auto_max = self.max_automation_ns.load(Ordering::Relaxed);
        let tts_sum = self.total_tts_ns.load(Ordering::Relaxed);
        let tts_max = self.max_tts_ns.load(Ordering::Relaxed);
        let restarts = self.restart_count.load(Ordering::Relaxed);
        format!(
            "**VOXY Metrics ({}s uptime, {} restarts):**\n  STT: {}, Automation avg: {:.1}ms max: {:.1}ms, TTS avg: {:.1}ms max: {:.1}ms",
            elapsed.as_secs(), restarts, sc,
            if sc > 0 { auto_sum as f64 / sc as f64 / 1_000_000.0 } else { 0.0 },
            auto_max as f64 / 1_000_000.0,
            if sc > 0 { tts_sum as f64 / sc as f64 / 1_000_000.0 } else { 0.0 },
            tts_max as f64 / 1_000_000.0,
        )
    }
}

static PICK_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn pick_idx(max: usize) -> usize {
    PICK_COUNTER.fetch_add(1, Ordering::Relaxed) % max
}

#[allow(dead_code)]
fn pick_greeting() -> &'static str {
    let g = ["Hello!", "Hi there!", "Hey!", "Yes?", "I'm here."];
    g[pick_idx(g.len())]
}

#[allow(dead_code)]
fn pick_goodbye() -> &'static str {
    let b = [
        "Goodbye!",
        "See you later!",
        "Take care!",
        "Talk soon!",
        "Bye!",
    ];
    b[pick_idx(b.len())]
}

#[allow(dead_code)]
fn pick_thanks() -> &'static str {
    let t = [
        "You're welcome!",
        "Happy to help!",
        "Anytime!",
        "My pleasure.",
    ];
    t[pick_idx(t.len())]
}

#[allow(dead_code)]
fn pick_identity() -> &'static str {
    let i = [
        "I am VOXY, your voice assistant.",
        "I'm VOXY! Your personal AI assistant.",
        "This is VOXY at your service.",
    ];
    i[pick_idx(i.len())]
}

#[allow(dead_code)]
fn pick_verify_response(success: bool, app: &str) -> String {
    if success {
        let ok = [
            "{} is now open and ready.",
            "{} has been launched successfully.",
            "I've opened {} for you.",
        ];
        ok[pick_idx(ok.len())].replace("{}", app)
    } else {
        let fail = [
            "I had trouble opening {}. Let me retry.",
            "Sorry, I couldn't launch {} right now.",
        ];
        fail[pick_idx(fail.len())].replace("{}", app)
    }
}

fn pick_echo(text: &str) -> String {
    let prompts = ["I heard you mention", "You said", "Noted:", "I understand"];
    let prefix = prompts[pick_idx(prompts.len())];
    format!("{prefix}. {text}")
}

type PipelineFuture =
    Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>>;

#[allow(dead_code)]
fn get_memory_usage() -> (u64, u64) {
    let mut used = 0u64;
    let mut total = 0u64;
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("wmic")
            .args([
                "OS",
                "get",
                "TotalVisibleMemorySize,FreePhysicalMemory",
                "/format:csv",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 3 {
                    if let Ok(free_kb) = parts[2].trim().parse::<u64>() {
                        if let Ok(total_kb) = parts[1].trim().parse::<u64>() {
                            total = total_kb * 1024;
                            used = total.saturating_sub(free_kb * 1024);
                        }
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(info) = std::fs::read_to_string("/proc/meminfo") {
            for line in info.lines() {
                if let Some(val) = line.strip_prefix("MemTotal:") {
                    total = val
                        .trim()
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                        * 1024;
                } else if let Some(val) = line.strip_prefix("MemAvailable:") {
                    let avail = val
                        .trim()
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                        * 1024;
                    if total > avail {
                        used = total - avail;
                    }
                }
            }
        }
    }
    (used, total)
}

#[allow(dead_code)]
fn get_cpu_usage() -> f64 {
    #[cfg(target_os = "windows")]
    {
        use std::time::Instant;
        let mut system = sysinfo::System::new();
        system.refresh_cpu_all();
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(100));
        system.refresh_cpu_all();
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            let total: f64 = system.cpus().iter().map(|c| c.cpu_usage() as f64).sum();
            total / system.cpus().len() as f64
        } else {
            0.0
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::thread::sleep(Duration::from_millis(100));
        0.0
    }
}

#[allow(dead_code)]
fn measure_stage_ms<F, T>(f: F) -> (f64, T)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    (ms, result)
}

#[allow(dead_code)]
fn detect_simd() -> Vec<String> {
    let mut caps = Vec::new();
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            caps.push("AVX2".to_string());
        }
        if is_x86_feature_detected!("avx512f") {
            caps.push("AVX-512 (F)".to_string());
        }
        if is_x86_feature_detected!("avx512bw") {
            caps.push("AVX-512 (BW)".to_string());
        }
        if is_x86_feature_detected!("sse2") {
            caps.push("SSE2".to_string());
        }
        if is_x86_feature_detected!("ssse3") {
            caps.push("SSSE3".to_string());
        }
        if is_x86_feature_detected!("fma") {
            caps.push("FMA".to_string());
        }
        if is_x86_feature_detected!("aes") {
            caps.push("AES-NI".to_string());
        }
    }
    if caps.is_empty() {
        caps.push("none detected".to_string());
    }
    caps
}

#[allow(dead_code)]
fn detect_cpu_features() -> Vec<String> {
    let mut features = Vec::new();
    features.push(format!("logical cores: {}", num_cpus::get()));
    features.push(format!("physical cores: {}", num_cpus::get_physical()));
    features
}

fn run_pipeline(running: Arc<AtomicBool>, metrics: Arc<VoiceMetrics>) -> PipelineFuture {
    Box::pin(async move {
        let config = voxy_voice::VoiceConfig {
            auto_start_capture: true,
            wake_word: "hey voxy".into(),
            wake_word_enabled: true,
            vad_enabled: true,
            vad_threshold: 0.05,
            ..Default::default()
        };

        let audio_mgr = Box::new(voxy_audio::WasapiDeviceManager::new());
        let pipeline = Arc::new(VoicePipeline::with_audio_mgr(config.clone(), audio_mgr));
        pipeline.initialize().await?;
        pipeline.with_default_engines().await?;

        #[cfg(feature = "whisper-engine")]
        {
            let whisper = voxy_whisper::WhisperSttEngine::new()
                .with_model_path("models/ggml-base.en.bin".into());
            match whisper.load_model() {
                Ok(()) => tracing::info!("Whisper model loaded"),
                Err(e) => tracing::error!("Failed to load whisper model: {e}"),
            }
            pipeline.set_stt_engine(Box::new(whisper)).await?;
        }
        #[cfg(not(feature = "whisper-engine"))]
        {
            tracing::warn!("whisper-engine feature disabled; STT will return empty");
            pipeline
                .set_stt_engine(Box::new(voxy_whisper::WhisperSttEngine::new()))
                .await?;
        }

        #[cfg(feature = "piper-engine")]
        {
            let tts = KokoroTtsEngine::new()
                .with_voice("default")
                .with_speed(1.0)
                .with_pitch(1.0)
                .with_model_path("models/en_US-lessac-medium.onnx".into());
            match tts.load_model() {
                Ok(()) => tracing::info!("Piper TTS model loaded"),
                Err(e) => tracing::error!("Failed to load piper model: {e}"),
            }
            pipeline.set_tts_engine(Box::new(tts)).await?;
        }
        #[cfg(not(feature = "piper-engine"))]
        {
            tracing::warn!("piper-engine feature disabled; TTS will return empty");
            let tts = KokoroTtsEngine::new()
                .with_voice("default")
                .with_speed(1.0)
                .with_pitch(1.0);
            pipeline.set_tts_engine(Box::new(tts)).await?;
        }

        pipeline.start_capture().await?;

        // ── Voice Engine V2: initialize watchdog, calibrator, metrics ──
        pipeline.initialize_v2().await?;
        if let Some(ref _watchdog) = pipeline.watchdog() {
            tracing::info!("V2 watchdog registered for 5 stages");
        }
        if let Some(ref _calibrator) = pipeline.calibrator() {
            tracing::info!("V2 self-calibrator ready (will calibrate during first use)");
        }
        if let Some(ref _mc) = pipeline.metrics_collector() {
            tracing::info!("V2 metrics collector initialized");
        }
        if let Some(ref _vm) = pipeline.voice_memory() {
            tracing::info!("V2 voice memory initialized");
        }
        tracing::info!("Voice Engine V2 fully integrated");

        // ── Experience Layer ──────────────────────────────────────────────
        let intelligence_config = IntelligenceConfig::default();
        let (exp_bridge, exp_input_tx, mut exp_output_rx) =
            ExperienceBridge::new(intelligence_config);
        exp_bridge.start().await;
        let exp_bridge = Arc::new(exp_bridge);
        let mut moment_engine = MomentEngine::new();
        tracing::info!("Experience Layer started");

        // ── Cognitive Orchestrator ────────────────────────────────────────
        let cognitive_bridge = Arc::new(CognitiveBridge::new(OrchestratorConfig::default()));
        tracing::info!("Cognitive Orchestrator initialized");

        // ── Runtime Guard ────────────────────────────────────────────────
        let guard = Arc::new(RuntimeGuard::new(GuardConfig::default()));
        tracing::info!("Runtime Guard initialized");

        // Register core subsystems
        let g = guard.clone();
        g.register_subsystem("voice_pipeline", || async {
            voxy_health::HealthReport::new("voice_pipeline", voxy_shared::HealthStatus::Healthy)
        })
        .await;

        let g = guard.clone();
        g.register_subsystem("cognitive_bridge", || async {
            voxy_health::HealthReport::new("cognitive_bridge", voxy_shared::HealthStatus::Healthy)
        })
        .await;

        let g = guard.clone();
        g.register_subsystem("experience_bridge", || async {
            voxy_health::HealthReport::new("experience_bridge", voxy_shared::HealthStatus::Healthy)
        })
        .await;

        let g = guard.clone();
        g.register_subsystem("desktop_bridge", || async {
            voxy_health::HealthReport::new("desktop_bridge", voxy_shared::HealthStatus::Healthy)
        })
        .await;

        guard.heartbeat("voice_pipeline");
        guard.heartbeat("cognitive_bridge");
        guard.heartbeat("experience_bridge");
        guard.heartbeat("desktop_bridge");
        tracing::info!("Runtime Guard: 4 subsystems registered");

        // ── Desktop Runtime ──────────────────────────────────────────────
        let desktop_config = RuntimeConfig::new("VOXY");
        let mut desktop_runtime = DesktopRuntime::new(desktop_config).unwrap_or_else(|e| {
            tracing::warn!("Desktop runtime init failed (non-fatal): {}", e);
            DesktopRuntime::new(RuntimeConfig::new("VOXY")).expect("desktop runtime fallback")
        });
        let _ = desktop_runtime.start().await;
        let desktop_runtime = Arc::new(desktop_runtime);
        tracing::info!("Desktop runtime initialized");

        // ── Graceful Shutdown Coordinator ────────────────────────────────
        let mut graceful =
            shutdown::GracefulShutdown::new(running.clone()).with_timeout(Duration::from_secs(30));

        // ── Background Runtime ────────────────────────────────────────────
        let (bg_runtime, reflection_submitter, _knowledge_validation_submitter) =
            background::BackgroundRuntime::new(
                running.clone(),
                metrics.clone(),
                cognitive_bridge.clone(),
                guard.clone(),
            );
        tracing::info!("Background runtime started (9 tasks)");

        // ── Register Shutdown Subsystems ─────────────────────────────────
        {
            let bg = bg_runtime.clone_for_shutdown();
            graceful.register_simple(
                "background_runtime",
                shutdown::ShutdownPriority::Background,
                Duration::from_secs(10),
                move || {
                    let bg = bg.clone();
                    async move { bg.shutdown().await }
                },
            );
        }
        {
            let bridge = cognitive_bridge.clone();
            graceful.register_simple(
                "cognitive_bridge",
                shutdown::ShutdownPriority::Background,
                Duration::from_secs(5),
                move || {
                    let bridge = bridge.clone();
                    async move {
                        bridge.shutdown();
                    }
                },
            );
        }
        {
            let exp = exp_bridge.clone();
            graceful.register_simple(
                "experience_bridge",
                shutdown::ShutdownPriority::Services,
                Duration::from_secs(5),
                move || {
                    let exp = exp.clone();
                    async move {
                        exp.stop().await;
                    }
                },
            );
        }
        {
            let dr = desktop_runtime.clone();
            graceful.register_simple(
                "desktop_runtime",
                shutdown::ShutdownPriority::Background,
                Duration::from_secs(5),
                move || {
                    let dr = dr.clone();
                    async move {
                        let _ = dr.shutdown().await;
                    }
                },
            );
        }

        // ── Ollama LLM ───────────────────────────────────────────────────
        let ollama_url =
            std::env::var("VOXY_OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".into());
        let ollama_model =
            std::env::var("VOXY_OLLAMA_MODEL").unwrap_or_else(|_| "voxy-fast:latest".into());
        let llm = Arc::new(
            OllamaProvider::new(&ollama_url, &ollama_model)
                .map_err(|e| tracing::warn!("Failed to create Ollama client: {e}"))
                .unwrap_or_else(|_| OllamaProvider::default()),
        );
        match llm.health().await {
            Ok(true) => tracing::info!("Ollama connected: {} @ {}", ollama_model, ollama_url),
            Ok(false) => tracing::warn!("Ollama health check failed, will retry on first request"),
            Err(e) => tracing::warn!("Ollama not reachable: {e} — LLM responses will be fallbacks"),
        }

        // Register Ollama LLM as a healable subsystem
        {
            let ollama_url_clone = ollama_url.clone();
            let g = guard.clone();
            g.register_healable(
                "ollama_llm",
                move || {
                    let url = format!("{}/api/tags", ollama_url_clone);
                    async move {
                        match reqwest::Client::new()
                            .get(&url)
                            .timeout(Duration::from_secs(5))
                            .send()
                            .await
                        {
                            Ok(resp) if resp.status().is_success() => {
                                voxy_health::HealthReport::new(
                                    "ollama_llm",
                                    voxy_shared::HealthStatus::Healthy,
                                )
                            }
                            Ok(resp) => voxy_health::HealthReport::new(
                                "ollama_llm",
                                voxy_shared::HealthStatus::Degraded(format!("HTTP {}", resp.status())),
                            ),
                            Err(e) => voxy_health::HealthReport::new(
                                "ollama_llm",
                                voxy_shared::HealthStatus::Unhealthy(format!("{}", e)),
                            ),
                        }
                    }
                },
                move || {
                    let url = format!("{}/api/tags", ollama_url);
                    async move {
                        // Wait briefly for Ollama to potentially recover
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        // Verify Ollama is actually healthy before declaring recovery
                        match reqwest::Client::new()
                            .get(&url)
                            .timeout(Duration::from_secs(5))
                            .send()
                            .await
                        {
                            Ok(resp) if resp.status().is_success() => {
                                tracing::info!("Ollama recovery verified — health check passed");
                                Ok(())
                            }
                            Ok(resp) => {
                                let msg = format!("Ollama returned HTTP {}", resp.status());
                                tracing::warn!("Ollama recovery failed: {}", msg);
                                Err(msg)
                            }
                            Err(e) => {
                                tracing::warn!("Ollama recovery failed: {}", e);
                                Err(format!("{}", e))
                            }
                        }
                    }
                },
            )
            .await;
            guard.heartbeat("ollama_llm");
            tracing::info!("Runtime Guard: 5 subsystems registered (1 healable)");
        }

        // ── Cognitive Engine (intent matching) ────────────────────────────
        let cognitive = Arc::new(InMemoryCognitiveEngine::new(CognitionConfig::default()));

        // ── Guardian Security Engine ─────────────────────────────────────
        let audit_log = Arc::new(tokio::sync::Mutex::new(AuditLog::new()));
        let guardian = Arc::new(GuardianEngine::new(
            CapabilityRegistry::new(),
            PolicyEngine::with_default_rules(),
            GuardianConfig::default(),
        ));

        // ── Recovery Mode ───────────────────────────────────────────────
        let recovery_mode = Arc::new(tokio::sync::Mutex::new(RecoveryMode::new()));

        // Record startup audit event
        {
            let mut log = audit_log.lock().await;
            log.record_typed(
                "system",
                "startup",
                None,
                "allowed",
                Some("VOXY daemon started"),
                "none",
                "verified",
                voxy_security::policy::AuditLevel::Basic,
                AuditEventType::Authentication {
                    method: "system_boot".to_string(),
                    success: true,
                },
            );
        }

        tracing::info!("Guardian security engine initialized");

        // ── System Prompt (immutable layers) ─────────────────────────────
        let system_prompt = Arc::new(SystemPromptBuilder::new().build());

        // ── Conversation Memory ───────────────────────────────────────────
        let memory = Arc::new(tokio::sync::Mutex::new(ConversationMemory::new()));
        let metrics_ref = metrics.clone();
        let llm_ref = llm.clone();
        let exp_input_for_handler = exp_input_tx.clone();

        // ── Desktop context (shared state updated by timer loop) ──────────
        let desktop_context: Arc<tokio::sync::RwLock<String>> =
            Arc::new(tokio::sync::RwLock::new(String::new()));

        // ── Task completion counter (shared between response handler and timer) ──
        let tasks_completed = Arc::new(std::sync::atomic::AtomicU32::new(0));

        // ── Response Handler ──────────────────────────────────────────────
        let response_handler = {
            let cognitive = cognitive.clone();
            let memory_clone = memory.clone();
            let exp_input = exp_input_for_handler.clone();
            let llm = llm_ref.clone();
            let desktop_ctx = desktop_context.clone();
            let reflector = reflection_submitter.clone();
            let guardian = guardian.clone();
            let sys_prompt = system_prompt.clone();
            let tasks_completed = tasks_completed.clone();
            let audit_log = audit_log.clone();
            let recovery_mode = recovery_mode.clone();
            Box::new(
                move |text: String| -> Pin<Box<dyn std::future::Future<Output = String> + Send>> {
                    let cognitive = cognitive.clone();
                    let memory = memory_clone.clone();
                    let metrics = metrics_ref.clone();
                    let exp_input = exp_input.clone();
                    let llm = llm.clone();
                    let desktop_ctx = desktop_ctx.clone();
                    let reflector = reflector.clone();
                    let guardian = guardian.clone();
                    let sys_prompt = sys_prompt.clone();
                    let recovery_mode = recovery_mode.clone();
                    let tasks_completed = tasks_completed.clone();
                    let audit_log = audit_log.clone();
                    Box::pin(async move {
                        // ── 0. SANITIZE INPUT (security boundary) ────────
                        let sanitized = sanitize_user_input(&text);
                        if sanitized.injection_detected {
                            tracing::warn!(
                                patterns = ?sanitized.patterns,
                                "Injection pattern detected in user input — sanitizing"
                            );
                        }
                        let text = sanitized.text;

                        if text.is_empty() || text == "short sound" {
                            return String::new();
                        }

                        let response_start = Instant::now();

                        // ── 1. Feed transcript → Experience Layer ──────────
                        let _ = exp_input.send(ExperienceInput::VoiceTranscript {
                            text: text.clone(),
                            is_final: true,
                        });

                        // ── 2. Store user turn in memory ───────────────────
                        let lower = text.to_lowercase();
                        let mut mem = memory.lock().await;
                        mem.add_turn("user", &text);
                        let _is_follow_up = mem.is_follow_up(&lower);

                        // ── 3. Handle automation commands directly ──────────
                        if lower.contains("open")
                            || lower.contains("launch")
                            || lower.contains("start")
                        {
                            let app = if lower.contains("chrome") || lower.contains("browser") {
                                "chrome"
                            } else if lower.contains("notepad") || lower.contains("editor") {
                                "notepad"
                            } else if lower.contains("calc") || lower.contains("calculator") {
                                "calc"
                            } else if lower.contains("explorer") || lower.contains("files") {
                                "explorer"
                            } else {
                                let response =
                                    "I'm not sure which application to open.".to_string();
                                mem.add_turn("assistant", &response);
                                drop(mem);
                                let _ = exp_input.send(ExperienceInput::VoiceTranscript {
                                    text: response.clone(),
                                    is_final: true,
                                });
                                return response;
                            };

                            // ── GUARDIAN CHECK: automation requires authorization ──
                            let decision = {
                                let r = recovery_mode.lock().await;
                                guardian.evaluate(
                                    "voice-user",
                                    "automation:write",
                                    Some(app),
                                    "launch_application",
                                    std::collections::HashMap::new(),
                                    &r,
                                )
                            };

                            // Record typed audit event for guardian decision
                            {
                                let mut log = audit_log.lock().await;
                                let event_type = if decision.allowed {
                                    AuditEventType::Authorization {
                                        decision: "allowed".to_string(),
                                    }
                                } else {
                                    AuditEventType::Authorization {
                                        decision: "denied".to_string(),
                                    }
                                };
                                log.record_typed(
                                    "voice-user",
                                    "automation:write",
                                    Some(app),
                                    if decision.allowed { "allowed" } else { "denied" },
                                    Some(&decision.reason),
                                    "high",
                                    "trusted",
                                    voxy_security::policy::AuditLevel::Detailed,
                                    event_type,
                                );
                            }

                            if !decision.allowed {
                                // Activate recovery mode on critical-risk denials
                                if decision.requires_mfa {
                                    let mut recovery = recovery_mode.lock().await;
                                    if recovery.state() == voxy_security::recovery::RecoveryState::Normal {
                                        let auth = voxy_security::recovery::RecoveryAuth {
                                            subject: "system".to_string(),
                                            reason: format!(
                                                "Critical risk action denied: {}",
                                                decision.reason
                                            ),
                                            auth_method: "automatic_guardian".to_string(),
                                        };
                                        if let Err(e) = recovery.enter(auth) {
                                            tracing::error!(
                                                error = %e,
                                                "Failed to activate recovery mode"
                                            );
                                        } else {
                                            tracing::warn!(
                                                reason = %decision.reason,
                                                "Recovery mode activated due to critical threat"
                                            );
                                        }
                                    }
                                }

                                let response = format!(
                                    "I need permission to open {}. {}",
                                    app, decision.reason
                                );
                                mem.add_turn("assistant", &response);
                                drop(mem);
                                let _ = exp_input.send(ExperienceInput::VoiceTranscript {
                                    text: response.clone(),
                                    is_final: true,
                                });
                                return response;
                            }

                            mem.last_app = Some(app.to_string());
                            let uia = WindowsUiaBackend::new();
                            let response = if uia.is_available().await {
                                let auto_start = Instant::now();
                                match open_application(app).await {
                                    Ok(_) => {
                                        metrics.record_automation(auto_start.elapsed());
                                        format!("Done — {app} is now open.")
                                    }
                                    Err(e) => {
                                        metrics.record_automation(auto_start.elapsed());
                                        format!("I couldn't open {app}. {e}")
                                    }
                                }
                            } else {
                                format!("I'd open {app} for you, but automation isn't available.")
                            };
                            mem.add_turn("assistant", &response);
                            drop(mem);
                            let _ = exp_input.send(ExperienceInput::VoiceTranscript {
                                text: response.clone(),
                                is_final: true,
                            });
                            return response;
                        }

                        if lower.contains("open it again") || lower.contains("reopen") {
                            if let Some(app) = mem.last_app.clone() {
                                let response = match open_application(&app).await {
                                    Ok(_) => format!("Opening {app} again."),
                                    Err(e) => format!("Failed to reopen {app}: {e}"),
                                };
                                mem.add_turn("assistant", &response);
                                drop(mem);
                                let _ = exp_input.send(ExperienceInput::VoiceTranscript {
                                    text: response.clone(),
                                    is_final: true,
                                });
                                return response;
                            }
                        }

                        // ── 4. Build LLM prompt with full context ──────────
                        let history = mem.conversation_history();
                        let desktop = sanitize_context(&desktop_ctx.read().await.clone());
                        drop(mem);

                        let system_prompt = format!(
                            "{}\n\n\
                             Current desktop context: {}\n\n\
                             Recent conversation:\n{}",
                            sys_prompt,
                            if desktop.is_empty() {
                                "unknown"
                            } else {
                                &desktop
                            },
                            if history.is_empty() {
                                "No prior conversation.".to_string()
                            } else {
                                history
                            },
                        );

                        // ── 5. Generate response via Ollama LLM ───────────
                        let response = match llm
                            .complete(&format!(
                                "{system_prompt}\n\n{}",
                                voxy_security::prompt::SystemPromptBuilder::format_user_message(
                                    &text
                                )
                            ))
                            .await
                        {
                            Ok(raw) => {
                                let cleaned = raw
                                    .trim()
                                    .trim_start_matches("Assistant:")
                                    .trim()
                                    .to_string();
                                if cleaned.is_empty() {
                                    pick_echo(&text)
                                } else {
                                    // Sanitize LLM output to prevent system prompt leakage
                                    sanitize_llm_output(&cleaned)
                                }
                            }
                            Err(e) => {
                                tracing::warn!("LLM call failed: {e} — using fallback");
                                // Fallback to intent matching
                                let intent = IntentInput {
                                    raw_text: text.clone(),
                                    context: None,
                                    source: "voice".to_string(),
                                    metadata: std::collections::HashMap::new(),
                                };
                                match cognitive.process(&intent).await {
                                    Ok(_) => pick_echo(&text),
                                    Err(_) => pick_echo(&text),
                                }
                            }
                        };

                        let elapsed_ms = response_start.elapsed().as_millis();
                        tracing::info!(
                            input = %text,
                            response_len = response.len(),
                            latency_ms = elapsed_ms,
                            "Response generated"
                        );

                        // ── 6. Store assistant turn in memory ──────────────
                        {
                            let mut mem = memory.lock().await;
                            mem.add_turn("assistant", &response);
                        }

                        // ── 6b. Submit for reflection ────────────────────
                        {
                            let mem = memory.lock().await;
                            let messages: Vec<(String, String)> =
                                mem.turns.iter().cloned().collect();
                            drop(mem);
                            if messages.len() >= 2 {
                                let record =
                                    voxy_cognitive_orchestrator::reflection::ConversationRecord {
                                        id: uuid::Uuid::new_v4(),
                                        messages,
                                        context: String::new(),
                                        timestamp: chrono::Utc::now(),
                                    };
                                let _ = reflector.submit(record).await;
                            }
                        }

                        // ── 7. Feed response → Experience Layer ────────────
                        let _ = exp_input.send(ExperienceInput::VoiceTranscript {
                            text: response.clone(),
                            is_final: true,
                        });

                        // ── 8. Track task completion for moments ──────────
                        tasks_completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        response
                    })
                },
            )
        };

        pipeline.set_response_handler(response_handler).await;
        pipeline.start_listening().await?;

        tracing::info!("VOXY is listening. Say '{}' to activate.", config.wake_word);

        // ── Desktop Event Bridge ──────────────────────────────────────────
        let desktop_config = WorldModelConfig::default();
        let bridge = Arc::new(DesktopEventBridge::new(desktop_config));
        if let Err(e) = bridge.start().await {
            tracing::warn!("Desktop event bridge failed to start: {}", e);
        } else {
            tracing::info!("Desktop event bridge started");
        }

        let exp_input_tx_clone = exp_input_tx.clone();
        let bridge_clone = bridge.clone();
        let desktop_ctx_clone = desktop_context.clone();
        let guard_clone = guard.clone();
        let mut timer = tokio::time::interval(Duration::from_secs(5));
        let mut last_focused_app: Option<String> = None;
        let mut last_activity: Option<String> = None;
        let mut last_idle: Option<bool> = None;

        // ── Moment context tracking ───────────────────────────────────
        let mut last_idle_since: Option<Instant> = None;
        let mut focus_start: Option<Instant> = None;
        let mut has_thanked_focus = false;

        // ── Experience Output → Visual Presence ───────────────────────────
        let (presence_tx, _presence_rx) = tokio::sync::broadcast::channel::<String>(64);
        let presence_tx_clone = presence_tx.clone();
        let exp_output_handle = tokio::spawn(async move {
            loop {
                match exp_output_rx.recv().await {
                    Ok(output) => {
                        // Forward presence state changes to visual presence channel
                        let presence_str = format!("{:?}", output.presence_state);
                        let _ = presence_tx_clone.send(presence_str);

                        // Forward mood changes
                        let mood_str = format!("{:?}", output.current_mood);
                        let _ = presence_tx_clone.send(format!("mood:{mood_str}"));

                        tracing::debug!(
                            mood = ?output.current_mood,
                            presence = ?output.presence_state,
                            speed = output.voice_params.speed,
                            "Presence update forwarded"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Experience output channel closed");
                        break;
                    }
                }
            }
        });

        // ── Main Loop: Desktop context → Experience Layer ─────────────────
        loop {
            tokio::select! {
                    _ = timer.tick() => {
                        if !running.load(Ordering::Relaxed) {
                            break;
                        }
                        let ctx = bridge_clone.get_current_context().await;

                        // Build desktop context string for LLM
                        let ctx_str = format!(
                            "App: {}, Activity: {}, Window: {}, Idle: {}",
                            ctx.focused_app.as_deref().unwrap_or("none"),
                            ctx.activity_type.as_deref().unwrap_or("none"),
                            ctx.window_title.as_deref().unwrap_or("none"),
                            ctx.is_idle,
                        );
                        *desktop_ctx_clone.write().await = ctx_str;

                        guard_clone.heartbeat("voice_pipeline");
                        guard_clone.heartbeat("cognitive_bridge");
                        guard_clone.heartbeat("experience_bridge");
                        guard_clone.heartbeat("desktop_bridge");

                        // ── V2 watchdog heartbeats ──
            if let Some(ref watchdog) = pipeline.watchdog() {
                            watchdog.heartbeat("audio_input");
                        }

                        tracing::debug!(
                            app = ?ctx.focused_app,
                            activity = ?ctx.activity_type,
                            idle = ctx.is_idle,
                            "Desktop context"
                        );

                        // Feed desktop events → Experience Layer
                        if let Some(ref app) = ctx.focused_app {
                            if last_focused_app.as_ref() != Some(app) {
                                let _ = exp_input_tx_clone.send(ExperienceInput::DesktopFocusChanged {
                                    app: app.clone(),
                                    window_title: ctx.window_title.clone(),
                                });
                                last_focused_app = Some(app.clone());
                            }
                        }

                        if let Some(ref activity) = ctx.activity_type {
                            if last_activity.as_ref() != Some(activity) {
                                let _ = exp_input_tx_clone.send(ExperienceInput::DesktopActivityChanged {
                                    activity: activity.clone(),
                                });
                                last_activity = Some(activity.clone());
                            }
                        }

                        if last_idle != Some(ctx.is_idle) {
                            let _ = exp_input_tx_clone.send(ExperienceInput::DesktopIdle {
                                is_idle: ctx.is_idle,
                            });
                            last_idle = Some(ctx.is_idle);

                            // Track idle/active transitions for moments
                            if ctx.is_idle {
                                last_idle_since = Some(Instant::now());
                                focus_start = None;
                                has_thanked_focus = false;
                            } else {
                                if last_idle_since.is_some() {
                                    focus_start = Some(Instant::now());
                                }
                            }
                        }

                        // Track tasks completed (successful LLM responses)
                        if last_focused_app.as_ref() != Some(&ctx.focused_app.clone().unwrap_or_default())
                            && ctx.focused_app.is_some()
                        {
                            // Application change can indicate task boundary
                        }

                        // ── Sync voice speed from Experience Layer ────────
                        {
                            let snapshot = exp_bridge.get_snapshot().await;
                            let speed = snapshot.current_mood.voice_speed_modifier();
                            pipeline.set_voice_speed(speed).await;
                        }

                        // ── Companion Moments ────────────────────────────
                        let idle_duration_chrono = last_idle_since
                            .map(|t| {
                                let secs = t.elapsed().as_secs() as i64;
                                chrono::Duration::seconds(secs)
                            })
                            .unwrap_or_else(|| chrono::Duration::seconds(0));
                        let absence_duration = if ctx.is_idle {
                            last_idle_since
                                .map(|t| {
                                    let secs = t.elapsed().as_secs() as i64;
                                    chrono::Duration::seconds(secs)
                                })
                                .unwrap_or_else(|| chrono::Duration::seconds(0))
                        } else if let Some(last_idle) = last_idle_since {
                            let secs = last_idle.elapsed().as_secs() as i64;
                            chrono::Duration::seconds(secs)
                        } else {
                            chrono::Duration::seconds(0)
                        };
                        let focused_duration = if !ctx.is_idle {
                            focus_start
                                .map(|t| {
                                    let secs = t.elapsed().as_secs() as i64;
                                    chrono::Duration::seconds(secs)
                                })
                                .unwrap_or_else(|| chrono::Duration::seconds(0))
                        } else {
                            chrono::Duration::seconds(0)
                        };
                        let user_just_returned = last_idle == Some(true) && !ctx.is_idle;

                        // Reset daily counters at midnight
                        let now = chrono::Local::now();
                        if now.hour() == 0 && now.minute() == 0 {
                            tasks_completed.store(0, std::sync::atomic::Ordering::Relaxed);
                        }

                        let moment_ctx = MomentContext {
                            user_just_returned,
                            absence_duration,
                            is_idle: ctx.is_idle,
                            idle_duration: idle_duration_chrono,
                            battery_percent: None,
                            is_charging: None,
                            next_meeting_in_minutes: None,
                            recent_download_complete: None,
                            focused_duration,
                            has_been_thanked_for_focus: has_thanked_focus,
                            tasks_completed_today: tasks_completed.load(std::sync::atomic::Ordering::Relaxed) as usize,
                            project_completed: false,
                            code_just_compiled: false,
                        };
                        let moments = moment_engine.check_moments(&moment_ctx);
                        for moment in moments {
                            match moment.moment_type {
                                voxy_companion_intelligence::MomentType::FocusedWork => {
                                    has_thanked_focus = true;
                                }
                                _ => {}
                            }
                            let _ = exp_input_tx_clone.send(ExperienceInput::SystemEvent {
                                event_type: format!("{:?}", moment.moment_type),
                                data: Some(moment.message),
                            });
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        running.store(false, Ordering::Relaxed);
                        break;
                    }
                }
        }

        // ── Cleanup ───────────────────────────────────────────────────────
        tracing::info!("Shutting down VOXY...");

        // Finalize recovery mode if it was active during this session
        {
            let mut recovery = recovery_mode.lock().await;
            if recovery.is_active() {
                let report = recovery.abort();
                if let Some(r) = report {
                    tracing::warn!(
                        report_id = %r.id,
                        authorized_by = %r.authorized_by,
                        "Recovery mode was active at shutdown — aborted"
                    );
                    let mut log = audit_log.lock().await;
                    log.record_typed(
                        "system",
                        "recovery_abort",
                        None,
                        "allowed",
                        Some("Recovery mode aborted at shutdown"),
                        "high",
                        "verified",
                        voxy_security::policy::AuditLevel::Full,
                        AuditEventType::RecoveryModeActivated {
                            reason: "Shutdown during recovery".to_string(),
                        },
                    );
                }
            }
        }

        // Record shutdown audit event
        {
            let mut log = audit_log.lock().await;
            log.record_typed(
                "system",
                "shutdown",
                None,
                "allowed",
                Some("VOXY daemon shutting down"),
                "none",
                "verified",
                voxy_security::policy::AuditLevel::Basic,
                AuditEventType::Authentication {
                    method: "system_shutdown".to_string(),
                    success: true,
                },
            );
        }

        graceful.execute().await;
        let _ = exp_output_handle.await;
        pipeline.stop_listening().await;
        pipeline.stop_capture().await?;
        pipeline.shutdown().await?;
        tracing::info!("VOXY shutdown complete");
        Ok(())
    })
}

async fn open_application(app_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let uia = Arc::new(WindowsUiaBackend::new());
    let hybrid = HybridBackend::builder()
        .with_primary(uia)
        .build()
        .map_err(|e| format!("Failed to create hybrid backend: {e}"))?;
    hybrid.initialize().await?;

    hybrid.key_combination(&["win", "r"]).await?;
    tokio::time::sleep(Duration::from_millis(300)).await;

    match app_name {
        "chrome" => hybrid.type_text("chrome", 8).await?,
        "notepad" => hybrid.type_text("notepad", 8).await?,
        "calc" => hybrid.type_text("calc", 8).await?,
        "explorer" => hybrid.type_text("explorer", 8).await?,
        "cmd" => hybrid.type_text("cmd", 8).await?,
        _ => {}
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
    hybrid.key_press("enter").await?;
    tokio::time::sleep(Duration::from_millis(800)).await;

    hybrid.key_combination(&["alt", "space"]).await.ok();
    tokio::time::sleep(Duration::from_millis(100)).await;
    hybrid.key_press("escape").await.ok();

    Ok(true)
}

#[tokio::main]
async fn main() {
    // Install crash handler before anything else
    voxy_logging::install_crash_handler();

    let _main_span = tracing::debug_span!("voxy_main").entered();
    setup_tracing();
    tracing::info!("Starting VOXY Assistant v{}", env!("CARGO_PKG_VERSION"));

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--benchmark") {
        tracing::info!("Running in benchmark mode...");

        let iterations = args
            .iter()
            .position(|a| a == "--iterations")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(100);

        let warmup = args
            .iter()
            .position(|a| a == "--warmup")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(10);

        let config = benchmark::BenchmarkConfig {
            iterations,
            warmup_iterations: warmup,
            measure_system_metrics: true,
            measure_voice_metrics: true,
            output_path: args
                .iter()
                .position(|a| a == "--output")
                .and_then(|i| args.get(i + 1))
                .cloned(),
            whisper_model_path: args
                .iter()
                .position(|a| a == "--whisper-model")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "models/ggml-base.en.bin".into()),
            piper_model_path: args
                .iter()
                .position(|a| a == "--piper-model")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "models/en_US-lessac-medium.onnx".into()),
            piper_config_path: args
                .iter()
                .position(|a| a == "--piper-config")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "models/en_US-lessac-medium.json".into()),
            ollama_url: args
                .iter()
                .position(|a| a == "--ollama-url")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "http://localhost:11434".into()),
            ollama_model: args
                .iter()
                .position(|a| a == "--ollama-model")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "gemma3:4b".into()),
        };

        let report = benchmark::run_full_benchmark(config).await;

        let markdown = report.to_markdown();

        if let Some(path) = &report.config.output_path {
            let path_clone = path.clone();
            let markdown_clone = markdown.clone();
            let write_result = tokio::task::spawn_blocking(move || {
                match std::fs::canonicalize(std::path::Path::new(&path_clone)) {
                    Ok(canonical) => {
                        let is_safe = canonical.components().all(|c| match c {
                            std::path::Component::Normal(s) => {
                                let p = s.to_string_lossy().to_lowercase();
                                !p.contains("password") && !p.contains("secret")
                                    && !p.contains("credential") && !p.contains("authorized_keys")
                                    && !p.contains("shadow") && !p.contains("sam")
                            }
                            _ => true,
                        });
                        if !is_safe {
                            Err(format!("Refusing to write benchmark report to sensitive path: {path_clone}"))
                        } else if let Err(e) = std::fs::write(&path_clone, &markdown_clone) {
                            Err(format!("Failed to write benchmark report: {e}"))
                        } else {
                            Ok(path_clone)
                        }
                    }
                    Err(_) => {
                        let parent = std::path::Path::new(&path_clone).parent().unwrap_or(std::path::Path::new("."));
                        if !parent.exists() {
                            Err(format!("Benchmark output directory does not exist: {}", parent.display()))
                        } else if let Err(e) = std::fs::write(&path_clone, &markdown_clone) {
                            Err(format!("Failed to write benchmark report: {e}"))
                        } else {
                            Ok(path_clone)
                        }
                    }
                }
            }).await;
            match write_result {
                Ok(Ok(p)) => tracing::info!("Benchmark report written to {p}"),
                Ok(Err(e)) => tracing::error!("{e}"),
                Err(e) => tracing::error!("Task join error: {e}"),
            }
        }

        println!("{}", markdown);

        tracing::info!(
            "Benchmark complete: {} stages, {} e2e metrics, {} bottlenecks identified.",
            report.stage_latencies.len(),
            report.e2e_latencies.len(),
            report.bottlenecks.len(),
        );
        return;
    }

    if args.iter().any(|a| a == "--status") {
        println!("VOXY Assistant v{}", env!("CARGO_PKG_VERSION"));
        println!("Status: Ready");
        println!("Features:");
        println!("  - Voice Pipeline (STT + TTS)");
        println!("  - Cognitive Orchestrator (reflection, knowledge validation, skill discovery)");
        println!("  - Runtime Guard (health monitoring, self-healing)");
        println!("  - Experience Layer (personality, mood, presence)");
        println!("  - Background Runtime (10 tasks)");
        println!();
        println!("Usage:");
        println!("  voxy-daemon              Start the daemon");
        println!("  voxy-daemon --benchmark  Run performance benchmark");
        println!("  voxy-daemon --status     Show this status");
        return;
    }

    let running = Arc::new(AtomicBool::new(true));
    let metrics = Arc::new(VoiceMetrics::new());

    let mut backoff = Duration::from_secs(1);

    while running.load(Ordering::Relaxed) {
        let result = run_pipeline(running.clone(), metrics.clone()).await;

        if !running.load(Ordering::Relaxed) {
            break;
        }

        match result {
            Ok(()) => {
                tracing::info!("VOXY loop ended normally, restarting...");
                backoff = Duration::from_secs(1);
            }
            Err(e) => {
                tracing::error!(
                    "VOXY loop crashed: {}. Restarting in {}s...",
                    e,
                    backoff.as_secs()
                );
                metrics.restart_count.fetch_add(1, Ordering::Relaxed);
            }
        }

        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(Duration::from_secs(30));
    }

    let report = metrics.report();
    tracing::info!("{report}");
    tracing::info!("VOXY Assistant stopped");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_simd_no_panic() {
        let caps = detect_simd();
        assert!(!caps.is_empty());
    }

    #[test]
    fn test_detect_cpu_features() {
        let features = detect_cpu_features();
        assert!(features.len() >= 2);
    }

    #[test]
    fn test_get_memory_usage_no_panic() {
        let (used, total) = get_memory_usage();
        assert!(total > 0 || total == 0);
        assert!(used <= total || total == 0);
    }

    #[test]
    fn test_measure_stage_sync() {
        let (ms, result) = measure_stage_ms(|| 42);
        assert!(ms >= 0.0);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_latency_sample_basic() {
        use benchmark::LatencySample;
        let mut s = LatencySample::new("test");
        s.record(10.0);
        s.record(20.0);
        s.record(30.0);
        assert_eq!(s.count(), 3);
        assert!((s.avg_ms() - 20.0).abs() < 1e-6);
        assert!((s.min_ms() - 10.0).abs() < 1e-6);
        assert!((s.max_ms() - 30.0).abs() < 1e-6);
        assert!((s.percentile(50.0) - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_latency_sample_empty() {
        use benchmark::LatencySample;
        let s = LatencySample::new("empty");
        assert_eq!(s.count(), 0);
        assert!((s.avg_ms() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_latency_sample_percentile() {
        use benchmark::LatencySample;
        let mut s = LatencySample::new("pct");
        for i in 1..=100 {
            s.record(i as f64);
        }
        assert!((s.percentile(50.0) - 50.5).abs() < 1.0);
        assert!((s.percentile(95.0) - 95.5).abs() < 1.0);
        assert!((s.percentile(99.0) - 99.5).abs() < 1.0);
    }

    #[test]
    fn test_benchmark_report_creation() {
        use benchmark::{BenchmarkConfig, BenchmarkReport};
        let config = BenchmarkConfig::default();
        let report = BenchmarkReport::new(config);
        let formatted = report.to_markdown();
        assert!(formatted.contains("VOXY Voice Runtime Performance Benchmark Report"));
    }

    #[test]
    fn test_benchmark_report_no_data() {
        use benchmark::{BenchmarkConfig, BenchmarkReport};
        let config = BenchmarkConfig::default();
        let report = BenchmarkReport::new(config);
        let fmt = report.to_markdown();
        assert!(fmt.contains("VOXY Voice Runtime Performance Benchmark Report"));
    }

    #[test]
    fn test_bottleneck_ranking() {
        use benchmark::{Bottleneck, Severity};
        let mut bottlenecks = vec![
            Bottleneck {
                rank: 0,
                stage: "fast".into(),
                avg_ms: 5.0,
                p95_ms: 10.0,
                percent_of_total: 1.0,
                severity: Severity::Low,
                improvement_estimate: "ok".into(),
            },
            Bottleneck {
                rank: 0,
                stage: "slow".into(),
                avg_ms: 500.0,
                p95_ms: 800.0,
                percent_of_total: 50.0,
                severity: Severity::High,
                improvement_estimate: "optimize".into(),
            },
        ];
        bottlenecks.sort_by(|a, b| b.avg_ms.partial_cmp(&a.avg_ms).unwrap());
        assert_eq!(bottlenecks[0].stage, "slow");
        assert_eq!(bottlenecks[1].stage, "fast");
    }
}
