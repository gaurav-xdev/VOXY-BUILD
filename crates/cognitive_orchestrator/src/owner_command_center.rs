//! Owner Command Center — admin panel data providers.
//!
//! Provides live data for monitoring and controlling the system:
//! - Live task status
//! - Running agents
//! - Memory stats
//! - Goals and progress
//! - Project status
//! - CPU/GPU metrics
//! - Model status
//! - System logs
//! - Warning/alert feed
//! - Risk score

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Core Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoalId(pub String);

// ============================================================================
// Live Task Status
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTask {
    pub id: TaskId,
    pub name: String,
    pub status: TaskStatus,
    pub progress: f32,
    pub started_at: Option<DateTime<Utc>>,
    pub elapsed_ms: Option<u64>,
    pub assigned_agent: Option<AgentId>,
    pub subtasks_total: u32,
    pub subtasks_completed: u32,
    pub current_step: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSnapshot {
    pub tasks: Vec<LiveTask>,
    pub total_running: u32,
    pub total_queued: u32,
    pub total_completed: u32,
    pub total_failed: u32,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Running Agents
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Working,
    Waiting,
    Error(String),
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningAgent {
    pub id: AgentId,
    pub name: String,
    pub role: String,
    pub state: AgentState,
    pub current_task: Option<TaskId>,
    pub tasks_completed: u32,
    pub uptime_ms: u64,
    pub last_heartbeat: DateTime<Utc>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSnapshot {
    pub agents: Vec<RunningAgent>,
    pub total_agents: u32,
    pub active_agents: u32,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Memory Stats
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatsDashboard {
    pub total_items: u64,
    pub working_memory: u64,
    pub short_term: u64,
    pub long_term: u64,
    pub episodic: u64,
    pub semantic: u64,
    pub relationships: u64,
    pub storage_bytes: u64,
    pub cache_hit_rate: f32,
    pub compression_ratio: f32,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Goals and Progress
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalStatus {
    Draft,
    Active,
    Blocked,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProgress {
    pub id: GoalId,
    pub name: String,
    pub status: GoalStatus,
    pub progress: f32,
    pub sub_goals_total: u32,
    pub sub_goals_completed: u32,
    pub dependencies_met: bool,
    pub deadline: Option<DateTime<Utc>>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSnapshot {
    pub goals: Vec<GoalProgress>,
    pub total_goals: u32,
    pub active_goals: u32,
    pub completed_goals: u32,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Project Status
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectPhase {
    Planning,
    Development,
    Testing,
    Deployment,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatus {
    pub name: String,
    pub phase: ProjectPhase,
    pub progress: f32,
    pub files_changed: u32,
    pub tests_passing: u32,
    pub tests_failing: u32,
    pub open_issues: u32,
    pub last_commit: Option<DateTime<Utc>>,
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub projects: Vec<ProjectStatus>,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// System Metrics
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub cores_active: u32,
    pub total_cores: u32,
    pub frequency_mhz: u32,
    pub temperature_celsius: Option<f32>,
    pub load_average_1m: f32,
    pub load_average_5m: f32,
    pub load_average_15m: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub name: String,
    pub usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub temperature_celsius: Option<f32>,
    pub power_watts: Option<f32>,
    pub driver_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub gpu: Option<GpuMetrics>,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub disk_used_gb: f32,
    pub disk_total_gb: f32,
    pub network_up_mbps: f32,
    pub network_down_mbps: f32,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Model Status
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    Loaded,
    Loading,
    Unloaded,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub status: ModelStatus,
    pub backend: String,
    pub parameters: u64,
    pub memory_mb: u64,
    pub latency_ms: f32,
    pub throughput_tokens_per_sec: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSnapshot {
    pub models: Vec<ModelInfo>,
    pub active_model: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// System Logs
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSnapshot {
    pub entries: Vec<LogEntry>,
    pub total_entries: u64,
    pub error_count: u64,
    pub warn_count: u64,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Warning/Alert Feed
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub source: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
    pub auto_dismiss: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSnapshot {
    pub alerts: Vec<Alert>,
    pub unacknowledged_count: u32,
    pub critical_count: u32,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Risk Score
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskBreakdown {
    pub security: f32,
    pub reliability: f32,
    pub performance: f32,
    pub complexity: f32,
    pub dependency_health: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub overall: f32,
    pub breakdown: RiskBreakdown,
    pub factors: Vec<String>,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Owner Command Center (aggregator)
// ============================================================================

/// Aggregates all dashboard data for the owner/admin panel.
pub struct OwnerCommandCenter {
    max_log_entries: usize,
    max_alerts: usize,
    log_buffer: Vec<LogEntry>,
    alerts: Vec<Alert>,
}

impl OwnerCommandCenter {
    pub fn new(max_log_entries: usize, max_alerts: usize) -> Self {
        Self {
            max_log_entries,
            max_alerts,
            log_buffer: Vec::new(),
            alerts: Vec::new(),
        }
    }

    pub fn default_config() -> Self {
        Self::new(1000, 100)
    }

    /// Add a log entry.
    pub fn add_log(&mut self, entry: LogEntry) {
        self.log_buffer.push(entry);
        if self.log_buffer.len() > self.max_log_entries {
            self.log_buffer
                .drain(0..self.log_buffer.len() - self.max_log_entries);
        }
    }

    /// Add an alert.
    pub fn add_alert(&mut self, alert: Alert) {
        if self.alerts.len() >= self.max_alerts {
            self.alerts.remove(0);
        }
        self.alerts.push(alert);
    }

    /// Acknowledge an alert.
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> bool {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            true
        } else {
            false
        }
    }

    /// Get current log snapshot.
    pub fn log_snapshot(&self) -> LogSnapshot {
        let error_count = self
            .log_buffer
            .iter()
            .filter(|e| matches!(e.level, LogLevel::Error))
            .count() as u64;
        let warn_count = self
            .log_buffer
            .iter()
            .filter(|e| matches!(e.level, LogLevel::Warn))
            .count() as u64;

        LogSnapshot {
            entries: self.log_buffer.clone(),
            total_entries: self.log_buffer.len() as u64,
            error_count,
            warn_count,
            timestamp: Utc::now(),
        }
    }

    /// Get current alert snapshot.
    pub fn alert_snapshot(&self) -> AlertSnapshot {
        let unacknowledged = self.alerts.iter().filter(|a| !a.acknowledged).count() as u32;
        let critical = self
            .alerts
            .iter()
            .filter(|a| matches!(a.severity, AlertSeverity::Critical))
            .count() as u32;

        AlertSnapshot {
            alerts: self.alerts.clone(),
            unacknowledged_count: unacknowledged,
            critical_count: critical,
            timestamp: Utc::now(),
        }
    }

    /// Filter logs by level.
    pub fn logs_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.log_buffer
            .iter()
            .filter(|e| e.level == level)
            .collect()
    }

    /// Filter logs by source.
    pub fn logs_by_source(&self, source: &str) -> Vec<&LogEntry> {
        self.log_buffer
            .iter()
            .filter(|e| e.source == source)
            .collect()
    }

    /// Get critical alerts only.
    pub fn critical_alerts(&self) -> Vec<&Alert> {
        self.alerts
            .iter()
            .filter(|a| matches!(a.severity, AlertSeverity::Critical))
            .collect()
    }

    /// Clear acknowledged alerts.
    pub fn clear_acknowledged(&mut self) {
        self.alerts.retain(|a| !a.acknowledged);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_log(level: LogLevel, source: &str, msg: &str) -> LogEntry {
        LogEntry {
            timestamp: Utc::now(),
            level,
            source: source.to_string(),
            message: msg.to_string(),
            metadata: HashMap::new(),
        }
    }

    fn sample_alert(severity: AlertSeverity, msg: &str) -> Alert {
        Alert {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            source: "test".to_string(),
            message: msg.to_string(),
            timestamp: Utc::now(),
            acknowledged: false,
            auto_dismiss: false,
        }
    }

    #[test]
    fn command_center_creation() {
        let cc = OwnerCommandCenter::default_config();
        assert_eq!(cc.log_buffer.len(), 0);
        assert_eq!(cc.alerts.len(), 0);
    }

    #[test]
    fn add_and_retrieve_logs() {
        let mut cc = OwnerCommandCenter::default_config();
        cc.add_log(sample_log(LogLevel::Info, "system", "Started"));
        cc.add_log(sample_log(LogLevel::Error, "auth", "Failed login"));
        cc.add_log(sample_log(LogLevel::Warn, "memory", "High usage"));

        let snap = cc.log_snapshot();
        assert_eq!(snap.total_entries, 3);
        assert_eq!(snap.error_count, 1);
        assert_eq!(snap.warn_count, 1);
    }

    #[test]
    fn log_buffer_respects_limit() {
        let mut cc = OwnerCommandCenter::new(5, 100);
        for i in 0..10 {
            cc.add_log(sample_log(LogLevel::Info, "test", &format!("msg {}", i)));
        }
        assert_eq!(cc.log_buffer.len(), 5);
    }

    #[test]
    fn add_and_acknowledge_alerts() {
        let mut cc = OwnerCommandCenter::default_config();
        let alert = sample_alert(AlertSeverity::Critical, "CPU over 90%");
        cc.add_alert(alert.clone());

        let snap = cc.alert_snapshot();
        assert_eq!(snap.critical_count, 1);
        assert_eq!(snap.unacknowledged_count, 1);

        cc.acknowledge_alert(&alert.id);
        let snap = cc.alert_snapshot();
        assert_eq!(snap.unacknowledged_count, 0);
    }

    #[test]
    fn acknowledge_nonexistent_alert() {
        let mut cc = OwnerCommandCenter::default_config();
        assert!(!cc.acknowledge_alert("nonexistent"));
    }

    #[test]
    fn clear_acknowledged_alerts() {
        let mut cc = OwnerCommandCenter::default_config();
        let a1 = sample_alert(AlertSeverity::Warning, "w1");
        let a2 = sample_alert(AlertSeverity::Critical, "c1");
        cc.add_alert(a1.clone());
        cc.add_alert(a2.clone());
        cc.acknowledge_alert(&a1.id);
        cc.clear_acknowledged();
        assert_eq!(cc.alerts.len(), 1);
        assert_eq!(cc.alerts[0].id, a2.id);
    }

    #[test]
    fn logs_by_level() {
        let mut cc = OwnerCommandCenter::default_config();
        cc.add_log(sample_log(LogLevel::Info, "a", "i1"));
        cc.add_log(sample_log(LogLevel::Error, "a", "e1"));
        cc.add_log(sample_log(LogLevel::Info, "a", "i2"));

        let errors = cc.logs_by_level(LogLevel::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "e1");
    }

    #[test]
    fn logs_by_source() {
        let mut cc = OwnerCommandCenter::default_config();
        cc.add_log(sample_log(LogLevel::Info, "auth", "login"));
        cc.add_log(sample_log(LogLevel::Info, "db", "query"));
        cc.add_log(sample_log(LogLevel::Info, "auth", "logout"));

        let auth_logs = cc.logs_by_source("auth");
        assert_eq!(auth_logs.len(), 2);
    }

    #[test]
    fn critical_alerts() {
        let mut cc = OwnerCommandCenter::default_config();
        cc.add_alert(sample_alert(AlertSeverity::Info, "info"));
        cc.add_alert(sample_alert(AlertSeverity::Critical, "crit1"));
        cc.add_alert(sample_alert(AlertSeverity::Critical, "crit2"));

        let crits = cc.critical_alerts();
        assert_eq!(crits.len(), 2);
    }

    #[test]
    fn alert_severity_variants() {
        let _i = AlertSeverity::Info;
        let _w = AlertSeverity::Warning;
        let _c = AlertSeverity::Critical;
    }

    #[test]
    fn task_status_variants() {
        let _q = TaskStatus::Queued;
        let _r = TaskStatus::Running;
        let _p = TaskStatus::Paused;
        let _c = TaskStatus::Completed;
        let _f = TaskStatus::Failed("err".to_string());
        let _x = TaskStatus::Cancelled;
    }

    #[test]
    fn agent_state_variants() {
        let _i = AgentState::Idle;
        let _w = AgentState::Working;
        let _w = AgentState::Waiting;
        let _e = AgentState::Error("err".to_string());
        let _o = AgentState::Offline;
    }

    #[test]
    fn log_level_equality() {
        assert_eq!(LogLevel::Error, LogLevel::Error);
        assert_ne!(LogLevel::Info, LogLevel::Warn);
    }

    #[test]
    fn alert_buffer_respects_limit() {
        let mut cc = OwnerCommandCenter::new(1000, 3);
        for _ in 0..5 {
            cc.add_alert(sample_alert(AlertSeverity::Warning, "alert"));
        }
        assert_eq!(cc.alerts.len(), 3);
    }
}
