use std::fmt;

#[derive(Debug, Clone)]
pub enum HardwareEvent {
    DeviceConnected {
        device_id: String,
        device_type: String,
    },
    DeviceDisconnected {
        device_id: String,
        device_type: String,
    },
    DeviceError {
        device_id: String,
        error: String,
    },
    AudioLevelChanged {
        device_id: String,
        level: f32,
    },
    HardwareStatusChanged {
        status: String,
        details: Option<String>,
    },
}

impl fmt::Display for HardwareEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceConnected {
                device_id,
                device_type,
            } => {
                write!(f, "Device connected: {} ({})", device_id, device_type)
            }
            Self::DeviceDisconnected {
                device_id,
                device_type,
            } => {
                write!(f, "Device disconnected: {} ({})", device_id, device_type)
            }
            Self::DeviceError { device_id, error } => {
                write!(f, "Device error: {} - {}", device_id, error)
            }
            Self::AudioLevelChanged { device_id, level } => {
                write!(f, "Audio level changed: {} level={}", device_id, level)
            }
            Self::HardwareStatusChanged { status, details } => {
                if let Some(d) = details {
                    write!(f, "Hardware status changed: {} ({})", status, d)
                } else {
                    write!(f, "Hardware status changed: {}", status)
                }
            }
        }
    }
}
