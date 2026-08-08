use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;

use crate::config::HomeConfig;
use crate::error::Result;
use crate::event::HomeEvent;

#[derive(Debug, Clone)]
pub struct RoomSpec {
    pub id: String,
    pub name: String,
    pub room_type: RoomType,
    pub devices: Vec<String>,
    pub zones: Vec<ZoneSpec>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoomType {
    LivingRoom,
    Bedroom,
    Kitchen,
    Office,
    Bathroom,
    Garage,
    Outdoor,
    Hallway,
    Custom(String),
}

impl fmt::Display for RoomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LivingRoom => write!(f, "LivingRoom"),
            Self::Bedroom => write!(f, "Bedroom"),
            Self::Kitchen => write!(f, "Kitchen"),
            Self::Office => write!(f, "Office"),
            Self::Bathroom => write!(f, "Bathroom"),
            Self::Garage => write!(f, "Garage"),
            Self::Outdoor => write!(f, "Outdoor"),
            Self::Hallway => write!(f, "Hallway"),
            Self::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZoneSpec {
    pub id: String,
    pub name: String,
    pub coordinates: Option<ZoneCoordinates>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneCoordinates {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct SceneConfig {
    pub id: String,
    pub name: String,
    pub devices: HashMap<String, SceneDeviceState>,
    pub triggers: Vec<SceneTrigger>,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub struct SceneDeviceState {
    pub device_id: String,
    pub state: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum SceneTrigger {
    TimeSchedule(String),
    DeviceEvent(String, String),
    VoiceCommand(String),
    Manual,
}

#[derive(Debug, Clone)]
pub struct ProjectSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    pub room_id: Option<String>,
    pub devices: Vec<String>,
    pub scenes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    pub id: String,
    pub name: String,
    pub lighting_scene: Option<String>,
    pub climate_settings: ClimateSettings,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClimateSettings {
    pub temperature_celsius: f64,
    pub humidity_percent: f64,
    pub fan_speed: String,
}

#[async_trait]
pub trait HomeManager: Send + Sync {
    async fn init(&self, config: &HomeConfig) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
    fn config(&self) -> &HomeConfig;

    async fn add_room(&self, room: RoomSpec) -> Result<()>;
    async fn remove_room(&self, room_id: &str) -> Result<()>;
    async fn get_room(&self, room_id: &str) -> Result<RoomSpec>;
    async fn list_rooms(&self) -> Result<Vec<RoomSpec>>;

    async fn register_device(
        &self,
        room_id: &str,
        device_id: &str,
        device_type: &str,
    ) -> Result<()>;
    async fn unregister_device(&self, room_id: &str, device_id: &str) -> Result<()>;
    async fn list_devices(&self, room_id: &str) -> Result<Vec<String>>;
    async fn all_devices(&self) -> Result<Vec<(String, String)>>;

    async fn activate_scene(&self, scene_id: &str) -> Result<()>;
    async fn deactivate_scene(&self, scene_id: &str) -> Result<()>;
    async fn list_scenes(&self) -> Result<Vec<SceneConfig>>;
    async fn get_scene(&self, scene_id: &str) -> Result<SceneConfig>;

    async fn create_project(&self, project: ProjectSpec) -> Result<()>;
    async fn delete_project(&self, project_id: &str) -> Result<()>;
    async fn get_project(&self, project_id: &str) -> Result<ProjectSpec>;
    async fn list_projects(&self) -> Result<Vec<ProjectSpec>>;

    async fn set_environment(&self, environment_id: &str) -> Result<()>;
    async fn get_environment(&self) -> Result<EnvironmentConfig>;
    async fn list_environments(&self) -> Result<Vec<EnvironmentConfig>>;

    async fn on_event(&self, handler: Box<dyn Fn(HomeEvent) + Send + Sync>) -> Result<()>;
}

#[async_trait]
pub trait DeviceRegistry: Send + Sync {
    async fn register(
        &self,
        device_id: &str,
        device_type: &str,
        room_id: Option<&str>,
    ) -> Result<()>;
    async fn unregister(&self, device_id: &str) -> Result<()>;
    async fn is_registered(&self, device_id: &str) -> bool;
    async fn device_room(&self, device_id: &str) -> Option<String>;
    async fn devices_in_room(&self, room_id: &str) -> Result<Vec<String>>;
    async fn all_devices(&self) -> Result<Vec<(String, String, Option<String>)>>;
    async fn device_count(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_spec_creation() {
        let room = RoomSpec {
            id: "r1".into(),
            name: "Living Room".into(),
            room_type: RoomType::LivingRoom,
            devices: vec!["light1".into(), "tv1".into()],
            zones: vec![],
            metadata: HashMap::new(),
        };
        assert_eq!(room.id, "r1");
        assert_eq!(room.devices.len(), 2);
    }

    #[test]
    fn test_scene_config_creation() {
        let mut devices = HashMap::new();
        devices.insert(
            "light1".into(),
            SceneDeviceState {
                device_id: "light1".into(),
                state: [("power".into(), "on".into())].into(),
            },
        );
        let scene = SceneConfig {
            id: "scene-1".into(),
            name: "Evening".into(),
            devices,
            triggers: vec![SceneTrigger::Manual],
            priority: 1,
        };
        assert_eq!(scene.id, "scene-1");
        assert_eq!(scene.triggers.len(), 1);
    }

    #[test]
    fn test_project_spec_creation() {
        let project = ProjectSpec {
            id: "proj-1".into(),
            name: "Build".into(),
            description: "Build project".into(),
            room_id: Some("office".into()),
            devices: vec!["dev1".into()],
            scenes: vec!["scene1".into()],
        };
        assert_eq!(project.name, "Build");
        assert!(project.room_id.is_some());
    }

    #[test]
    fn test_climate_settings() {
        let settings = ClimateSettings {
            temperature_celsius: 22.5,
            humidity_percent: 45.0,
            fan_speed: "auto".into(),
        };
        assert_eq!(settings.temperature_celsius, 22.5);
        assert_eq!(settings.humidity_percent, 45.0);
    }

    #[test]
    fn test_zone_coordinates() {
        let coords = ZoneCoordinates {
            x: 0.0,
            y: 0.0,
            z: Some(2.5),
            width: 5.0,
            height: 4.0,
        };
        assert_eq!(coords.z, Some(2.5));
        assert_eq!(coords.width, 5.0);
    }

    #[test]
    fn test_room_type_display() {
        assert_eq!(RoomType::LivingRoom.to_string(), "LivingRoom");
        assert_eq!(RoomType::Bedroom.to_string(), "Bedroom");
        assert_eq!(RoomType::Kitchen.to_string(), "Kitchen");
        assert_eq!(RoomType::Office.to_string(), "Office");
        assert_eq!(RoomType::Bathroom.to_string(), "Bathroom");
        assert_eq!(RoomType::Garage.to_string(), "Garage");
        assert_eq!(RoomType::Outdoor.to_string(), "Outdoor");
        assert_eq!(RoomType::Hallway.to_string(), "Hallway");
        assert_eq!(
            RoomType::Custom("Studio".into()).to_string(),
            "Custom(Studio)"
        );
    }

    #[test]
    fn test_room_type_partial_eq() {
        assert_eq!(RoomType::LivingRoom, RoomType::LivingRoom);
        assert_ne!(RoomType::LivingRoom, RoomType::Bedroom);
    }

    #[test]
    fn test_trait_definitions_compile() {
        // Verify that the trait bounds are satisfied by checking they are object-safe
        fn _assert_object_safe_home_manager(_: Box<dyn HomeManager>) {}
        fn _assert_object_safe_device_registry(_: Box<dyn DeviceRegistry>) {}
    }
}
