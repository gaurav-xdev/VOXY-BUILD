use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticsReport {
    pub os_name: String,
    pub os_version: String,
    pub hostname: String,
    pub cpu_count: usize,
    pub cpu_brand: String,
    pub memory_total_bytes: u64,
    pub process_uptime_seconds: u64,
    pub kernel_version: String,
}

pub struct SystemDiagnostics;

impl SystemDiagnostics {
    pub fn os_info() -> (String, String) {
        (
            System::name().unwrap_or_else(|| "unknown".into()),
            System::os_version().unwrap_or_else(|| "unknown".into()),
        )
    }

    pub fn hostname() -> String {
        System::host_name().unwrap_or_else(|| "unknown".into())
    }

    pub fn cpu_info() -> (usize, String) {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        let count = sys.cpus().len();
        let brand = if count > 0 {
            let first = &sys.cpus()[0];
            first.brand().to_string()
        } else {
            "unknown".into()
        };
        (count, brand)
    }

    pub fn memory_total() -> u64 {
        let mut sys = System::new();
        sys.refresh_memory();
        sys.total_memory()
    }

    pub fn uptime() -> u64 {
        System::uptime()
    }

    pub fn kernel_version() -> String {
        System::kernel_version().unwrap_or_else(|| "unknown".into())
    }

    pub fn collect_all() -> DiagnosticsReport {
        let (os_name, os_version) = Self::os_info();
        let (cpu_count, cpu_brand) = Self::cpu_info();

        DiagnosticsReport {
            os_name,
            os_version,
            hostname: Self::hostname(),
            cpu_count,
            cpu_brand,
            memory_total_bytes: Self::memory_total(),
            process_uptime_seconds: Self::uptime(),
            kernel_version: Self::kernel_version(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_collect_all() {
        let report = SystemDiagnostics::collect_all();
        assert!(!report.os_name.is_empty());
        assert!(!report.hostname.is_empty());
        assert!(report.cpu_count > 0);
        assert!(!report.cpu_brand.is_empty());
        assert!(report.memory_total_bytes > 0);
        assert!(!report.kernel_version.is_empty());
    }

    #[test]
    fn diagnostics_os_info() {
        let (name, version) = SystemDiagnostics::os_info();
        assert!(!name.is_empty());
        assert!(!version.is_empty());
    }

    #[test]
    fn diagnostics_hostname() {
        let hostname = SystemDiagnostics::hostname();
        assert!(!hostname.is_empty());
    }

    #[test]
    fn diagnostics_cpu_info() {
        let (count, brand) = SystemDiagnostics::cpu_info();
        assert!(count > 0);
        assert!(!brand.is_empty());
    }

    #[test]
    fn diagnostics_memory_total() {
        let total = SystemDiagnostics::memory_total();
        assert!(total > 0);
    }

    #[test]
    fn diagnostics_uptime() {
        let uptime = SystemDiagnostics::uptime();
        assert!(uptime > 0);
    }

    #[test]
    fn diagnostics_serde() {
        let report = SystemDiagnostics::collect_all();
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("os_name"));
        assert!(json.contains("hostname"));
    }
}
