# VOXY Health Architecture Design

**Status**: DRAFT — For Internal Review
**Version**: 1.0
**Author**: VOXY Architecture Team
**Date**: 2026-07-24

---

## 1. Overview

The Health layer provides **comprehensive system observability**, **automated recovery**, and **predictive failure detection** for VOXY. It goes beyond simple health checks to provide a full diagnostic and self-healing framework.

### Scope

| Component | Responsibility |
|-----------|----------------|
| **Health Monitor** | Periodic checks, aggregation, alerting |
| **Diagnostics** | Deep system inspection, root cause analysis |
| **Self-Test** | Startup/runtime validation of all subsystems |
| **Crash Reporting** | Capture, classify, report crashes |
| **Performance Monitoring** | Latency, throughput, resource tracking |
| **Resource Monitoring** | CPU, memory, disk, network, GPU |
| **Watchdog** | Liveness, deadlock detection, forced recovery |
| **Auto Recovery** | Remediation actions, escalation |
| **Predictive Failure** | ML-based anomaly detection, forecasting |

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Health Orchestrator                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌────────────┐ │
│  │   Health    │ │Diagnostics  │ │  Self-Test  │ │  Crash     │ │
│  │  Monitor    │ │   Engine    │ │  Framework  │ │  Reporter  │ │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └─────┬──────┘ │
│         │               │               │               │        │
│  ┌──────▼───────────────▼───────────────▼───────────────▼──────┐ │
│  │                    Health Event Bus                          │ │
│  └──────┬───────────────┬───────────────┬───────────────┬──────┘ │
│         │               │               │               │        │
│  ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐ │
│  │  Resource   │ │Performance  │ │  Watchdog   │ │  Recovery   │ │
│  │  Monitor    │ │  Monitor    │ │             │ │  Engine     │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
│         │               │               │               │        │
│  ┌──────▼───────────────▼───────────────▼───────────────▼──────┐ │
│  │                  Predictive Failure Engine                   │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Health Monitor

### 3.1 Check Types

```rust
pub enum CheckType {
    /// Fast, non-intrusive (ping, port, process alive)
    Liveness,
    /// Deeper check (DB query, API call, cache hit)
    Readiness,
    /// Comprehensive (full workflow, data integrity)
    Deep,
    /// Background, continuous (metrics, trends)
    Continuous,
}

pub enum CheckCategory {
    System,       // CPU, memory, disk, network
    Dependency,   // Database, external APIs, message queue
    Application,  // Business logic, data consistency
    Security,     // Certificates, permissions, audit
    Custom,       // Plugin/agent specific
}
```

### 3.2 Health Check Definition

```rust
pub struct HealthCheck {
    pub id: String,
    pub name: String,
    pub check_type: CheckType,
    pub category: CheckCategory,
    pub interval: Duration,
    pub timeout: Duration,
    pub critical: bool,           // If false, degraded not unhealthy
    pub dependencies: Vec<String>, // Other check IDs that must pass first
    pub tags: HashMap<String, String>,
    pub execute: CheckFn,
}

pub type CheckFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output = CheckResult> + Send>> + Send + Sync>;

pub struct CheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
    pub latency_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>>,
}

pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
    Unknown,
}
```

### 3.3 Built-in Checks

| Check ID | Category | Type | Description |
|----------|----------|------|-------------|
| `system.cpu` | System | Continuous | CPU usage < threshold |
| `system.memory` | System | Continuous | Memory usage < threshold |
| `system.disk` | System | Continuous | Disk space > threshold |
| `system.network` | System | Liveness | Network interface up |
| `kernel.runtime` | Dependency | Readiness | Kernel responsive |
| `eventbus.health` | Dependency | Readiness | Event bus processing |
| `ipc.health` | Dependency | Readiness | IPC server accepting |
| `database.health` | Dependency | Readiness | DB query succeeds |
| `database.migrations` | Application | Deep | Migrations current |
| `plugins.loaded` | Application | Readiness | All required plugins loaded |
| `voice.pipeline` | Application | Deep | Audio pipeline functional |
| `vision.pipeline` | Application | Deep | Vision pipeline functional |
| `automation.backend` | Application | Deep | Automation backend responsive |
| `security.vault` | Security | Readiness | Secret vault accessible |
| `security.audit` | Security | Deep | Audit log writable |
| `model.router` | Application | Readiness | Model router responsive |
| `agent.runtime` | Application | Readiness | Agent runtime healthy |

---

## 4. Diagnostics Engine

### 4.1 Diagnostic Session

```rust
pub struct DiagnosticSession {
    pub id: Uuid,
    pub trigger: DiagnosticTrigger,
    pub scope: DiagnosticScope,
    pub started_at: DateTime<Utc>,
    pub checks: Vec<DiagnosticCheck>,
    pub findings: Vec<Finding>,
    pub root_cause: Option<RootCause>,
    pub remediation: Vec<RemediationAction>,
}

pub enum DiagnosticTrigger {
    Manual { requested_by: Subject },
    Scheduled { schedule: Schedule },
    Alert { alert_id: String },
    Degradation { component: String },
    Crash { crash_id: String },
    Predictive { prediction_id: String },
}

pub enum DiagnosticScope {
    Full,
    Component(String),
    Subsystem(String),
    Custom(Vec<String>),
}

pub struct Finding {
    pub severity: Severity,
    pub component: String,
    pub check_id: String,
    pub description: String,
    pub evidence: Vec<Evidence>,
    pub impact: ImpactAssessment,
}

pub enum Severity { Info, Warning, Critical, Emergency }
```

### 4.2 Diagnostic Checks

| Check | Description |
|-------|-------------|
| `config.validate` | Configuration schema + values |
| `permissions.check` | File, capability, OS permissions |
| `connectivity.test` | All external dependencies |
| `data.integrity` | Checksums, foreign keys, consistency |
| `performance.baseline` | Compare against known good |
| `resource.leaks` | FD, memory, handle leaks |
| `deadlock.detect` | Lock graph analysis |
| `version.compatibility` | Plugin/kernel/model compatibility |
| `security.policy` | Policy evaluation test |

---

## 5. Self-Test Framework

### 5.1 Test Categories

```rust
pub enum SelfTestCategory {
    Startup,      // Run at startup, block ready
    Periodic,     // Run periodically in background
    OnDemand,     // Run when requested
    PreFlight,    // Run before critical operation
    PostFlight,   // Run after critical operation
}

pub struct SelfTest {
    pub id: String,
    pub name: String,
    pub category: SelfTestCategory,
    pub dependencies: Vec<String>,
    pub timeout: Duration,
    pub run: TestFn,
}

pub type TestFn = Box<dyn Fn(&mut TestContext) -> Pin<Box<dyn Future<Output = TestResult> + Send>> + Send + Sync>;

pub struct TestContext {
    pub config: &AppConfig,
    pub storage: &dyn StorageProvider,
    pub event_bus: &EventBus,
    pub ipc: &IpcClient,
    pub capabilities: &CapabilityManager,
}

pub struct TestResult {
    pub passed: bool,
    pub message: String,
    pub artifacts: Vec<TestArtifact>,
    pub duration_ms: u64,
}
```

### 5.2 Standard Self-Tests

| Test ID | Category | Description |
|---------|----------|-------------|
| `startup.config` | Startup | Config loads and validates |
| `startup.kernel` | Startup | Kernel initializes |
| `startup.database` | Startup | DB connects, migrations ok |
| `startup.plugins` | Startup | Required plugins load |
| `startup.ipc` | Startup | IPC server starts |
| `startup.security` | Startup | Vault, policies load |
| `periodic.memory` | Periodic | No memory leaks detected |
| `periodic.fd_leak` | Periodic | File descriptor count stable |
| `periodic.clock_sync` | Periodic | System clock synchronized |
| `preflight.voice` | PreFlight | Audio devices accessible |
| `preflight.vision` | PreFlight | Screen capture works |
| `preflight.automation` | PreFlight | UI automation permission |

---

## 6. Crash Reporting

### 6.1 Crash Capture

```rust
pub struct CrashReport {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub process: ProcessInfo,
    pub exception: ExceptionInfo,
    pub threads: Vec<ThreadInfo>,
    pub modules: Vec<ModuleInfo>,
    pub memory: MemoryInfo,
    pub context: CrashContext,
    pub minidump: Option<Vec<u8>>,  // Windows minidump / Linux coredump
}

pub struct ExceptionInfo {
    pub code: u32,
    pub address: usize,
    pub type: ExceptionType,
    pub message: String,
}

pub struct CrashContext {
    pub component: Option<String>,
    pub operation: Option<String>,
    pub user_action: Option<String>,
    pub config_hash: String,
    pub build_info: BuildInfo,
}
```

### 6.2 Crash Classification

| Class | Examples | Action |
|-------|----------|--------|
| `NullPointer` | Dereference null | Auto-restart component |
| `AssertionFailure` | Invariant violated | Alert, don't restart |
| `StackOverflow` | Recursion, large stack | Restart with larger stack |
| `OOM` | Allocation failure | Trigger memory recovery |
| `Deadlock` | Lock timeout | Force unlock, alert |
| `SecurityViolation` | Capability check failed | Audit, quarantine |
| `PluginCrash` | Plugin panic | Unload plugin, continue |
| `Unknown` | Unclassified | Full capture, alert |

### 6.3 Reporting Pipeline

```
Crash → Capture → Classify → Anonymize → Store → Notify → Analyze
                    │
                    └─→ Upload (opt-in) → Upstream
```

---

## 7. Performance Monitoring

### 7.1 Metrics Collection

```rust
pub struct PerformanceMonitor {
    collectors: Vec<Box<dyn MetricCollector>>,
    aggregator: MetricAggregator,
    exporter: MetricExporter,
}

pub trait MetricCollector: Send + Sync {
    fn name(&self) -> &str;
    fn collect(&self) -> Vec<MetricPoint>;
    fn interval(&self) -> Duration;
}

pub struct MetricPoint {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub metric_type: MetricType,
}

pub enum MetricType {
    Counter, Gauge, Histogram, Summary
}
```

### 7.2 Standard Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `voxy_latency_seconds` | Histogram | `component`, `operation`, `status` | Operation latency |
| `voxy_throughput_total` | Counter | `component`, `operation` | Operations per second |
| `voxy_error_rate` | Gauge | `component`, `error_type` | Error rate |
| `voxy_queue_depth` | Gauge | `component`, `queue` | Pending work |
| `voxy_cache_hit_ratio` | Gauge | `cache` | Cache effectiveness |
| `voxy_gc_pause_seconds` | Histogram | `generation` | GC pause times |

### 7.3 SLO/SLI Definitions

```rust
pub struct SloDefinition {
    pub name: String,
    pub sli: SliQuery,
    pub target: f64,        // e.g., 0.999
    pub window: Duration,   // e.g., 30 days
    pub alert_threshold: f64, // e.g., 0.99
}

pub enum SliQuery {
    Latency { percentile: f64, max_ms: u64 },
    Availability { min_success_rate: f64 },
    Throughput { min_ops_per_sec: f64 },
    ErrorRate { max_error_rate: f64 },
}
```

**Standard SLOs:**

| SLO | Target | Window | Alert At |
|-----|--------|--------|----------|
| API Latency (p99) | < 500ms | 30d | < 99% |
| IPC Latency (p99) | < 50ms | 30d | < 99.9% |
| Voice Pipeline Latency | < 200ms | 30d | < 99% |
| System Availability | 99.9% | 30d | 99.5% |
| Plugin Crash Rate | < 0.1%/hr | 24h | > 1%/hr |
| Memory Growth | < 10MB/hr | 24h | > 50MB/hr |

---

## 8. Resource Monitoring

### 8.1 Resources Tracked

```rust
pub struct ResourceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disk: Vec<DiskInfo>,
    pub network: Vec<NetworkInfo>,
    pub gpu: Option<GpuInfo>,
    pub process: ProcessInfo,
}

pub struct CpuInfo {
    pub usage_percent: f64,
    pub per_core: Vec<f64>,
    pub load_average: [f64; 3],
    pub temperature_celsius: Option<f64>,
}

pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub process_rss: u64,
    pub process_vms: u64,
}

pub struct GpuInfo {
    pub name: String,
    pub memory_total: u64,
    pub memory_used: u64,
    pub utilization_percent: f64,
    pub temperature_celsius: Option<f64>,
    pub power_watts: Option<f64>,
}
```

### 8.2 Thresholds & Alerts

| Resource | Warning | Critical | Action |
|----------|---------|----------|--------|
| CPU | > 70% | > 90% | Throttle, alert |
| Memory | > 80% | > 95% | GC, shed load, alert |
| Disk | > 80% | > 95% | Cleanup, alert |
| GPU Memory | > 80% | > 95% | Offload, alert |
| File Descriptors | > 70% | > 90% | Leak check, alert |
| Network Errors | > 1% | > 5% | Diagnose, alert |

---

## 9. Watchdog

### 9.1 Watchdog Types

```rust
pub enum WatchdogType {
    /// Process-level: is main thread responsive?
    Process { interval: Duration, timeout: Duration },
    /// Component-level: is component making progress?
    Component { component: String, heartbeat_interval: Duration },
    /// Deadlock: lock wait graph analysis
    Deadlock { check_interval: Duration },
    /// Resource: leaked resources
    ResourceLeak { check_interval: Duration, threshold: u64 },
    /// Custom: user-defined
    Custom { name: String, check: WatchdogCheckFn },
}
```

### 9.2 Watchdog Actions

```rust
pub enum WatchdogAction {
    Log { level: Level, message: String },
    Alert { severity: AlertSeverity, message: String },
    RestartComponent { component: String, graceful: bool },
    RestartProcess { graceful: bool },
    TriggerDiagnostic { scope: DiagnosticScope },
    EnterRecoveryMode { reason: String },
    ForceKill { target: String },
}
```

---

## 10. Auto Recovery Engine

### 10.1 Recovery Strategies

```rust
pub struct RecoveryStrategy {
    pub name: String,
    pub trigger: RecoveryTrigger,
    pub preconditions: Vec<Precondition>,
    pub steps: Vec<RecoveryStep>,
    pub postconditions: Vec<Postcondition>,
    pub max_attempts: u32,
    pub backoff: BackoffPolicy,
}

pub enum RecoveryTrigger {
    HealthCheckFailed { check_id: String, consecutive: u32 },
    MetricThresholdExceeded { metric: String, threshold: f64 },
    CrashDetected { crash_class: CrashClass },
    WatchdogTimeout { watchdog: String },
    ResourceExhaustion { resource: String },
    Manual { requested_by: Subject },
}

pub enum RecoveryStep {
    RestartComponent { component: String, config: Option<ComponentConfig> },
    ClearCache { cache: String },
    ReconnectDependency { dependency: String },
    ReloadConfig { component: String },
    RunSelfTest { test_id: String },
    ScaleResources { resource: String, amount: i64 },
    Failover { primary: String, backup: String },
    QuarantinePlugin { plugin_id: String },
    ShedLoad { component: String, percentage: f64 },
    ExecuteRemediation { script: String, params: HashMap<String, String> },
    NotifyOperator { message: String, urgency: Urgency },
}
```

### 10.2 Standard Recovery Flows

| Trigger | Strategy |
|---------|----------|
| Plugin crash | Unload → Run self-test → Reload (max 3x) → Quarantine |
| IPC unresponsive | Restart IPC server → Reconnect clients |
| Database unavailable | Reconnect with backoff → Failover to replica |
| Memory pressure | GC → Clear caches → Shed load → Restart component |
| Voice pipeline stall | Reset audio device → Reinitialize pipeline |
| Vision capture fail | Re-enumerate devices → Retry capture |
| Agent deadlock | Force unlock → Restart agent runtime |
| Config corruption | Reload from backup → Validate → Alert |

---

## 11. Predictive Failure Detection

### 11.1 Anomaly Detection

```rust
pub struct PredictiveEngine {
    models: HashMap<String, Box<dyn AnomalyModel>>,
    feature_store: FeatureStore,
    alert_generator: AlertGenerator,
}

pub trait AnomalyModel: Send + Sync {
    fn name(&self) -> &str;
    fn train(&mut self, data: &TrainingData) -> Result<()>;
    fn predict(&self, features: &FeatureVector) -> Prediction;
    fn update(&mut self, features: &FeatureVector, actual: f64) -> Result<()>;
}

pub struct Prediction {
    pub anomaly_score: f64,        // 0.0 - 1.0
    pub predicted_failure: Option<PredictedFailure>,
    pub confidence: f64,
    pub explanation: String,
}

pub struct PredictedFailure {
    pub component: String,
    pub failure_type: String,
    pub estimated_time: DateTime<Utc>,
    pub probability: f64,
    pub recommended_action: String,
}
```

### 11.2 Features

| Feature Category | Examples |
|------------------|----------|
| **Trend** | Memory growth rate, error rate slope, latency trend |
| **Seasonal** | Hourly/daily patterns, usage cycles |
| **Correlation** | CPU ↔ latency, memory ↔ GC pauses |
| **State** | Component state transitions, config changes |
| **External** | OS updates, network changes, dependency versions |

### 11.3 Models

| Model | Use Case | Algorithm |
|-------|----------|-----------|
| `memory_leak` | Detect unbounded memory growth | Linear regression + changepoint |
| `latency_degradation` | Slow increases in latency | EWMA + threshold |
| `error_burst` | Sudden error rate spikes | Poisson process |
| `resource_exhaustion` | Disk, FD, handle leaks | Trend extrapolation |
| `cascading_failure` | Dependency failure propagation | Graph-based |
| `model_drift` | ML model accuracy degradation | Statistical drift detection |

---

## 12. Integration Points

### 12.1 Kernel Integration

```rust
// Kernel registers health checks for all services
impl ManagedService for MyService {
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy // or Degraded/Unhealthy
    }
}
```

### 12.2 Event Bus Integration

```rust
// Health events published to event bus
pub enum HealthEvent {
    CheckCompleted { check_id: String, result: CheckResult },
    StatusChanged { component: String, from: HealthStatus, to: HealthStatus },
    DiagnosticStarted { session_id: Uuid },
    DiagnosticCompleted { session_id: Uuid, findings: Vec<Finding> },
    CrashReported { crash_id: Uuid, report: CrashReport },
    RecoveryStarted { strategy: String, trigger: RecoveryTrigger },
    RecoveryCompleted { strategy: String, success: bool },
    PredictiveAlert { prediction: Prediction },
    ResourceThresholdExceeded { resource: String, value: f64, threshold: f64 },
}
```

### 12.3 IPC Health Endpoint

```rust
// IPC method: voxy.health.check
{
  "scope": "full|component:name|subsystem:name",
  "include_diagnostics": false,
  "include_predictions": false
}

// Response
{
  "overall": "healthy|degraded|unhealthy",
  "checks": [...],
  "diagnostics": [...],
  "predictions": [...],
  "timestamp": "..."
}
```

---

## 13. Configuration

```toml
[health]
enabled = true
check_interval_seconds = 30
deep_check_interval_seconds = 300

[health.thresholds]
cpu_warning = 70
cpu_critical = 90
memory_warning = 80
memory_critical = 95
disk_warning = 80
disk_critical = 95

[health.watchdog]
process_interval_seconds = 10
process_timeout_seconds = 30
deadlock_check_interval_seconds = 60
resource_leak_check_interval_seconds = 300

[health.recovery]
max_restart_attempts = 3
restart_backoff_seconds = 5
enable_auto_recovery = true

[health.predictive]
enabled = true
model_update_interval_hours = 24
anomaly_threshold = 0.8

[health.crash_reporting]
enabled = true
capture_minidump = true
anonymize = true
upload_opt_in = false

[health.performance]
slo_window_days = 30
export_interval_seconds = 15
```

---

## 14. Implementation Roadmap

### Phase 1: Core Health (Week 1)
- [ ] `HealthMonitor` with check registry
- [ ] Built-in system checks
- [ ] `HealthStatus` aggregation
- [ ] Event bus integration
- [ ] Basic IPC health endpoint

### Phase 2: Diagnostics & Self-Test (Week 2)
- [ ] `DiagnosticEngine` with scoped runs
- [ ] `SelfTestFramework` with categories
- [ ] Standard self-tests
- [ ] Diagnostic CLI command

### Phase 3: Crash & Performance (Week 3)
- [ ] `CrashReporter` with minidump
- [ ] `PerformanceMonitor` with SLOs
- [ ] Metrics export (Prometheus)
- [ ] Alerting rules

### Phase 4: Watchdog & Recovery (Week 3-4)
- [ ] `Watchdog` with multiple types
- [ ] `RecoveryEngine` with strategies
- [ ] Standard recovery flows
- [ ] Recovery audit log

### Phase 5: Predictive (Week 4-5)
- [ ] Feature extraction pipeline
- [ ] Baseline anomaly models
- [ ] Model training pipeline
- [ ] Prediction API

### Phase 6: Hardening (Week 5)
- [ ] Chaos testing
- [ ] Failure injection
- [ ] Performance benchmarks
- [ ] Documentation

---

## 15. Review Checklist

- [ ] All subsystems have health checks
- [ ] Checks categorized by criticality
- [ ] Self-tests cover startup + runtime
- [ ] Crash capture works on all platforms
- [ ] SLOs defined and measurable
- [ ] Recovery strategies documented
- [ ] Predictive models have training data
- [ ] Integration with kernel, IPC, event bus
- [ ] Configuration covers all environments
- [ ] Alerting routes defined
- [ ] Runbook links for each alert

---

**Next Step**: Internal review → approve → implement Phase 1