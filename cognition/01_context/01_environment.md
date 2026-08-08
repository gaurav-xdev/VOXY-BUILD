# Environment Context

## Purpose

The Environment Context module provides the COS with awareness of the physical and computational environment in which the user operates. It answers the questions: *When is it? Where is the user? What is the machine doing? What is the network state?* This context is consumed by the `ContextAssembler` as a `ContextSource::WorldModel` source, producing the `WorldSnapshot` that flows into intent analysis, reasoning, planning, and tool selection.

## Responsibilities

1. **Temporal awareness**: Current time, date, timezone, day-of-week, working hours
2. **Location awareness**: Geographic coordinates, timezone, indoor/outdoor detection
3. **Ambient awareness**: Light level, noise level, temperature
4. **System state awareness**: CPU load, memory pressure, disk usage, battery state
5. **Network awareness**: Internet connectivity, connection type, latency, bandwidth
6. **Application awareness**: Foreground window, focused application, active workspace
7. **Display awareness**: Connected displays, resolution, DPI, active display
8. **Power awareness**: AC/battery, charge level, estimated runtime, power profile
9. **OS event awareness**: Sleep/wake, lid open/close, display on/off, user lock/unlock

## Internal Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                     ENVIRONMENT CONTEXT                              │
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │  Temporal Source  │  │  System Source   │  │  Network Source   │  │
│  │  (clock, tz)     │  │  (cpu, mem, bat) │  │  (connectivity)  │  │
│  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘  │
│           │                     │                      │             │
│           ▼                     ▼                      ▼             │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              EnvironmentCollector                             │  │
│  │  Polls OS APIs, system monitors, network probes              │  │
│  │  Produces: UserEnvironment, AmbientConditions, DesktopState  │  │
│  └──────────────────────────┬───────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              WorldSnapshot Assembly                           │  │
│  │  Combines: DesktopState + UserEnvironment + Devices + Tasks  │  │
│  │  Output: WorldSnapshot (consumed by WorldModelProvider)      │  │
│  └──────────────────────────┬───────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              Context Source                                   │  │
│  │  Wraps WorldSnapshot as ContextSource::WorldModel             │  │
│  │  Feeds into ContextAssembler.assemble()                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### OS-Level Signals

```rust
pub struct OsEnvironmentSignal {
    /// Timestamp of observation
    pub observed_at: DateTime<Utc>,
    
    /// Time source (NTP-synchronized system clock)
    pub time: TimeSignal,
    
    /// System resource signals
    pub system: SystemSignal,
    
    /// Network signals
    pub network: NetworkSignal,
    
    /// Display signals
    pub display: DisplaySignal,
    
    /// Power signals
    pub power: PowerSignal,
    
    /// OS lifecycle signals
    pub lifecycle: LifecycleSignal,
}

pub struct TimeSignal {
    /// Current UTC time
    pub utc_now: DateTime<Utc>,
    
    /// Local timezone identifier (IANA)
    pub timezone: String,
    
    /// Local time offset from UTC
    pub utc_offset: FixedOffset,
    
    /// Day of week (1=Monday, 7=Sunday)
    pub day_of_week: u8,
    
    /// Hour of day (0-23) in local time
    pub hour_local: u8,
    
    /// Whether current time falls in typical working hours
    pub is_working_hours: bool,
    
    /// Whether current time is nighttime
    pub is_night: bool,
}

pub struct SystemSignal {
    /// CPU load average (1-minute)
    pub cpu_load_1m: f32,
    
    /// CPU load average (5-minute)
    pub cpu_load_5m: f32,
    
    /// CPU load average (15-minute)
    pub cpu_load_15m: f32,
    
    /// CPU core count
    pub cpu_cores: u32,
    
    /// Memory total (bytes)
    pub memory_total: u64,
    
    /// Memory used (bytes)
    pub memory_used: u64,
    
    /// Memory available (bytes)
    pub memory_available: u64,
    
    /// Swap total (bytes)
    pub swap_total: u64,
    
    /// Swap used (bytes)
    pub swap_used: u64,
    
    /// Disk total (bytes)
    pub disk_total: u64,
    
    /// Disk used (bytes)
    pub disk_used: u64,
    
    /// Disk I/O read (bytes/sec)
    pub disk_io_read: u64,
    
    /// Disk I/O write (bytes/sec)
    pub disk_io_write: u64,
    
    /// System uptime (seconds)
    pub uptime_seconds: u64,
    
    /// Process count
    pub process_count: u32,
    
    /// Load classification
    pub load_class: SystemLoadClass,
}

pub enum SystemLoadClass {
    Idle,
    Light,
    Moderate,
    Heavy,
    Critical,
}

pub struct NetworkSignal {
    /// Internet available
    pub internet_available: bool,
    
    /// Connection type
    pub connection_type: ConnectionType,
    
    /// Download bandwidth (Mbps)
    pub download_mbps: Option<f64>,
    
    /// Upload bandwidth (Mbps)
    pub upload_mbps: Option<f64>,
    
    /// Latency to DNS server (ms)
    pub latency_ms: Option<f64>,
    
    /// Connection quality score (0.0-1.0)
    pub quality_score: f64,
    
    /// VPN active
    pub vpn_active: bool,
    
    /// Proxy configured
    pub proxy_configured: bool,
    
    /// Active network interface name
    pub interface_name: Option<String>,
    
    /// IP address (private, not logged)
    pub ip_address: Option<String>,
}

pub enum ConnectionType {
    Wifi,
    Ethernet,
    Cellular,
    Cellular4g,
    Cellular5g,
    Vpn,
    Unknown,
}

pub struct DisplaySignal {
    /// Number of connected displays
    pub display_count: u32,
    
    /// Active display index
    pub active_display: u32,
    
    /// Display configurations
    pub displays: Vec<DisplayConfig>,
    
    /// Primary display width (pixels)
    pub primary_width: u32,
    
    /// Primary display height (pixels)
    pub primary_height: u32,
    
    /// Primary display DPI
    pub primary_dpi: u32,
    
    /// Display scale factor
    pub scale_factor: f64,
    
    /// Night mode active
    pub night_mode: bool,
    
    /// Display brightness (0-100)
    pub brightness: Option<u8>,
}

pub struct DisplayConfig {
    /// Display identifier
    pub id: String,
    
    /// Display name
    pub name: String,
    
    /// Width (pixels)
    pub width: u32,
    
    /// Height (pixels)
    pub height: u32,
    
    /// DPI
    pub dpi: u32,
    
    /// Is primary
    pub is_primary: bool,
    
    /// Is built-in (laptop screen)
    pub is_builtin: bool,
}

pub struct PowerSignal {
    /// AC power connected
    pub ac_power: bool,
    
    /// Battery present
    pub battery_present: bool,
    
    /// Battery charge level (0-100)
    pub battery_level: Option<u8>,
    
    /// Battery charging state
    pub charging_state: ChargingState,
    
    /// Estimated time remaining (seconds)
    pub time_remaining: Option<u64>,
    
    /// Power profile
    pub power_profile: PowerProfile,
    
    /// Battery health (0-100)
    pub battery_health: Option<u8>,
    
    /// Battery cycle count
    pub cycle_count: Option<u32>,
}

pub enum ChargingState {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

pub enum PowerProfile {
    Balanced,
    Performance,
    PowerSaver,
    Custom(String),
}

pub struct LifecycleSignal {
    /// System just woke from sleep
    pub just_woke: bool,
    
    /// Lid is open (laptop)
    pub lid_open: bool,
    
    /// Display is on
    pub display_on: bool,
    
    /// User is logged in
    pub user_logged_in: bool,
    
    /// User is locked (screen locked)
    pub user_locked: bool,
    
    /// System is idle (no user input for N seconds)
    pub system_idle: bool,
    
    /// Idle duration (seconds)
    pub idle_duration_seconds: u64,
    
    /// Last user input timestamp
    pub last_input_at: Option<DateTime<Utc>>,
    
    /// System sleep count since boot
    pub sleep_count: u32,
}
```

## Outputs

### Environment Snapshot

```rust
pub struct EnvironmentSnapshot {
    /// Snapshot identifier
    pub id: String,
    
    /// Capture timestamp
    pub captured_at: DateTime<Utc>,
    
    /// Temporal context
    pub temporal: TemporalContext,
    
    /// System context
    pub system: SystemContext,
    
    /// Network context
    pub network: NetworkContext,
    
    /// Display context
    pub display: DisplayContext,
    
    /// Power context
    pub power: PowerContext,
    
    /// Lifecycle context
    pub lifecycle: LifecycleContext,
    
    /// Data freshness (seconds since capture)
    pub freshness: u64,
    
    /// Confidence in snapshot accuracy
    pub confidence: f64,
}

pub struct TemporalContext {
    /// ISO 8601 timestamp
    pub timestamp: DateTime<Utc>,
    
    /// IANA timezone string
    pub timezone: String,
    
    /// UTC offset in minutes
    pub utc_offset_minutes: i32,
    
    /// Day of week
    pub day_of_week: DayOfWeek,
    
    /// Month
    pub month: u8,
    
    /// Day of month
    pub day: u8,
    
    /// Hour in local time
    pub hour: u8,
    
    /// Minute in local time
    pub minute: u8,
    
    /// Is working hours (configurable, default 9-17 weekdays)
    pub is_working_hours: bool,
    
    /// Is weekend
    pub is_weekend: bool,
    
    /// Is nighttime (configurable, default 22-6)
    pub is_night: bool,
    
    /// Is holiday (if calendar integration enabled)
    pub is_holiday: bool,
}

pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

pub struct SystemContext {
    /// CPU load classification
    pub load_class: SystemLoadClass,
    
    /// Memory pressure classification
    pub memory_pressure: MemoryPressure,
    
    /// Disk pressure classification
    pub disk_pressure: DiskPressure,
    
    /// System uptime (human-readable)
    pub uptime_human: String,
    
    /// Number of running processes
    pub process_count: u32,
    
    /// System is under resource stress
    pub under_stress: bool,
    
    /// Recommended action for resource stress
    pub stress_recommendation: Option<String>,
}

pub enum MemoryPressure {
    Low,
    Moderate,
    High,
    Critical,
}

pub enum DiskPressure {
    Low,
    Moderate,
    High,
    Critical,
}

pub struct NetworkContext {
    /// Internet available
    pub available: bool,
    
    /// Connection type
    pub connection_type: ConnectionType,
    
    /// Quality classification
    pub quality: NetworkQuality,
    
    /// Bandwidth classification
    pub bandwidth: BandwidthClass,
    
    /// Latency classification
    pub latency_class: LatencyClass,
    
    /// VPN active
    pub vpn_active: bool,
    
    /// Safe to use bandwidth-intensive operations
    pub bandwidth_safe: bool,
    
    /// Safe to use latency-sensitive operations
    pub latency_safe: bool,
}

pub enum NetworkQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Offline,
}

pub enum BandwidthClass {
    High,
    Medium,
    Low,
    None,
}

pub enum LatencyClass {
    Low,
    Medium,
    High,
    Unknown,
}

pub struct DisplayContext {
    /// Number of displays
    pub count: u32,
    
    /// Display configuration summary
    pub summary: String,
    
    /// Primary resolution
    pub primary_resolution: String,
    
    /// DPI classification
    pub dpi_class: DpiClass,
    
    /// Night mode active
    pub night_mode: bool,
    
    /// Screen reader active
    pub screen_reader_active: bool,
    
    /// Accessibility features active
    pub accessibility_features: Vec<String>,
}

pub enum DpiClass {
    Low,
    Medium,
    High,
    Retina,
}

pub struct PowerContext {
    /// On AC power
    pub on_ac: bool,
    
    /// Battery level percentage
    pub battery_percent: Option<u8>,
    
    /// Battery status classification
    pub battery_status: BatteryStatus,
    
    /// Estimated time remaining
    pub time_remaining: Option<Duration>,
    
    /// Power profile
    pub profile: PowerProfile,
    
    /// Safe to perform CPU-intensive operations
    pub cpu_safe: bool,
    
    /// Safe to perform disk-intensive operations
    pub disk_safe: bool,
}

pub enum BatteryStatus {
    High,
    Medium,
    Low,
    Critical,
    Charging,
    Full,
    NoBattery,
}

pub struct LifecycleContext {
    /// System just woke from sleep
    pub just_woke: bool,
    
    /// Lid is open
    pub lid_open: bool,
    
    /// Display is on
    pub display_on: bool,
    
    /// User is locked
    pub user_locked: bool,
    
    /// System idle
    pub is_idle: bool,
    
    /// Idle duration
    pub idle_duration: Duration,
    
    /// Time since last user input
    pub time_since_last_input: Option<Duration>,
    
    /// Should reduce background activity
    pub reduce_background: bool,
    
    /// Should pause non-critical operations
    pub pause_non_critical: bool,
}
```

## State Transitions

```
┌─────────────────────────────────────────────────────────────────────┐
│                 ENVIRONMENT CONTEXT LIFECYCLE                        │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │   INITIALIZING   │                                               │
│  └────────┬─────────┘                                               │
│           │ (OS APIs queried)                                       │
│           ▼                                                          │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │    MONITORING    │────▶│   DEGRADED       │                     │
│  └────────┬─────────┘     └────────┬─────────┘                     │
│           │ (OS signal change)      │ (OS API failure)              │
│           ▼                         ▼                                │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   UPDATING       │     │   CACHING        │                     │
│  └────────┬─────────┘     └────────┬─────────┘                     │
│           │ (snapshot produced)     │ (use cached snapshot)          │
│           ▼                         ▼                                │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   PUBLISHING     │     │   RECOVERING     │                     │
│  └────────┬─────────┘     └────────┬─────────┘                     │
│           │ (event emitted)         │ (API restored)                │
│           ▼                         ▼                                │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   MONITORING     │◀────│   MONITORING     │                     │
│  └──────────────────┘     └──────────────────┘                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Decision Logic

### Polling Strategy

```rust
/// Environment context polling intervals
pub struct PollingConfig {
    /// Time signal: update every 60 seconds
    pub time_interval: Duration,
    
    /// System signal: update every 5 seconds under normal load,
    /// every 1 second under high load
    pub system_interval_normal: Duration,
    pub system_interval_stressed: Duration,
    
    /// Network signal: update every 30 seconds
    pub network_interval: Duration,
    
    /// Display signal: update on OS event (no polling needed)
    pub display_event_driven: bool,
    
    /// Power signal: update every 60 seconds, or on AC/battery transition
    pub power_interval: Duration,
    
    /// Lifecycle signal: update on OS event (no polling needed)
    pub lifecycle_event_driven: bool,
}
```

### Stress Detection

```rust
fn classify_system_stress(system: &SystemSignal) -> SystemLoadClass {
    let cpu_score = classify_cpu_load(system.cpu_load_1m, system.cpu_cores);
    let memory_score = classify_memory_pressure(system.memory_used, system.memory_total);
    let disk_score = classify_disk_pressure(system.disk_used, system.disk_total);
    
    // Use worst-case classification
    let worst = [cpu_score, memory_score, disk_score]
        .into_iter()
        .max_by_key(|c| match c {
            SystemLoadClass::Critical => 4,
            SystemLoadClass::Heavy => 3,
            SystemLoadClass::Moderate => 2,
            SystemLoadClass::Light => 1,
            SystemLoadClass::Idle => 0,
        })
        .unwrap_or(SystemLoadClass::Idle);
    
    worst
}
```

## Failure Modes

| Mode | Detection | Recovery | Prevention |
|------|-----------|----------|------------|
| OS API timeout | Read timeout > 2s | Use cached values, log warning | Adaptive timeout, retry with backoff |
| OS API unavailable | Repeated failures | Switch to event-driven where possible | Graceful degradation per signal type |
| Stale data | Freshness > threshold | Log warning, reduce confidence | Adaptive polling frequency |
| Permission denied | Permission error | Skip signal, log info | Request permissions at startup |
| Clock skew | NTP sync failure | Warn user, use local clock | Monitor NTP sync status |

### Recovery Strategy

```rust
impl EnvironmentMonitor {
    async fn recover_from_api_failure(&self, failed_signal: &str) {
        match failed_signal {
            "system" => {
                // System API failed: increase polling interval, use cached
                self.system_poll_interval.store(
                    self.system_interval_stressed.as_millis() as u64,
                    Ordering::Relaxed,
                );
                tracing::warn!("System API failed, switching to cached values");
            }
            "network" => {
                // Network API failed: assume offline, verify later
                self.network_available.store(false, Ordering::Relaxed);
                self.schedule_network_retry();
            }
            "power" => {
                // Power API failed: assume AC power, safe default
                self.ac_power.store(true, Ordering::Relaxed);
                tracing::warn!("Power API failed, assuming AC power");
            }
            _ => {
                tracing::warn!(signal = failed_signal, "Unknown environment signal failed");
            }
        }
    }
}
```

## Privacy Considerations

1. **Location data**: Latitude/longitude are never logged or transmitted. Location is used only for timezone inference and local context. Users can disable location access entirely.
2. **IP address**: Stored only in memory, never persisted, never transmitted. Used only for network quality inference.
3. **Screen content**: Screen content is analyzed locally, never transmitted. OCR results are processed in-memory.
4. **System metrics**: CPU/memory/disk metrics are aggregated and anonymized. Per-process data is not collected.
5. **Battery data**: Battery health data is used only for power management decisions. Not transmitted.
6. **User activity**: Idle time and input timestamps are used only for context. Not logged in detail.
7. **Display data**: Display configuration is used for DPI-aware rendering. Not associated with user identity.

## Security Considerations

1. **OS API authentication**: All OS-level API calls use the current user's security context. No privilege escalation.
2. **Network probing**: Network quality probes use only ICMP/DNS to known-safe endpoints. No data transmission.
3. **Memory protection**: Environment data is stored in process memory, not shared with other processes.
4. **Input validation**: All OS signals are validated before use. Malformed data triggers graceful degradation.
5. **Permission model**: Each signal type requires explicit permission. Permissions are checked at collection time.

## Future Extensibility

1. **Calendar integration**: Import calendar events for richer temporal context (meetings, appointments)
2. **Weather API**: Real weather data via API (with user permission) for ambient awareness
3. **Location services**: Indoor positioning via WiFi, Bluetooth beacons
4. **Smart home integration**: Home device state for ambient context
5. **Health sensors**: Heart rate, posture data from wearables
6. **Multi-machine awareness**: Context from multiple devices the user operates
7. **Predictive context**: Predict environment changes before they occur

## Examples

### Example 1: Working Hours Context

```
Signal: TimeSignal { hour_local: 14, day_of_week: Wednesday, is_working_hours: true }
System: SystemSignal { cpu_load_1m: 2.1, cpu_cores: 8, memory_used: 12GB, memory_total: 32GB }
Network: NetworkSignal { internet_available: true, connection_type: Ethernet, latency_ms: 5.2 }
Display: DisplaySignal { display_count: 2, primary_width: 2560, primary_height: 1440 }
Power: PowerSignal { ac_power: true, battery_level: None }
Result: EnvironmentSnapshot {
    temporal: { is_working_hours: true, is_weekend: false, is_night: false },
    system: { load_class: Moderate, memory_pressure: Low, under_stress: false },
    network: { available: true, quality: Excellent, bandwidth_safe: true },
    display: { count: 2, dpi_class: High, night_mode: false },
    power: { on_ac: true, cpu_safe: true, disk_safe: true },
    lifecycle: { reduce_background: false, pause_non_critical: false },
}
```

### Example 2: Nighttime Laptop on Battery

```
Signal: TimeSignal { hour_local: 23, is_night: true }
System: SystemSignal { cpu_load_1m: 0.3, memory_used: 8GB, memory_total: 16GB }
Power: PowerSignal { ac_power: false, battery_level: 35, charging_state: Discharging }
Lifecycle: LifecycleSignal { lid_open: true, display_on: true, idle_duration_seconds: 300 }
Result: EnvironmentSnapshot {
    temporal: { is_working_hours: false, is_night: true },
    system: { load_class: Idle, under_stress: false },
    power: { on_ac: false, battery_percent: 35, battery_status: Medium, cpu_safe: false },
    lifecycle: { is_idle: true, idle_duration: 300s, reduce_background: true, pause_non_critical: true },
}
```

### Example 3: System Under Stress

```
Signal: SystemSignal { cpu_load_1m: 14.2, cpu_cores: 8, memory_used: 30GB, memory_total: 32GB }
Result: SystemLoadClass::Critical, MemoryPressure::Critical
EnvironmentSnapshot.system.under_stress = true
EnvironmentSnapshot.system.stress_recommendation = "Consider closing memory-intensive applications"
EnvironmentSnapshot.power.cpu_safe = false
EnvironmentSnapshot.lifecycle.pause_non_critical = true
```

## Engineering Notes

- All timestamps use `chrono::DateTime<Utc>` for consistency
- Timezone is detected from OS, stored as IANA string
- System metrics are collected via platform-specific APIs (Windows: WMI/Performance Counters, Linux: /proc, macOS: sysctl)
- Network probing uses DNS resolution + HTTP HEAD to a configurable endpoint
- Display information is platform-specific (Windows: EnumDisplayDevices, Linux: xrandr, macOS: CGDisplay)
- Power information uses OS power management APIs (Windows: PowerGetActiveScheme, Linux: /sys/class/power_supply, macOS: IOPSCopyPowerSourcesInfo)
- Environment snapshots are stored in a ring buffer of configurable size (default: 60 snapshots)
- Freshness is calculated as `Utc::now() - captured_at`
- Confidence degrades linearly with freshness: `confidence = max(0.0, 1.0 - (freshness_seconds / max_freshness_seconds))`
