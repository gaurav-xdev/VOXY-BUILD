use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use voxy_shared::HealthStatus;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    Memory,
    Cpu,
    Service,
    EventBus,
    Ipc,
    Database,
    Custom(String),
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory => write!(f, "memory"),
            Self::Cpu => write!(f, "cpu"),
            Self::Service => write!(f, "service"),
            Self::EventBus => write!(f, "event_bus"),
            Self::Ipc => write!(f, "ipc"),
            Self::Database => write!(f, "database"),
            Self::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub name: String,
    pub status: HealthStatus,
    pub timestamp: DateTime<Utc>,
    pub latency_ms: Option<f64>,
    pub details: Option<String>,
    pub component_type: ComponentType,
}

impl HealthReport {
    pub fn new(name: impl Into<String>, status: HealthStatus) -> Self {
        Self {
            name: name.into(),
            status,
            timestamp: Utc::now(),
            latency_ms: None,
            details: None,
            component_type: ComponentType::Service,
        }
    }

    pub fn with_latency(mut self, latency_ms: f64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_component_type(mut self, ct: ComponentType) -> Self {
        self.component_type = ct;
        self
    }

    pub fn is_healthy(&self) -> bool {
        self.status.is_healthy()
    }

    pub fn is_degraded(&self) -> bool {
        self.status.is_degraded()
    }

    pub fn is_unhealthy(&self) -> bool {
        self.status.is_unhealthy()
    }

    pub fn elapsed_seconds(&self) -> f64 {
        (Utc::now() - self.timestamp).num_seconds() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_creation() {
        let report = HealthReport::new("test", HealthStatus::Healthy);
        assert_eq!(report.name, "test");
        assert!(report.is_healthy());
        assert_eq!(report.component_type, ComponentType::Service);
    }

    #[test]
    fn report_with_latency() {
        let report = HealthReport::new("test", HealthStatus::Healthy).with_latency(1.5);
        assert_eq!(report.latency_ms, Some(1.5));
    }

    #[test]
    fn report_with_component_type() {
        let report = HealthReport::new("mem", HealthStatus::Healthy)
            .with_component_type(ComponentType::Memory);
        assert_eq!(report.component_type, ComponentType::Memory);
    }

    #[test]
    fn report_with_details() {
        let report = HealthReport::new("test", HealthStatus::Degraded("slow".into()))
            .with_details("response time > 2s");
        assert_eq!(report.details.as_deref(), Some("response time > 2s"));
        assert!(report.is_degraded());
    }

    #[test]
    fn report_unhealthy() {
        let report = HealthReport::new("svc", HealthStatus::Unhealthy("crash".into()));
        assert!(report.is_unhealthy());
        assert!(!report.is_healthy());
    }

    #[test]
    fn component_type_display() {
        assert_eq!(ComponentType::Memory.to_string(), "memory");
        assert_eq!(ComponentType::Cpu.to_string(), "cpu");
        assert_eq!(ComponentType::Service.to_string(), "service");
        assert_eq!(ComponentType::EventBus.to_string(), "event_bus");
        assert_eq!(ComponentType::Ipc.to_string(), "ipc");
        assert_eq!(ComponentType::Database.to_string(), "database");
        assert_eq!(
            ComponentType::Custom("my_comp".into()).to_string(),
            "custom:my_comp"
        );
    }

    #[test]
    fn report_serde_roundtrip() {
        let report = HealthReport::new("svc", HealthStatus::Healthy)
            .with_latency(0.5)
            .with_component_type(ComponentType::Service);
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: HealthReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "svc");
        assert!(deserialized.is_healthy());
        assert_eq!(deserialized.component_type, ComponentType::Service);
    }
}
