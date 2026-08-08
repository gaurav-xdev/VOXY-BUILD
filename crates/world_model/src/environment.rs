#[derive(Debug, Clone)]
pub struct UserEnvironment {
    pub location: Option<Location>,
    pub timezone: String,
    pub ambient: AmbientConditions,
}

impl Default for UserEnvironment {
    fn default() -> Self {
        Self {
            location: None,
            timezone: String::from("UTC"),
            ambient: AmbientConditions::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AmbientConditions {
    pub light_level_lux: Option<f64>,
    pub noise_level_db: Option<f64>,
    pub temperature_celsius: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_environment_default() {
        let env = UserEnvironment::default();
        assert!(env.location.is_none());
        assert_eq!(env.timezone, "UTC");
        assert!(env.ambient.light_level_lux.is_none());
        assert!(env.ambient.noise_level_db.is_none());
        assert!(env.ambient.temperature_celsius.is_none());
    }

    #[test]
    fn test_ambient_conditions_default() {
        let amb = AmbientConditions::default();
        assert!(amb.light_level_lux.is_none());
        assert!(amb.noise_level_db.is_none());
        assert!(amb.temperature_celsius.is_none());
    }

    #[test]
    fn test_location_creation() {
        let loc = Location {
            latitude: 37.7749,
            longitude: -122.4194,
            name: Some("San Francisco".to_string()),
        };
        assert_eq!(loc.latitude, 37.7749);
        assert_eq!(loc.longitude, -122.4194);
        assert_eq!(loc.name.unwrap(), "San Francisco");
    }
}
