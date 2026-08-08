use crate::snapshot::RuntimeSnapshot;
use serde::{Deserialize, Serialize};

/// Dashboard data exposed via HTTP/WebSocket for monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub snapshot: RuntimeSnapshot,
    pub html: String,
}

impl DashboardData {
    pub fn from_snapshot(snapshot: RuntimeSnapshot) -> Self {
        let html = generate_html(&snapshot);
        Self { snapshot, html }
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn generate_html(snap: &RuntimeSnapshot) -> String {
    let health_color = match &snap.overall_health {
        voxy_shared::HealthStatus::Healthy => "#22c55e",
        voxy_shared::HealthStatus::Degraded(_) => "#eab308",
        voxy_shared::HealthStatus::Unhealthy(_) => "#ef4444",
    };

    let subsystem_rows: String = snap
        .subsystems
        .values()
        .map(|s| {
            let (status_text, status_color) = match &s.health {
                voxy_shared::HealthStatus::Healthy => ("Healthy", "#22c55e"),
                voxy_shared::HealthStatus::Degraded(_msg) => ("Degraded", "#eab308"),
                voxy_shared::HealthStatus::Unhealthy(_msg) => ("Unhealthy", "#ef4444"),
            };
            format!(
                r#"<tr>
                    <td>{}</td>
                    <td style="color:{}">{}</td>
                    <td>{}</td>
                    <td>{:.1}ms</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                escape_html(&s.name),
                status_color,
                status_text,
                s.restart_count,
                s.latency_ms.unwrap_or(0.0),
                s.uptime_secs,
                escape_html(s.last_error.as_deref().unwrap_or("-")),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mood = escape_html(snap.current_mood.as_deref().unwrap_or("-"));
    let presence = escape_html(snap.presence_state.as_deref().unwrap_or("-"));
    let activity = escape_html(snap.desktop_activity.as_deref().unwrap_or("-"));
    let timestamp = escape_html(&snap.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string());

    format!(
        r#"<!DOCTYPE html>
<html><head>
<title>VOXY Runtime Dashboard</title>
<meta http-equiv="refresh" content="5">
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 20px; background: #1a1a2e; color: #e0e0e0; }}
h1 {{ color: #fff; }}
.card {{ background: #16213e; border-radius: 8px; padding: 16px; margin: 8px 0; display: inline-block; min-width: 150px; }}
.card-label {{ font-size: 12px; color: #888; }}
.card-value {{ font-size: 24px; font-weight: bold; }}
table {{ border-collapse: collapse; width: 100%; margin-top: 16px; }}
th, td {{ padding: 8px 12px; text-align: left; border-bottom: 1px solid #333; }}
th {{ background: #0f3460; }}
.health-bar {{ width: 100%; height: 20px; background: #333; border-radius: 10px; overflow: hidden; }}
.health-fill {{ height: 100%; background: {}; transition: width 0.3s; }}
</style>
</head><body>
<h1>VOXY Runtime Dashboard</h1>
<p>Last updated: {}</p>

<div class="health-bar"><div class="health-fill" style="width:{:.0}%"></div></div>
<p style="text-align:center; color:{}; font-weight:bold;">{:.0}% Healthy</p>

<div>
  <div class="card"><div class="card-label">CPU</div><div class="card-value">{:.1}%</div></div>
  <div class="card"><div class="card-label">RAM</div><div class="card-value">{}MB / {}MB</div></div>
  <div class="card"><div class="card-label">Threads</div><div class="card-value">{}</div></div>
  <div class="card"><div class="card-label">Uptime</div><div class="card-value">{}s</div></div>
  <div class="card"><div class="card-label">Restarts</div><div class="card-value">{}</div></div>
  <div class="card"><div class="card-label">Mood</div><div class="card-value">{}</div></div>
  <div class="card"><div class="card-label">Presence</div><div class="card-value">{}</div></div>
  <div class="card"><div class="card-label">Activity</div><div class="card-value">{}</div></div>
</div>

<h2>Subsystems</h2>
<table>
<tr><th>Name</th><th>Status</th><th>Restarts</th><th>Latency</th><th>Uptime</th><th>Last Error</th></tr>
{}
</table>

</body></html>"#,
        health_color,
        timestamp,
        snap.health_pct,
        health_color,
        snap.health_pct,
        snap.cpu_usage_pct,
        snap.ram_usage_mb,
        snap.ram_total_mb,
        snap.thread_count,
        snap.uptime_secs,
        snap.total_restarts,
        mood,
        presence,
        activity,
        subsystem_rows,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{RuntimeSnapshot, SubsystemStatus};
    use voxy_shared::HealthStatus;

    #[test]
    fn dashboard_from_snapshot() {
        let mut snap = RuntimeSnapshot::new();
        snap.cpu_usage_pct = 45.2;
        snap.ram_usage_mb = 512;
        snap.ram_total_mb = 8192;
        snap.subsystems.insert(
            "audio".into(),
            SubsystemStatus {
                name: "audio".into(),
                health: HealthStatus::Healthy,
                last_heartbeat: chrono::Utc::now(),
                restart_count: 0,
                last_error: None,
                latency_ms: Some(1.5),
                uptime_secs: 300,
            },
        );

        let dashboard = DashboardData::from_snapshot(snap);
        assert!(dashboard.html.contains("VOXY Runtime Dashboard"));
        assert!(dashboard.html.contains("audio"));
        assert!(dashboard.html.contains("45.2%"));
    }

    #[test]
    fn dashboard_html_not_empty() {
        let snap = RuntimeSnapshot::new();
        let dashboard = DashboardData::from_snapshot(snap);
        assert!(!dashboard.html.is_empty());
    }
}
