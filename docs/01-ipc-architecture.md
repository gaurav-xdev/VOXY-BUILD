# VOXY IPC Architecture Design

**Status**: DRAFT — For Internal Review
**Version**: 1.0
**Author**: VOXY Architecture Team
**Date**: 2026-07-24

---

## 1. Overview

The IPC (Inter-Process Communication) layer is the nervous system of VOXY. It connects:

- **Daemon ↔ Overlay** (local desktop)
- **Core ↔ Plugins** (sandboxed WASM/native)
- **Agent ↔ Agent** (multi-agent runtime)
- **Core ↔ Voice Backend** (STT/TTS/wake word)
- **Core ↔ Vision Backend** (screen capture/OCR/UI analysis)
- **Core ↔ Automation Backend** (UI interaction)
- **Core ↔ Home Backend** (IoT/matter/thread)
- **Remote Node ↔ Remote Node** (distributed execution)
- **Mobile Companion ↔ Core** (remote control)

### Design Principles

1. **Protocol-First**: Wire format defined before implementation
2. **Transport-Agnostic**: Named pipes, Unix sockets, TCP, WebSocket, QUIC
3. **Capability-Based Auth**: Every message carries capability token
4. **Versioned Protocol**: Semantic versioning with negotiation
5. **Streaming-First**: Async streams for audio, video, events
6. **Event Replay**: Events can be consumed live, replayed, or snapshotted
7. **Observable**: Built-in tracing, metrics, dead-letter queue
8. **Resilient**: Automatic reconnection, circuit breakers, backpressure

---

## 2. Wire Protocol

### 2.1 Frame Format (Binary)

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRAME HEADER (24 bytes)                   │
├──────────┬──────────┬──────────┬──────────┬──────────┬──────────┤
│ Magic    │ Version  │ Flags    │ Stream ID │ Length   │ Checksum │
│ (4 bytes)│ (2 bytes)│ (2 bytes)│ (8 bytes) │ (8 bytes)│ (4 bytes)│
├──────────┴──────────┴──────────┴──────────┴──────────┴──────────┤
│                        PAYLOAD (variable)                        │
└─────────────────────────────────────────────────────────────────┘
```

| Field | Size | Description |
|-------|------|-------------|
| Magic | 4 bytes | `0x56 0x4F 0x58 0x59` ("VOXY") |
| Version | 2 bytes | Protocol version (major.minor) |
| Flags | 2 bytes | Bitfield (see below) |
| Stream ID | 8 bytes | `0` = control, `>0` = data stream |
| Length | 8 bytes | Payload length (big-endian) |
| Checksum | 4 bytes | CRC32C of header + payload |

### 2.2 Flag Bits

| Bit | Name | Description |
|-----|------|-------------|
| 0 | `COMPRESSED` | Payload compressed (zstd) |
| 1 | `ENCRYPTED` | Payload encrypted (ChaCha20-Poly1305) |
| 2 | `FRAGMENTED` | Part of fragmented message |
| 3 | `LAST_FRAGMENT` | Final fragment |
| 4 | `PRIORITY` | High priority (bypass queue) |
| 5 | `HEARTBEAT` | Heartbeat frame |
| 6 | `CANCELLATION` | Cancellation signal |
| 7 | `ACK_REQUIRED` | Requires explicit ACK |
| 8-15 | Reserved | Future use |

### 2.3 Message Types (Control Stream ID = 0)

| Type ID | Name | Direction | Description |
|---------|------|-----------|-------------|
| 0x01 | `HELLO` | C→S | Client hello with capabilities |
| 0x02 | `WELCOME` | S→C | Server welcome, assigned session ID |
| 0x03 | `AUTH_CHALLENGE` | S→C | Authentication challenge |
| 0x04 | `AUTH_RESPONSE` | C→S | Authentication response |
| 0x05 | `AUTH_RESULT` | S→C | Authentication result |
| 0x10 | `REQUEST` | C→S | Request/response RPC |
| 0x11 | `RESPONSE` | S→C | RPC response |
| 0x12 | `STREAM_OPEN` | Both | Open data stream |
| 0x13 | `STREAM_DATA` | Both | Stream data chunk |
| 0x14 | `STREAM_CLOSE` | Both | Close stream |
| 0x20 | `EVENT_SUBSCRIBE` | C→S | Subscribe to event topic |
| 0x21 | `EVENT_UNSUBSCRIBE` | C→S | Unsubscribe |
| 0x22 | `EVENT_NOTIFY` | S→C | Event notification |
| 0x30 | `HEARTBEAT` | Both | Keep-alive |
| 0x31 | `PING` | Both | Latency measurement |
| 0x32 | `PONG` | Both | Ping response |
| 0x40 | `CANCEL` | Both | Cancel request/stream |
| 0xFF | `ERROR` | Both | Protocol error |

---

## 3. Request/Response Protocol

### 3.1 Request Frame (Type 0x10)

```json
{
  "id": "uuid-v4",
  "method": "string",
  "params": {},
  "timeout_ms": 30000,
  "capabilities": ["cap:id"],
  "trace_context": {
    "trace_id": "hex",
    "span_id": "hex",
    "trace_flags": 1
  },
  "metadata": {}
}
```

### 3.2 Response Frame (Type 0x11)

```json
{
  "id": "uuid-v4",
  "status": "ok | error | cancelled | timeout",
  "result": {},
  "error": {
    "code": -32600,
    "message": "string",
    "data": {}
  },
  "trace_context": {}
}
```

### 3.3 Standard Methods

| Method | Description |
|--------|-------------|
| `voxy.health.check` | Health check |
| `voxy.capabilities.list` | List available capabilities |
| `voxy.capabilities.check` | Check capability granted |
| `voxy.config.get` | Get configuration |
| `voxy.config.set` | Set configuration |
| `voxy.plugins.list` | List loaded plugins |
| `voxy.plugins.load` | Load plugin |
| `voxy.plugins.unload` | Unload plugin |
| `voxy.agents.list` | List agents |
| `voxy.agents.spawn` | Spawn agent |
| `voxy.agents.kill` | Kill agent |
| `voxy.memory.store` | Store memory |
| `voxy.memory.retrieve` | Retrieve memory |
| `voxy.memory.search` | Search memory |
| `voxy.voice.transcribe` | Speech to text |
| `voxy.voice.synthesize` | Text to speech |
| `voxy.vision.analyze` | Analyze image |
| `voxy.vision.capture` | Capture screen |
| `voxy.automation.click` | Click at coordinates |
| `voxy.automation.type` | Type text |
| `voxy.automation.screenshot` | Take screenshot |
| `voxy.home.devices.list` | List home devices |
| `voxy.home.devices.control` | Control device |

---

## 4. Streaming Protocol

### 4.1 Stream Lifecycle

```
OPEN → DATA* → CLOSE
  │         │
  └── CANCEL (optional, either direction)
```

### 4.2 Stream Open (Type 0x12)

```json
{
  "stream_id": "uint64",
  "type": "audio | video | events | binary",
  "direction": "send | recv | duplex",
  "codec": "opus | h264 | raw | json",
  "parameters": {
    "sample_rate": 16000,
    "channels": 1,
    "bitrate": 32000
  },
  "capabilities": ["audio:capture"]
}
```

### 4.3 Stream Data (Type 0x13)

Binary payload with stream ID in frame header. Sequence numbers in frame flags (bits 8-15).

### 4.4 Stream Close (Type 0x14)

```json
{
  "stream_id": "uint64",
  "reason": "completed | error | cancelled | timeout",
  "error": {}
}
```

### 4.5 Standard Stream Types

| Stream ID Range | Type | Use Case |
|-----------------|------|----------|
| 1-1000 | Audio Input | Microphone → STT |
| 1001-2000 | Audio Output | TTS → Speaker |
| 2001-3000 | Video | Screen → Vision |
| 3001-4000 | Events | High-frequency events |
| 4001-5000 | Binary | File transfer, model weights |

---

## 5. Event Protocol

### 5.1 Subscription (Type 0x20)

```json
{
  "subscription_id": "uuid-v4",
  "topic": "voxy.voice.*",
  "filter": {
    "capabilities": ["audio:capture"],
    "source": "plugin:whisper"
  },
  "delivery": "at_least_once | at_most_once"
}
```

### 5.2 Notification (Type 0x22)

```json
{
  "subscription_id": "uuid-v4",
  "topic": "voxy.voice.transcript",
  "timestamp": "ISO8601",
  "payload": {},
  "trace_context": {}
}
```

### 5.3 Standard Event Topics

| Topic | Payload | Description |
|-------|---------|-------------|
| `voxy.voice.wake_word` | `{detector, confidence}` | Wake word detected |
| `voxy.voice.transcript` | `{text, language, confidence, is_final}` | STT result |
| `voxy.vision.scene_change` | `{scene_type, confidence}` | Scene changed |
| `voxy.automation.action` | `{action, target, result}` | Automation executed |
| `voxy.agent.spawned` | `{agent_id, type, config}` | Agent started |
| `voxy.agent.completed` | `{agent_id, result}` | Agent finished |
| `voxy.plugin.loaded` | `{plugin_id, manifest}` | Plugin loaded |
| `voxy.plugin.error` | `{plugin_id, error}` | Plugin error |
| `voxy.health.degraded` | `{component, reason}` | Health degraded |
| `voxy.security.consent_required` | `{capability, context}` | User consent needed |

---

## 6. Authentication & Authorization

### 6.1 Authentication Flow

```
Client                          Server
  │                                │
  ├── HELLO (caps, version) ──────►│
  │                                │
  │◄── AUTH_CHALLENGE (nonce) ─────┤
  │                                │
  ├── AUTH_RESPONSE (sig, token) ──►│
  │                                │
  │◄── AUTH_RESULT (session_id) ───┤
  │                                │
  ├── REQUEST (with session_id) ───►│
  │                                │
```

### 6.2 Authentication Methods

| Method | Description | Use Case |
|--------|-------------|----------|
| `token` | Pre-shared capability token | Plugin ↔ Core |
| `mutual_tls` | mTLS with client certs | Remote nodes |
| `oauth2` | OAuth 2.0 bearer token | Mobile companion |
| `device_code` | Device authorization flow | Headless devices |

### 6.3 Capability Token (JWT-like)

```json
{
  "sub": "plugin:whisper",
  "iat": 1700000000,
  "exp": 1700086400,
  "caps": ["audio:capture", "model:inference"],
  "scope": "local",
  "nonce": "random"
}
```

Signed with Ed25519. Public keys distributed via capability manifest.

### 6.4 Authorization on Each Request

Every `REQUEST` frame includes `capabilities` array. Server validates:
1. Token valid (signature, expiry, nonce)
2. Token contains all required capabilities
3. Capability matches method (e.g., `voxy.voice.transcribe` requires `audio:capture`)

---

## 7. Version Negotiation

### 7.1 Hello Exchange

```
Client: HELLO { supported_versions: [1, 2], min_version: 1 }
Server: WELCOME { version: 2, features: [...] }
```

### 7.2 Version Scheme

- **Major**: Breaking wire format changes
- **Minor**: New message types, optional fields (backward compatible)
- **Patch**: Bug fixes only

### 7.3 Compatibility Rules

- Server MUST support N-1 major versions
- Client SHOULD negotiate highest common version
- Unknown message types → ignore (forward compatibility)
- Unknown fields → ignore (forward compatibility)

---

## 8. Heartbeat & Keep-Alive

### 8.1 Heartbeat Interval

- Default: 30 seconds
- Configurable per-connection
- Minimum: 5 seconds

### 8.2 Heartbeat Frame (Type 0x30)

```json
{
  "timestamp": "ISO8601",
  "sequence": 42,
  "load": {
    "cpu_percent": 12.5,
    "memory_mb": 256,
    "active_streams": 3,
    "pending_requests": 7
  }
}
```

### 8.3 Failure Detection

- Missed 3 heartbeats → connection unhealthy
- Missed 5 heartbeats → force close
- Client can send `PING` (0x31) for RTT measurement

---

## 9. Cancellation

### 9.1 Request Cancellation

Client sends `CANCEL` (0x40) with request ID:

```json
{
  "request_id": "uuid",
  "reason": "user_cancelled | timeout | superseded"
}
```

Server responds with `RESPONSE` status `cancelled`.

### 9.2 Stream Cancellation

Either party sends `STREAM_CLOSE` with `reason: cancelled`.

### 9.3 Connection-Level Cancellation

`CANCEL` with `request_id: "all"` cancels all pending operations.

---

## 10. Compression

### 10.1 When to Compress

- Payload > 1 KB
- Compressible content (JSON, text, protobuf)
- Not already compressed (audio/video)

### 10.2 Algorithm

- **zstd** (default): Level 3, good ratio/speed
- **lz4**: For ultra-low latency (audio metadata)
- **none**: For already-compressed streams

### 10.3 Negotiation

`HELLO` includes `compression: ["zstd", "lz4"]`. `WELCOME` selects one.

---

## 11. Transport Abstraction

### 11.1 Transport Trait

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&self, endpoint: &Endpoint) -> Result<Connection>;
    async fn listen(&self, endpoint: &Endpoint) -> Result<Listener>;
    fn scheme(&self) -> &str; // "ipc", "tcp", "ws", "quic"
    fn max_frame_size(&self) -> usize;
}
```

### 11.2 Supported Transports

| Scheme | Implementation | Use Case |
|--------|----------------|----------|
| `ipc` | Named pipes (Windows) / Unix sockets (Linux/macOS) | Local daemon ↔ overlay, plugins |
| `tcp` | TCP + TLS | Remote nodes, LAN |
| `ws` | WebSocket + TLS | Browser, mobile companion |
| `quic` | QUIC (quinn) | High-performance remote, NAT traversal |

### 11.3 Endpoint Format

```
ipc:///pipe/voxy-daemon           # Windows named pipe
ipc:///run/voxy/daemon.sock       # Unix socket
tcp://192.168.1.100:8080          # TCP
tls://voxy.example.com:8443       # TLS
ws://localhost:8080/ipc           # WebSocket
wss://voxy.example.com/ipc        # Secure WebSocket
quic://voxy.example.com:8443      # QUIC
```

---

## 12. Connection Lifecycle

```
CONNECTING → AUTHENTICATING → AUTHENTICATED → READY
                                      ↓
                                 DISCONNECTING → CLOSED
                                      ↓
                                   RECONNECTING → CONNECTING
```

### 12.1 Reconnection Policy

- Exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s (max)
- Jitter: ±25%
- Max retries: infinite (configurable)
- Session resumption: Server issues `session_token` on auth; client presents on reconnect

---

## 13. Security Considerations

1. **All connections encrypted in production** (TLS 1.3 / QUIC / Noise)
2. **Capability tokens short-lived** (default 1 hour, rotatable)
3. **Rate limiting** per connection (1000 req/s default)
4. **Frame size limits** (default 16 MB, configurable)
5. **Stream count limits** (default 64 concurrent)
6. **Audit logging** for all auth events and capability checks

---

## 14. Observability

### 14.1 Metrics (Prometheus)

| Metric | Type | Labels |
|--------|------|--------|
| `voxy_ipc_connections_total` | Counter | `transport`, `state` |
| `voxy_ipc_frames_total` | Counter | `direction`, `type` |
| `voxy_ipc_frame_bytes` | Histogram | `direction`, `compressed` |
| `voxy_ipc_request_duration_seconds` | Histogram | `method`, `status` |
| `voxy_ipc_stream_active` | Gauge | `type` |
| `voxy_ipc_errors_total` | Counter | `transport`, `error_type` |

### 14.2 Tracing

Every frame carries `trace_context` (W3C TraceContext). Spans created for:
- Connection establishment
- Authentication
- Each request/response
- Stream open/close

### 14.3 Dead Letter Queue

Failed deliveries (serialization error, handler panic, timeout) → DLQ with:
- Original frame
- Error details
- Timestamp
- Retry count

---

## 15. Implementation Plan

### Phase 1: Core Protocol (Week 1-2)
- [ ] Frame codec (encode/decode)
- [ ] Message types & serialization
- [ ] In-memory transport (testing)

### Phase 2: IPC Transport (Week 2-3)
- [ ] Windows named pipes
- [ ] Unix domain sockets
- [ ] Connection pooling

### Phase 3: Auth & Versioning (Week 3-4)
- [ ] Capability token validation
- [ ] Version negotiation
- [ ] Session resumption

### Phase 4: Streaming & Events (Week 4-5)
- [ ] Stream lifecycle management
- [ ] Backpressure handling
- [ ] Event subscription engine

### Phase 5: Advanced Features (Week 5-6)
- [ ] TLS/QUIC transports
- [ ] Compression
- [ ] Circuit breaker
- [ ] Load balancing (multiple endpoints)

### Phase 6: Observability & Hardening (Week 6-7)
- [ ] Metrics export
- [ ] Distributed tracing
- [ ] Chaos testing
- [ ] Fuzzing

---

## 16. API Surface (Rust)

```rust
// Core traits
#[async_trait]
pub trait IpcClient: Send + Sync {
    async fn connect(&mut self, endpoint: &Endpoint) -> Result<()>;
    async fn request(&self, req: Request) -> Result<Response>;
    async fn open_stream(&self, params: StreamParams) -> Result<Stream>;
    async fn subscribe(&self, topic: &str, filter: EventFilter) -> Result<Subscription>;
    async fn close(&mut self) -> Result<()>;
}

#[async_trait]
pub trait IpcServer: Send + Sync {
    async fn bind(&mut self, endpoint: &Endpoint) -> Result<()>;
    async fn register_handler(&self, method: &str, handler: Box<dyn RequestHandler>);
    async fn register_stream_handler(&self, stream_type: &str, handler: Box<dyn StreamHandler>);
    async fn broadcast_event(&self, topic: &str, event: Event) -> Result<()>;
    async fn run(&mut self) -> Result<()>;
}

// Transport abstraction
#[async_trait]
pub trait Transport: Send + Sync {
    type Connection: Connection;
    type Listener: Listener;
    
    async fn connect(&self, endpoint: &Endpoint) -> Result<Self::Connection>;
    async fn listen(&self, endpoint: &Endpoint) -> Result<Self::Listener>;
}

// Codec
pub trait Codec: Send + Sync {
    fn encode(&self, frame: &Frame) -> Result<Vec<u8>>;
    fn decode(&self, bytes: &[u8]) -> Result<Frame>;
}
```

---

## 16. Event Replay

The Event Bus supports four consumption modes for every event topic. This enables crash recovery, snapshot restoration, and historical analysis.

### 16.1 Consumption Modes

```rust
pub enum EventConsumptionMode {
    /// Live events only. Default mode.
    /// Events published after subscription are delivered.
    Live,
    
    /// Historical replay from a point in time.
    /// Events between `from` and `now` are delivered in order,
    /// then transitions to live mode.
    Replay { from: DateTime<Utc> },
    
    /// Full snapshot of current state for a topic.
    /// Returns the latest known state for each event key,
    /// no live streaming.
    Snapshot { topic: String },
    
    /// Complete event history for a time range.
    /// All stored events in the range, no live transition.
    History { from: DateTime<Utc, to: DateTime<Utc> },
    
    /// Recovery mode: replay events from a crash checkpoint,
    /// then verify state consistency, then switch to live.
    Recovery { 
        checkpoint_id: String,
        verify_consistency: bool,
    },
}
```

### 16.2 Replay Protocol

```json
// Client requests replay subscription (Type 0x20 extended)
{
  "subscription_id": "uuid-v4",
  "topic": "voxy.voice.transcript",
  "mode": "replay",
  "replay_from": "2026-07-24T10:00:00Z",
  "replay_rate": 2.0,    // 2x speed
  "filter": {},
  "delivery": "at_least_once"
}

// Server response (Type 0x20 ACK)
{
  "subscription_id": "uuid-v4",
  "mode": "replay",
  "replay_status": {
    "total_events": 15420,
    "estimated_duration_ms": 5000,
    "from": "2026-07-24T10:00:00Z",
    "to": "2026-07-24T12:00:00Z"
  }
}

// Server pushes events via EVENT_NOTIFY (Type 0x22)
// When replay catches up to live, sends REPLAY_COMPLETE:
{
  "subscription_id": "uuid-v4",
  "event": "replay.complete",
  "payload": {
    "events_replayed": 15420,
    "duration_ms": 4875,
    "switched_to_live": true
  }
}
```

### 16.3 Recovery Mode Protocol

```
Crash Detected
  │
  ▼
Client: SUBSCRIBE (mode: "recovery", checkpoint_id: "...")
  │
  ▼
Server: Replays events since checkpoint
  │
  ▼
Server: Sends RECOVERY_CHECKPOINT with reconciled state
  │
  ▼
Client: Verifies state consistency
  │  ✅ → RECOVERY_ACK → switch to live
  │  ❌ → RECOVERY_FAILED → initiate full recovery
  │
  ▼
Server: EVENT_NOTIFY (live)
```

```json
// Recovery checkpoint
{
  "subscription_id": "uuid-v4",
  "event": "recovery.checkpoint",
  "payload": {
    "last_replayed_event_id": "evt_99999",
    "state_hash": "sha256:abc123...",
    "events_since_checkpoint": 42,
    "next_event_id": "evt_100000"
  }
}
```

### 16.4 Event Storage

Events are stored in the `StorageProvider` with TTL-based retention:

```rust
pub struct StoredEvent {
    pub event_id: String,
    pub topic: String,
    pub payload: Vec<u8>,
    pub metadata: EventMetadata,
    pub timestamp: DateTime<Utc>,
    pub ttl: Duration,
}

impl EventStore {
    /// Store event for replay
    pub async fn store(&self, topic: &str, event: &StoredEvent) -> Result<()>;
    
    /// Replay events in a time range
    pub async fn replay(&self, topic: &str, range: TimeRange, rate: f64) -> Result<EventStream>;
    
    /// Get snapshot for a topic (latest state per key)
    pub async fn snapshot(&self, topic: &str) -> Result<HashMap<String, StoredEvent>>;
    
    /// Get recovery events since checkpoint
    pub async fn recovery(&self, checkpoint_id: &str) -> Result<Vec<StoredEvent>>;
    
    /// Clean up expired events
    pub async fn cleanup_expired(&self) -> Result<u64>;
}
```

### 16.5 Retention Policies

| Topic Pattern | Retention | Purpose |
|---------------|-----------|---------|
| `voxy.voice.*` | 24 hours | Voice transcripts |
| `voxy.vision.*` | 1 hour | Scene changes |
| `voxy.automation.*` | 7 days | Automation history |
| `voxy.security.*` | 90 days | Security events |
| `voxy.audit.*` | 1 year | Compliance |
| `voxy.agent.*` | 30 days | Agent lifecycle |
| `voxy.plugin.*` | 30 days | Plugin events |

### 16.6 Event Replay Metrics

| Metric | Type | Labels |
|--------|------|--------|
| `voxy_events_replayed_total` | Counter | `topic`, `mode` |
| `voxy_events_stored_total` | Counter | `topic` |
| `voxy_events_replay_duration_seconds` | Histogram | `topic`, `mode` |
| `voxy_events_storage_size_bytes` | Gauge | `topic` |

| Feature | Timeline | Notes |
|---------|----------|-------|
| QUIC transport | Stage 3 | For WAN, mobile |
| WASM plugin IPC | Stage 3 | Shared memory + message passing |
| RDMA support | Stage 5 | HPC/robotics |
| Message routing mesh | Stage 4 | Multi-node clusters |
| Protocol buffers alt | Optional | For high-perf internal |

---

## 18. Review Checklist

- [ ] Protocol spec complete
- [ ] All message types defined
- [ ] Auth flow documented
- [ ] Version strategy clear
- [ ] Transport abstraction sufficient
- [ ] Streaming semantics precise
- [ ] Error handling comprehensive
- [ ] Observability built-in
- [ ] Security model reviewed
- [ ] Backward compatibility guaranteed
- [ ] Performance targets defined
- [ ] Test strategy documented

---

**Next Step**: Internal review → approve → implement Phase 1