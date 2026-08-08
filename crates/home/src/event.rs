use std::fmt;

#[derive(Debug, Clone)]
pub enum HomeEvent {
    DeviceAdded {
        device_id: String,
        device_type: String,
        room_id: Option<String>,
    },
    DeviceRemoved {
        device_id: String,
        room_id: Option<String>,
    },
    DeviceStatusChanged {
        device_id: String,
        status: String,
    },
    RoomAdded {
        room_id: String,
        name: String,
    },
    RoomRemoved {
        room_id: String,
    },
    SceneActivated {
        scene_id: String,
        name: String,
    },
    SceneDeactivated {
        scene_id: String,
    },
    EnvironmentChanged {
        environment_id: String,
    },
    ProjectCreated {
        project_id: String,
    },
    ProjectDeleted {
        project_id: String,
    },
    UserPresenceChanged {
        user_id: String,
        present: bool,
    },
}

impl fmt::Display for HomeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceAdded {
                device_id,
                device_type,
                room_id,
            } => {
                write!(
                    f,
                    "Device added: {} ({}) in {:?}",
                    device_id, device_type, room_id
                )
            }
            Self::DeviceRemoved { device_id, room_id } => {
                write!(f, "Device removed: {} from {:?}", device_id, room_id)
            }
            Self::DeviceStatusChanged { device_id, status } => {
                write!(f, "Device status changed: {} -> {}", device_id, status)
            }
            Self::RoomAdded { room_id, name } => {
                write!(f, "Room added: {} ({})", room_id, name)
            }
            Self::RoomRemoved { room_id } => {
                write!(f, "Room removed: {}", room_id)
            }
            Self::SceneActivated { scene_id, name } => {
                write!(f, "Scene activated: {} ({})", scene_id, name)
            }
            Self::SceneDeactivated { scene_id } => {
                write!(f, "Scene deactivated: {}", scene_id)
            }
            Self::EnvironmentChanged { environment_id } => {
                write!(f, "Environment changed: {}", environment_id)
            }
            Self::ProjectCreated { project_id } => {
                write!(f, "Project created: {}", project_id)
            }
            Self::ProjectDeleted { project_id } => {
                write!(f, "Project deleted: {}", project_id)
            }
            Self::UserPresenceChanged { user_id, present } => {
                write!(f, "User presence changed: {} present={}", user_id, present)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_event_display() {
        let event = HomeEvent::RoomAdded {
            room_id: "living_room".into(),
            name: "Living Room".into(),
        };
        let s = event.to_string();
        assert!(s.contains("Room added"));
        assert!(s.contains("living_room"));

        let event = HomeEvent::DeviceAdded {
            device_id: "light1".into(),
            device_type: "Light".into(),
            room_id: Some("living_room".into()),
        };
        let s = event.to_string();
        assert!(s.contains("Device added"));

        let event = HomeEvent::UserPresenceChanged {
            user_id: "user1".into(),
            present: true,
        };
        let s = event.to_string();
        assert!(s.contains("present=true"));
    }
}
