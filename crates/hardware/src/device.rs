use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    Microphone,
    Speaker,
    Camera,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Microphone => write!(f, "Microphone"),
            Self::Speaker => write!(f, "Speaker"),
            Self::Camera => write!(f, "Camera"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceStatus {
    Available,
    Busy,
    Disconnected,
    Error,
}

impl fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available => write!(f, "Available"),
            Self::Busy => write!(f, "Busy"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
}
