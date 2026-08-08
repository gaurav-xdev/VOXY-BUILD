# VOXY Future Compatibility Review

**Status**: DRAFT — For Internal Review
**Version**: 1.0
**Author**: VOXY Architecture Team
**Date**: 2026-07-24

---

## 1. Purpose

This document validates that **Stage 2 (Infrastructure Layer)** design decisions support all known future VOXY capabilities. Every stage 2 crate (`voxy-ipc`, `voxy-security`, `voxy-health`, `voxy-database`) must be designed so that future stages can integrate without infrastructure rewrites.

---

## 2. Future Capability Compatibility Matrix

### 2.1 Voice Runtime (Human-like Voice)

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **Audio streaming** (microphone → STT, TTS → speaker) | ✅ IPC streaming protocol with `audio` stream type | Stream types defined in `01-ipc-architecture.md` §4.5 |
| **Low latency** (stt < 200ms, tts < 100ms) | ✅ zstd compression, high-priority flag, dedicated stream IDs | QoS via frame flags bit 4 |
| **Wake word** (constant audio monitoring) | ✅ Event protocol with `voxy.voice.wake_word` topic | Event topics in §5.3 |
| **Voice activity detection** (VAD events) | ✅ Event protocol with metadata in payload | Generic event payload schema |
| **Multiple audio devices** (input/output selection) | ✅ Transport abstraction; device enumerations in capability manifest | Devices via platform-core types |
| **Codec negotiation** (Opus, PCM, etc.) | ✅ Stream open with codec parameter in `parameters` field | §4.2 Stream Open |
| **Hotword customization** | ✅ Plugin architecture + storage per-plugin namespace | Storage namespace isolation |

**Conclusion**: ⚠️ Minor gap — audio streaming latency requirements may need dedicated buffer management in IPC layer. Edge-triggered events for VAD and wake word work with standard event channels.

---

### 2.2 Memory System (Episodic, Semantic, Procedural)

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **Episodic memory** (conversation storage) | ✅ Namespaced KV store (`memory.episodic`) | `02-storage-architecture.md` §3 |
| **Semantic memory** (knowledge graph) | ✅ Vector store + KV for relationships | `vector_upsert` + `vector_search` |
| **Procedural memory** (skills, workflows) | ✅ KV store with versioned records | `kv_set` with metadata |
| **Embedding storage** | ✅ Vector namespace with dimension config | `sqlite-vec`, `pgvector`, future DuckDB |
| **Memory decay / TTL** | ✅ `ttl` parameter on `kv_set` | Automatic cleanup |
| **Memory consolidation** (merge short→long term) | ✅ Transactional operations across namespaces | `begin_transaction` + multi-op |
| **Memory search** (hybrid vector + keyword) | ✅ `vector_search` with `filter` parameter | Filter for metadata/keyword pre-filter |
| **Cross-session memory** | ✅ Persistent storage; session IDs in namespace | `conversation.sessions.{id}` |

**Conclusion**: ✅ Fully supported. Memory system only needs `StorageProvider` trait and namespace conventions.

---

### 2.3 Planner (Task Planning & Execution)

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **Plan persistence** (save/load plans) | ✅ KV store with plan namespacing | `memory.procedural.plans` |
| **Plan step execution** (agent coordination) | ✅ IPC request/response for agent method calls | `voxy.agents.*` methods |
| **Plan state machine** | ✅ LifecycleStateMachine in `voxy-state-machine` | State machine crate complete |
| **Plan rollback** (compensation actions) | ✅ Transaction support for atomic operations | `StorageProvider` transactions |
| **Plan scheduling** (timed execution) | ✅ Health layer scheduling + event bus | `DiagnosticTrigger::Scheduled` pattern |
| **Goal decomposition** (sub-plan creation) | ✅ IPC streaming for step-by-step results | Stream close with status stream |

**Conclusion**: ✅ Fully supported. Planner is primarily a consumer of infrastructure, not a driver of infra design.

---

### 2.4 Multi-Agent Runtime

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **Agent ↔ Agent IPC** | ✅ IPC protocol with agent methods; streaming for agent communication | Agent methods defined in §3.3 |
| **Agent lifecycle** (spawn, kill, health) | ✅ Kernel `ManagedService` + IPC `voxy.agents.*` methods | Kernel service registry + IPC handlers |
| **Agent isolation** (capability-based) | ✅ `CapabilityManager` + `CapabilityToken` per agent | Security architecture §2.1 |
| **Agent state persistence** | ✅ Namespace per agent in storage | `agents.{agent_id}` namespace |
| **Agent health monitoring** | ✅ `HealthMonitor` with per-agent checks | Health architecture §3.3 |
| **Agent resource limits** | ✅ `ResourceGovernor` with per-agent budgets | Existing `resource-governor` crate |
| **Agent discovery** (find agent by capability) | ✅ `ManifestRegistry` + `CapabilityRegistry` | Capability manifest registry |

**Conclusion**: ✅ Fully supported. Multi-agent runtime maps naturally onto every Stage 2 component.

---

### 2.5 Desktop Automation (OpenClaw-level)

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **UI element detection** (accessibility API) | ✅ Vision IPC streams + automation backend | Platform-core `WindowPlatform` |
| **Input simulation** (click, type, key) | ✅ IPC `voxy.automation.*` methods | §3.3 Standard Methods |
| **Window management** (list, focus, close) | ✅ Platform-core `WindowPlatform` trait | `platform-core/src/traits.rs` |
| **Screen capture** | ✅ IPC `voxy.vision.capture` method + streaming | Stream type 2001-3000 for video |
| **Permission gating** (sensitive operations) | ✅ `CapabilityManager` — automation:input, screen:capture | Security capability taxonomy |
| **Cross-platform abstraction** | ✅ `Platform` trait with Windows/Linux/macOS impls | Platform layer complete |
| **Script recording/playback** | ✅ IPC streaming for event capture + planner for replay | Stream + planner combination |

**Conclusion**: ✅ Fully supported. Automation needs no Stage 2 changes beyond what's designed.

---

### 2.6 Vision (Screen Capture, OCR, UI Analysis)

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **Screen capture streaming** (high FPS) | ✅ IPC stream type `video` with frame flags | Stream type 2001-3000 |
| **Image analysis** (vision model invoke) | ✅ IPC `voxy.vision.analyze` method | §3.3 Standard Methods |
| **OCR results** (text from screen) | ✅ Event protocol with `voxy.vision.scene_change` | Event topic + payload |
| **Scene change detection** | ✅ Event-driven from `voxy.vision` to `voxy.event-bus` | Event bus pub/sub |
| **Model management** (vision models) | ✅ `ModelRouter` + storage for model artifacts | Model storage via blobs |
| **Privacy gating** (screen capture permission) | ✅ `screen:capture` capability with consent | Consent Manager popup |

**Conclusion**: ⚠️ Minor gap — high-FPS screen capture may stress IPC buffer management. Consider IPC transport-level frame batching and adaptive quality based on available bandwidth.

---

### 2.7 Plugin Ecosystem (WASM/Native)

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **Plugin ↔ Core IPC** | ✅ IPC protocol with full request/response, streaming, events | Dedicated IPC endpoint per plugin |
| **Plugin isolation** (sandboxing) | ✅ Capability-based permission grants + namespace isolation | Security + storage isolation |
| **Plugin lifecycle** (load, unload, update) | ✅ `PluginManager` + `PluginLifecycle` trait | Plugin runtime crate |
| **Plugin data storage** | ✅ Namespace `plugins.{plugin_id}` with quota | Storage isolation + `ResourceGovernor` |
| **Plugin health** | ✅ `HealthMonitor` with per-plugin checks | Health check per plugin |
| **Plugin crash handling** | ✅ `CrashReporter` + `RecoveryEngine` → quarantine | Recovery strategies §10.2 |
| **Plugin update/versioning** | ✅ `PluginManifest` with version constraints + migrated storage | Manifest + storage migrations |
| **Plugin marketplace** (remote registry) | ✅ Remote provider pattern + IPC for distribution | Remote transport + storage |

**Conclusion**: ✅ Fully supported. Plugin isolation is a first-class design concern throughout Stage 2.

---

### 2.8 Home Assistant (IoT, Matter, Smart Home)

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **Device discovery** (matter, thread, zigbee) | ✅ IPC streaming for discovery events + event bus + IPC transport | `voxy.home.devices.*` topics + Stream §4 |
| **Device control** (on/off, dim, color, lock) | ✅ IPC `voxy.home.devices.control` method + capability-gated | Standard method §3.3 |
| **Device state** (polling + event) | ✅ Event bus for state changes + KV for persistent state | Event + storage combo |
| **Automation rules** (if-this-then-that) | ✅ `Planner` + `FeatureFlags` + `PolicyEngine` + `GuardianEngine` | Rule storage + execution + security |
| **Energy monitoring** | ✅ `StorageProvider` time-series + `PredictiveEngine` | `ts_append` + anomaly detection |
| **Multi-protocol** (matter, zigbee, zwave, mqtt, knx) | ✅ Plugin architecture → each protocol = plugin | Plugin runtime + IPC |
| **Smart home platforms** (Home Assistant, SmartThings, Google Home, Apple Home, Alexa) | ✅ `HomeAutomationProvider` trait → each platform = implementation | Provider abstraction §4 |
| **Security integration** (door locks, alarm, cameras) | ✅ CRITICAL capabilities + `GuardianEngine` + consent | Capability risk levels |
| **Emergency actions** (fire, flood, intrusion) | ✅ Guardian mode triggers + high-priority IPC | `GuardianTrigger::Emergency` |
| **Remote access** (mobile companion) | ✅ Remote transport + security + audit | Transport + Identity + Audit |
| **Local control** (no internet) | ✅ LAN transport + local storage provider | Transport abstraction |
| **Voice control** (wake word → home command) | ✅ Voice pipeline + IPC to home provider | Voice + IPC |

#### 2.8.1 Home Automation Provider Interface

```rust
/// Interface for home automation platforms.
/// Each platform implementation (Home Assistant, SmartThings, etc.)
/// implements this trait.
#[async_trait]
pub trait HomeAutomationProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn supported_protocols(&self) -> Vec<Protocol>;
    
    // Device management
    async fn discover_devices(&self) -> Result<Vec<HomeDevice>>;
    async fn get_device(&self, id: &DeviceId) -> Result<Option<HomeDevice>>;
    async fn get_devices_by_room(&self, room: &str) -> Result<Vec<HomeDevice>>;
    
    // Control
    async fn execute_command(&self, device: &DeviceId, command: &HomeCommand) -> Result<HomeCommandResult>;
    async fn execute_commands(&self, commands: &[DeviceCommand]) -> Result<Vec<HomeCommandResult>>;
    
    // State
    async fn get_state(&self, device: &DeviceId) -> Result<DeviceState>;
    async fn subscribe_states(&self, devices: &[DeviceId]) -> Result<Box<dyn StateStream>>;
    
    // Automation
    async fn create_automation(&self, rule: &AutomationRule) -> Result<AutomationId>;
    async fn remove_automation(&self, id: &AutomationId) -> Result<()>;
    async fn list_automations(&self) -> Result<Vec<AutomationRule>>;
    
    // Configuration
    async fn configure(&self, config: &ProviderConfig) -> Result<()>;
    async fn health_check(&self) -> HealthStatus;
}

/// Supported home automation protocols.
pub enum Protocol {
    Matter,
    Zigbee,
    ZWave,
    Mqtt,
    Knx,
    Http,
    WebSocket,
    Bluetooth,
    Thread,
    Proprietary(String),
}

/// Common device types.
pub struct HomeDevice {
    pub id: DeviceId,
    pub name: String,
    pub device_type: DeviceType,
    pub room: Option<String>,
    pub capabilities: Vec<DeviceCapability>,
    pub protocol: Protocol,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub firmware_version: Option<String>,
    pub state: DeviceState,
    pub trust_level: TrustLevel,
}

pub enum DeviceType {
    Light,
    Switch,
    Lock,
    Thermostat,
    Sensor,
    Camera,
    Cover { kind: CoverKind },
    Speaker,
    Display,
    Vacuum,
    Alarm,
    Gateway,
    Custom(String),
}

pub enum DeviceCapability {
    OnOff,
    Brightness,
    ColorTemperature,
    ColorRgb,
    LockUnlock,
    TemperatureRead,
    TemperatureSet,
    HumidityRead,
    MotionDetect,
    ContactDetect,
    EnergyMeter,
    BatteryLevel,
    CameraStream,
    CoverPosition,
    AlarmArm,
}
```

#### 2.8.2 Security for Home Automation

```rust
// Home automation commands are gated by capability + trust level:
// 
// CRITICAL capabilities (require Guardian approval + MFA consent):
// - home:door_lock
// - home:security (alarm arm/disarm)
// - home:emergency (fire suppression, gas shutoff)
// - home:camera (view live stream)
//
// HIGH capabilities (require user consent):
// - home:climate (thermostat control)
// - home:automation (create automation rules)
// - home:energy (manage energy profile)
//
// MEDIUM capabilities (toast notification):
// - home:light (on/off/dim)
// - home:cover (open/close blinds)
// - home:speaker (volume control)
//
// Trust Level checks for home devices:
// - Verified + Trusted → full access within capability bounds
// - Known → MEDIUM commands only, HIGH requires consent
// - Unknown → all home commands blocked
// - Blocked/Compromised → blocked, Guardian alert

impl HomeAutomationProvider {
    /// Verify device trust level before executing command
    async fn verify_trust(&self, device: &HomeDevice, command: &HomeCommand) -> Result<bool> {
        let device_trust = device.trust_level;
        let command_risk = command.risk_level();
        
        match device_trust {
            TrustLevel::Verified => true,
            TrustLevel::Trusted => command_risk <= RiskLevel::High,
            TrustLevel::Known => command_risk <= RiskLevel::Medium,
            _ => false,
        }
    }
}
```

#### 2.8.3 Future Provider Implementations

| Provider | Protocol | Stage | Notes |
|----------|----------|-------|-------|
| **Home Assistant** | WebSocket API | Stage 5 | Most popular OSS home automation |
| **Matter** | IPv6-based | Stage 5 | Industry standard protocol |
| **Zigbee** | 802.15.4 | Stage 5 | Via coordinator plugin |
| **Z-Wave** | Proprietary | Stage 5 | Via dongle plugin |
| **MQTT** | Pub/sub | Stage 5 | Generic IoT protocol |
| **KNX** | Wired | Stage 6 | Building automation standard |
| **Philips Hue** | HTTP API | Stage 5 | Bridge-based lighting |
| **SmartThings** | Cloud API | Stage 5 | Samsung ecosystem |
| **Google Home** | Cloud API | Stage 5 | Google ecosystem |
| **Apple Home** | HAP | Stage 5 | Apple ecosystem |
| **Alexa** | Skill API | Stage 5 | Amazon ecosystem |

**Conclusion**: ✅ Fully supported. `HomeAutomationProvider` trait + capability gating + trust levels cover all future home automation providers.

---

### 2.9 Enterprise Deployment (Multi-Tenant, SSO, Compliance)

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **Multi-tenancy** (isolated tenants) | ✅ Namespace prefix per tenant (`{tenant_id}.config`, etc.) | Storage + capability isolation |
| **SSO / OIDC integration** | ✅ `TokenManager` with OAuth2/OIDC support | Token manager auth methods |
| **Role-based access control (RBAC)** | ✅ `PolicyEngine` with role-based Rego policies | Policy engine §2.6 |
| **Attribute-based access (ABAC)** | ✅ Policy engine supports any attribute | `PolicyInput` structure |
| **Audit logging (compliance-ready)** | ✅ `AuditLog` with tamper-evident hash chain + export | Audit §2.8 |
| **SLA monitoring** | ✅ `SloDefinition` + `PerformanceMonitor` | Health §7.3 |
| **Disaster recovery** | ✅ `BackupStrategy` + remote replication | Storage §10 |
| **Secret rotation** | ✅ `SecretVault` key rotation + `RecoveryMode` | Rotate all secrets on schedule |
| **Compliance reports** (SOC2, HIPAA, FedRAMP) | ✅ Audit export + integrity verification | Audit + integrity combo |
| **Rate limiting / throttling** | ✅ IPC rate limiting + `ResourceGovernor` | IPC §13 + governor |

**Conclusion**: ✅ Fully supported. Enterprise features are a direct consumer of every Stage 2 component.

---

### 2.10 Distributed Execution (Multi-Node, Remote Agents)

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **Remote node IPC** (WAN) | ✅ QUIC/TCP transport + mTLS + auth | Transport §11 + Auth §6 |
| **Node discovery** (find peers) | ✅ Remote transport bootstrapping + registry | Future: service discovery plugin |
| **Agent migration** (move agent across nodes) | ✅ IPC streaming for state transfer + storage for checkpoint | Stream + blob for large state |
| **Distributed storage** (cross-node replication) | ✅ Remote storage provider with consistency modes | Remote provider §3.5 |
| **Clock sync** (cross-node timing) | ✅ `HealthMonitor` clock check + NTP integration | `clock_sync` self-test |
| **Network partition** (split-brain handling) | ✅ Guardian mode + quorum-based decisions | Guardian §2.10 |
| **Federation** (independent clusters) | ✅ Remote transport + hierarchical policy | Policy distributed via bundles |
| **Edge deployment** (low-resource nodes) | ✅ Configurable SQLite provider + file vault | Lightweight deployment profile |

**Conclusion**: ⚠️ Minor gap — true distributed consensus (Raft/Paxos) needed for replicated state. Stage 2 provides transport and auth; consensus is a Stage 4+ concern.

---

### 2.11 Robotics (ROS2, Real-time, Sensors)

| Requirement | Stage 2 Support | Notes |
|-------------|-----------------|-------|
| **Sensor data streaming** (high-frequency) | ✅ IPC streaming with high-priority flag | Stream §4 + priority flag |
| **Real-time constraints** (< 1ms control loop) | ⚠️ Partial — IPC transport may need RTOS integration | Future: shared memory transport |
| **ROS2 integration** | ✅ Plugin architecture for ROS2 bridge | Plugin with ROS2 IPC transport |
| **Motor control** (PWM, servo) | ✅ IPC automation methods + security gating | `automation:*` capabilities |
| **SLAM / navigation** | ✅ Vision streams + vector storage for maps | Vision + storage |
| **Safety constraints** (emergency stop) | ✅ Guardian mode + watchdog with fail-safe | Guardian mode §2.10 |
| **Resource constraints** (embedded) | ✅ Configurable providers, SQLite + file vault | Lightweight profile |
| **Firmware updates** | ✅ Blob storage + plugin update mechanism | Blob + plugin lifecycle |

**Conclusion**: ⚠️ Gap — real-time IPC (< 1ms) needs shared-memory transport (Stage 5+). Non-real-time robotics is fully supported.

---

## 3. Gap Analysis Summary

| Gap | Severity | Affected Capabilities | Mitigation |
|-----|----------|-----------------------|------------|
| Real-time IPC (< 1ms) | Low | Robotics | Add `shared_memory` transport in later stage |
| Consensus/replication | Low | Distributed execution | Stage 4 concern; transport + auth ready |
| Distributed transactions | Low | Enterprise, distributed | Saga pattern in Stage 4 |
| High-FPS screen capture bandwidth | Low | Vision, automation | IPC frame batching; adaptive quality |
| Audio streaming buffer tuning | Low | Voice, home | Configurable buffer sizes in IPC config |
| Plugin sandboxing (WASM) | Low | Plugin ecosystem | Stage 5; IPC + security foundation ready |

**Overall Assessment**: No Stage 2 redesigns are required to support any future capability. All gaps are either:
- Addressed by configuration/parametrization of existing Stage 2 components
- Properly scoped for later stages with clean interfaces

---

## 4. Future Extension Points

### 4.1 IPC Extension Points

```rust
// New transport implementations require only:
#[async_trait]
impl Transport for SharedMemoryTransport {
    fn scheme(&self) -> &str { "shm" }
    async fn connect(&self, endpoint: &Endpoint) -> Result<Self::Connection> { ... }
    async fn listen(&self, endpoint: &Endpoint) -> Result<Self::Listener> { ... }
}

// Register via TransportRegistry:
transport_registry.register("shm", Arc::new(SharedMemoryTransport::new()));
```

### 4.2 Storage Extension Points

```rust
// New storage backend:
pub struct ScyllaDbProvider { /* ... */ }

#[async_trait]
impl StorageProvider for ScyllaDbProvider { /* ... */ }

// Register via:
StorageRegistry::register("scylladb", Arc::new(ScyllaDbProvider::new()));
```

### 4.3 Security Extension Points

```rust
// New vault backend:
pub struct YubiHsmBackend { /* ... */ }

#[async_trait]
impl VaultBackend for YubiHsmBackend { /* ... */ }

// New audit backend:
pub struct ElasticsearchAuditBackend { /* ... */ }

#[async_trait]
impl AuditBackend for ElasticsearchAuditBackend { /* ... */ }
```

### 4.4 Health Extension Points

```rust
// New anomaly model:
pub struct LstmAnomalyDetector { /* ... */ }

#[async_trait]
impl AnomalyModel for LstmAnomalyDetector { /* ... */ }
```

---

## 5. Backward Compatibility Guarantees

### 5.1 Trait Stability

Once Stage 2 is stabilized:
- `StorageProvider` — MAJOR version changes only
- `IpcClient` / `IpcServer` — MAJOR version changes only  
- `Transport` — MAJOR version changes only
- `CapabilityManager` — MAJOR version changes only
- `HealthMonitor` — MAJOR version changes only

### 5.2 Protocol Stability

- Wire format (frame header) — MAJOR changes only
- Control message types — NEW types only, no removal
- Event topics — NEW topics only, no removal
- IPC methods — NEW methods only, no removal

### 5.3 Storage Stability

- `StorageProvider` trait — additive changes only
- Migration system — forward-compatible by design
- Namespace convention — additive only

---

## 6. Timeline Mapping

```
Stage 2 (Current)
├── Infrastructure Layer
├── IPC Core Protocol
├── Storage Provider Trait + SQLite
├── Security Foundation
└── Health Foundation

Stage 3 (Next)
├── Voice Pipeline
├── Vision Pipeline  
├── Agent Runtime
└── Provider Implementations

Stage 4
├── Plugin Sandboxing (WASM)
├── Multi-Agent Coordination
├── Advanced Planner
└── Memory Consolidation

Stage 5
├── Desktop Automation
├── Home Assistant Integration
├── Distributed Consensus
└── Remote/Mobile Companion

Stage 6+
├── Robotics Integration
├── Enterprise RBAC + SSO
├── Advanced ML Anomaly Detection
├── Federated Learning
└── Adaptive Real-time IPC
```

---

## 7. Recommendation

**Stage 2 infrastructure design is compatible with all 11 future capabilities.**

Zero redesigns needed. All gaps are properly scoped for later stages.

Proceed with Stage 2 implementation as designed.

---

## 8. Review Checklist

- [ ] Voice runtime — streaming, latency, codec support
- [ ] Memory system — namespaces, vectors, TTL
- [ ] Planner — persistence, state machine, scheduling
- [ ] Multi-agent — IPC, isolation, health, discovery
- [ ] Desktop automation — input, vision, permissions
- [ ] Vision — streaming, analysis, scene detection
- [ ] Plugin ecosystem — IPC, isolation, data, lifecycle
- [ ] Home automation — devices, rules, protocols
- [ ] Enterprise — multi-tenant, SSO, RBAC, audit
- [ ] Distributed execution — remote IPC, discovery, failover
- [ ] Robotics — real-time, sensors, safety
- [ ] Extension points documented for all components
- [ ] Backward compatibility guarantees defined

---

**Next Step**: Internal review → approve → Stage 2 implementation ready to begin.