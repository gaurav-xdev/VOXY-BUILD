# Device Context

## Purpose

The Device Context module provides the COS with awareness of connected hardware devices. It tracks USB devices, Bluetooth peripherals, printers, monitors, storage, cameras, microphones, sensors, location, and smart home devices. It answers: *What devices are connected? What peripherals are available? What sensors are active? What is the device topology?*

## Responsibilities

1. **USB device tracking**: Detect and track USB devices
2. **Bluetooth device tracking**: Detect and track Bluetooth peripherals
3. **Printer awareness**: Detect connected printers
4. **Monitor awareness**: Track connected displays
5. **Storage awareness**: Track connected storage devices
6. **Camera awareness**: Detect connected cameras
7. **Microphone awareness**: Detect connected microphones
8. **Sensor awareness**: Track device sensors (accelerometer, gyroscope, GPS)
9. **Location awareness**: Track device location (if available)
10. **Smart home device tracking**: Track connected smart home devices

## Internal Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                       DEVICE CONTEXT                                 │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    INPUT SOURCES                              │   │
│  │                                                               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐   │   │
│  │  │  USB    │  │ Bluetooth│  │  OS     │  │  Network     │   │   │
│  │  │ Monitor │  │ Monitor │  │ Events  │  │  Discovery   │   │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬───────┘   │   │
│  │       │            │            │               │            │   │
│  └───────┼────────────┼────────────┼───────────────┼────────────┘   │
│          │            │            │               │                 │
│          ▼            ▼            ▼               ▼                 │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              DeviceContextManager                             │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Device Registry                                        │  │   │
│  │  │  - Add device                                           │  │   │
│  │  │  - Remove device                                        │  │   │
│  │  │  - Update device                                        │  │   │
│  │  │  - Query devices                                        │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Device Classification                                  │  │   │
│  │  │  - Categorize device                                    │  │   │
│  │  │  - Assess capabilities                                  │  │   │
│  │  │  - Determine relevance                                  │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              DeviceSnapshot                                   │   │
│  │  Point-in-time view of all connected devices                 │   │
│  │  Consumed by: Cognition, Automation, Grounding               │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Device Signals

```rust
pub struct DeviceSignal {
    /// Signal identifier
    pub id: String,
    
    /// Signal type
    pub signal_type: DeviceSignalType,
    
    /// Signal timestamp
    pub observed_at: DateTime<Utc>,
    
    /// Signal confidence
    pub confidence: f64,
    
    /// Signal data
    pub data: serde_json::Value,
}

pub enum DeviceSignalType {
    /// USB device connected
    UsbConnected {
        device: UsbDevice,
    },
    
    /// USB device disconnected
    UsbDisconnected {
        device_id: String,
    },
    
    /// USB device updated
    UsbUpdated {
        device_id: String,
        changes: HashMap<String, String>,
    },
    
    /// Bluetooth device connected
    BluetoothConnected {
        device: BluetoothDevice,
    },
    
    /// Bluetooth device disconnected
    BluetoothDisconnected {
        device_id: String,
    },
    
    /// Printer detected
    PrinterDetected {
        printer: PrinterDevice,
    },
    
    /// Monitor connected
    MonitorConnected {
        monitor: MonitorDevice,
    },
    
    /// Monitor disconnected
    MonitorDisconnected {
        monitor_id: String,
    },
    
    /// Storage device connected
    StorageConnected {
        storage: StorageDevice,
    },
    
    /// Storage device disconnected
    StorageDisconnected {
        storage_id: String,
    },
    
    /// Camera detected
    CameraDetected {
        camera: CameraDevice,
    },
    
    /// Microphone detected
    MicrophoneDetected {
        microphone: MicrophoneDevice,
    },
    
    /// Sensor reading
    SensorReading {
        sensor_id: String,
        reading: SensorReading,
    },
    
    /// Location updated
    LocationUpdated {
        location: LocationData,
    },
    
    /// Smart home device discovered
    SmartHomeDevice {
        device: SmartHomeDevice,
    },
}

pub struct UsbDevice {
    pub id: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor_name: String,
    pub product_name: String,
    pub device_type: UsbDeviceType,
    pub speed: UsbSpeed,
    pub connected_at: DateTime<Utc>,
}

pub enum UsbDeviceType {
    Storage,
    Input,
    Audio,
    Video,
    Printer,
    Network,
    Hub,
    Unknown,
}

pub enum UsbSpeed {
    Low,
    Full,
    High,
    Super,
    SuperPlus,
}

pub struct BluetoothDevice {
    pub id: String,
    pub name: String,
    pub device_type: BluetoothDeviceType,
    pub mac_address: String,
    pub signal_strength: i8,
    pub connected_at: DateTime<Utc>,
    pub battery_level: Option<u8>,
}

pub enum BluetoothDeviceType {
    Headphones,
    Speaker,
    Keyboard,
    Mouse,
    Gamepad,
    Watch,
    Fitness,
    Medical,
    Unknown,
}

pub struct PrinterDevice {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub status: PrinterStatus,
    pub capabilities: Vec<String>,
    pub paper_sizes: Vec<String>,
    pub ink_levels: Option<InkLevels>,
}

pub enum PrinterStatus {
    Ready,
    Printing,
    Error,
    Offline,
}

pub struct InkLevels {
    pub black: Option<u8>,
    pub cyan: Option<u8>,
    pub magenta: Option<u8>,
    pub yellow: Option<u8>,
}

pub struct MonitorDevice {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub is_primary: bool,
    pub is_builtin: bool,
    pub dpi: u32,
}

pub struct StorageDevice {
    pub id: String,
    pub name: String,
    pub device_type: StorageType,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub filesystem: String,
    pub mount_point: String,
}

pub enum StorageType {
    Hdd,
    Ssd,
    Nvme,
    Usb,
    Network,
    Optical,
}

pub struct CameraDevice {
    pub id: String,
    pub name: String,
    pub resolution: CameraResolution,
    pub fps: u32,
    pub is_builtin: bool,
    pub has_privacy_shutter: bool,
}

pub struct CameraResolution {
    pub width: u32,
    pub height: u32,
}

pub struct MicrophoneDevice {
    pub id: String,
    pub name: String,
    pub device_type: MicrophoneType,
    pub channels: u32,
    pub sample_rate: u32,
    pub is_builtin: bool,
    pub has_noise_cancellation: bool,
}

pub enum MicrophoneType {
    BuiltIn,
    External,
    Headset,
    Conference,
    Professional,
}

pub struct SensorReading {
    pub sensor_id: String,
    pub sensor_type: SensorType,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
}

pub enum SensorType {
    Accelerometer,
    Gyroscope,
    Magnetometer,
    Barometer,
    AmbientLight,
    Proximity,
    Temperature,
    Humidity,
    Gps,
}

pub struct LocationData {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: f64,
    pub timestamp: DateTime<Utc>,
    pub source: LocationSource,
}

pub enum LocationSource {
    Gps,
    Wifi,
    CellTower,
    IpGeolocation,
    Manual,
}

pub struct SmartHomeDevice {
    pub id: String,
    pub name: String,
    pub device_type: SmartHomeDeviceType,
    pub manufacturer: String,
    pub model: String,
    pub state: HashMap<String, String>,
    pub capabilities: Vec<String>,
    pub room: Option<String>,
}

pub enum SmartHomeDeviceType {
    Light,
    Thermostat,
    Lock,
    Camera,
    Speaker,
    Tv,
    Plug,
    Sensor,
    Hub,
    Unknown,
}
```

## Outputs

### Device Snapshot

```rust
pub struct DeviceSnapshot {
    /// Snapshot identifier
    pub id: String,
    
    /// Capture timestamp
    pub captured_at: DateTime<Utc>,
    
    /// Connected USB devices
    pub usb_devices: Vec<UsbDevice>,
    
    /// Connected Bluetooth devices
    pub bluetooth_devices: Vec<BluetoothDevice>,
    
    /// Connected printers
    pub printers: Vec<PrinterDevice>,
    
    /// Connected monitors
    pub monitors: Vec<MonitorDevice>,
    
    /// Connected storage devices
    pub storage_devices: Vec<StorageDevice>,
    
    /// Connected cameras
    pub cameras: Vec<CameraDevice>,
    
    /// Connected microphones
    pub microphones: Vec<MicrophoneDevice>,
    
    /// Active sensors
    pub sensors: Vec<SensorInfo>,
    
    /// Current location
    pub location: Option<LocationInfo>,
    
    /// Smart home devices
    pub smart_home_devices: Vec<SmartHomeDevice>,
    
    /// Device summary
    pub summary: DeviceSummary,
    
    /// Data freshness
    pub freshness: u64,
    
    /// Confidence in snapshot
    pub confidence: f64,
}

pub struct SensorInfo {
    pub sensor_type: SensorType,
    pub reading: f64,
    pub unit: String,
    pub last_updated: DateTime<Utc>,
    pub is_active: bool,
}

pub struct LocationInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub source: LocationSource,
    pub last_updated: DateTime<Utc>,
    pub indoor_outdoor: Option<IndoorOutdoor>,
}

pub enum IndoorOutdoor {
    Indoor,
    Outdoor,
    Unknown,
}

pub struct DeviceSummary {
    pub total_devices: u32,
    pub usb_count: u32,
    pub bluetooth_count: u32,
    pub printer_count: u32,
    pub monitor_count: u32,
    pub storage_count: u32,
    pub camera_count: u32,
    pub microphone_count: u32,
    pub sensor_count: u32,
    pub smart_home_count: u32,
    pub has_external_display: bool,
    pub has_external_storage: bool,
    pub has_audio_devices: bool,
    pub has_input_devices: bool,
}
```

## State Transitions

```
┌─────────────────────────────────────────────────────────────────────┐
│                  DEVICE CONTEXT STATE MACHINE                        │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │   INITIALIZING   │                                               │
│  └────────┬─────────┘                                               │
│           │ (device monitoring started)                             │
│           ▼                                                          │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   MONITORING     │────▶│   DEGRADED       │                     │
│  └────────┬─────────┘     └────────┬─────────┘                     │
│           │ (device change)         │ (monitoring restored)          │
│           ▼                         ▼                                │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   UPDATING       │────▶│   MONITORING     │                     │
│  └────────┬─────────┘     └──────────────────┘                     │
│           │ (update complete)                                       │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   MONITORING     │                                               │
│  └──────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Decision Logic

### When to Track a Device

```rust
fn should_track_device(
    device: &UsbDevice,
    config: &DeviceConfig,
) -> bool {
    // Always track known device types
    if matches!(device.device_type, UsbDeviceType::Storage | UsbDeviceType::Input | UsbDeviceType::Audio) {
        return true;
    }
    
    // Track if device has known vendor/product ID
    if is_known_device(device.vendor_id, device.product_id) {
        return true;
    }
    
    // Track if configured to track all devices
    if config.track_all_devices {
        return true;
    }
    
    false
}
```

### When to Query Location

```rust
fn should_query_location(
    device_snapshot: &DeviceSnapshot,
    config: &DeviceConfig,
) -> bool {
    // Query if location permission granted
    if !config.location_permission_granted {
        return false;
    }
    
    // Query if location is stale
    if let Some(location) = &device_snapshot.location {
        let age = Utc::now() - location.last_updated;
        if age > Duration::from_secs(config.location_staleness_seconds) {
            return true;
        }
    } else {
        // No location yet, query if permission granted
        return config.location_permission_granted;
    }
    
    false
}
```

## Failure Modes

| Mode | Detection | Recovery | Prevention |
|------|-----------|----------|------------|
| USB monitoring failure | No USB events | Retry monitoring | Multiple monitoring backends |
| Bluetooth monitoring failure | No BT events | Retry monitoring | Fallback to polling |
| Location permission denied | Permission error | Use last known location | Request permission at startup |
| Device enumeration slow | Timeout | Use cached device list | Background enumeration |
| Smart home API failure | API error | Use cached device states | Retry with backoff |
| Sensor reading failure | Invalid reading | Ignore reading | Sensor validation |

### Recovery Strategy

```rust
impl DeviceContextManager {
    async fn recover_from_monitoring_failure(&self, device_type: &str) {
        match device_type {
            "usb" => {
                tracing::warn!("USB monitoring failed, falling back to polling");
                self.fallback_to_usb_polling().await;
            }
            "bluetooth" => {
                tracing::warn!("Bluetooth monitoring failed, retrying");
                self.retry_bluetooth_monitoring().await;
            }
            "location" => {
                tracing::warn!("Location query failed, using cached location");
                self.use_cached_location();
            }
            _ => {
                tracing::warn!(device_type, "Unknown device monitoring failure");
            }
        }
    }
}
```

## Privacy Considerations

1. **Device tracking**: Device information is stored locally, never transmitted.
2. **Location data**: Location data is stored locally, never transmitted without explicit consent.
3. **Smart home devices**: Smart home device states are stored locally.
4. **Camera/microphone**: Camera and microphone device information is stored locally.
5. **User control**: Users can disable device tracking for any device type.
6. **No profiling**: Device data is not used for advertising or profiling.
7. **Data retention**: Device data is retained according to user-configured policy.

## Security Considerations

1. **Device authentication**: Only authenticated devices are tracked.
2. **Permission model**: Location and sensor access require explicit permission.
3. **Secure storage**: Device data is stored in encrypted local database.
4. **Access control**: Only authorized COS components can access device data.
5. **Audit logging**: Device access is auditable.
6. **No remote access**: Device data never leaves the device without explicit consent.

## Future Extensibility

1. **Device automation**: Automatically respond to device connections
2. **Device profiles**: Custom profiles for different device configurations
3. **Device analytics**: Analyze device usage patterns
4. **Device sharing**: Share device information across user's devices
5. **Device security**: Enhanced security for connected devices
6. **Device health**: Monitor device health and predict failures
7. **Device recommendations**: Recommend devices based on usage patterns

## Examples

### Example 1: USB Storage Connected

```
Signal: UsbConnected { device: UsbDevice { name: "USB Drive", device_type: Storage, total_bytes: 64GB } }
DeviceSnapshot: { storage_devices: [..., new_device], summary: { has_external_storage: true } }
Action: Cognition can now access files on USB drive
```

### Example 2: Bluetooth Headphones Connected

```
Signal: BluetoothConnected { device: BluetoothDevice { name: "AirPods", device_type: Headphones, battery_level: 80 } }
DeviceSnapshot: { bluetooth_devices: [..., new_device], summary: { has_audio_devices: true } }
Action: VoicePipeline can route audio to headphones
```

### Example 3: Multi-Monitor Setup

```
Signal: MonitorConnected { monitor: MonitorDevice { name: "Dell U2720Q", width: 3840, height: 2160, is_primary: false } }
DeviceSnapshot: { monitors: [builtin, new_monitor], summary: { has_external_display: true } }
Action: VisualContext can capture from external display
```

## Engineering Notes

- USB monitoring uses platform-specific APIs (Windows: SetupAPI, Linux: udev, macOS: IOKit)
- Bluetooth monitoring uses platform-specific APIs (Windows: BluetoothAPIs, Linux: BlueZ, macOS: CoreBluetooth)
- Location uses platform-specific APIs (Windows: Windows.Devices.Geolocation, Linux: GeoClue, macOS: Core Location)
- Smart home uses protocol-specific APIs (HomeKit, Matter, etc.)
- Device enumeration runs in background to avoid blocking main thread
- Device changes are event-driven where possible, polling as fallback
- All timestamps use `chrono::DateTime<Utc>` for consistency
