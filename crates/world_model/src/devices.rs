use std::collections::HashMap;
use voxy_hardware::device::DeviceStatus;

pub use voxy_hardware::device::DeviceType;

#[derive(Debug, Clone)]
pub struct ConnectedDevice {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub status: DeviceStatus,
    pub room_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxy_hardware::device::DeviceStatus;

    #[test]
    fn test_connected_device_creation() {
        let device = ConnectedDevice {
            id: "dev1".to_string(),
            name: "Office Camera".to_string(),
            device_type: "Camera".to_string(),
            status: DeviceStatus::Available,
            room_id: Some("office".to_string()),
            metadata: [("resolution".to_string(), "1080p".to_string())].into(),
        };
        assert_eq!(device.id, "dev1");
        assert_eq!(device.name, "Office Camera");
        assert_eq!(device.device_type, "Camera");
        assert_eq!(device.status, DeviceStatus::Available);
        assert_eq!(device.room_id.unwrap(), "office");
    }

    #[test]
    fn test_device_status_integration() {
        let available = DeviceStatus::Available;
        let busy = DeviceStatus::Busy;
        assert_eq!(format!("{}", available), "Available");
        assert_eq!(format!("{}", busy), "Busy");
        assert_ne!(available, busy);
    }
}
