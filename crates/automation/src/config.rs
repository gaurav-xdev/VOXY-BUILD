#[derive(Debug, Clone)]
pub struct AutomationConfig {
    pub default_backend: BackendType,
    pub retry_policy: RetryPolicy,
    pub timeouts: TimeoutConfig,
    pub verification: VerificationConfig,
    pub dpi_aware: bool,
    pub multi_monitor: bool,
    pub openclaw: OpenClawConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendType {
    WindowsUia,
    OpenClaw,
    Hybrid,
    Verification,
    Recovery,
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_factor: f64,
}

#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub element_wait_ms: u64,
    pub window_wait_ms: u64,
    pub action_ms: u64,
    pub verification_ms: u64,
}

#[derive(Debug, Clone)]
pub struct VerificationConfig {
    pub screenshot_before: bool,
    pub screenshot_after: bool,
    pub verify_element_state: bool,
    pub strict_bounds_check: bool,
}

#[derive(Debug, Clone)]
pub struct OpenClawConfig {
    pub endpoint: String,
    /// SECURITY: API key stored as Zeroizing to prevent memory exposure on drop.
    pub api_key: Option<zeroize::Zeroizing<String>>,
    pub timeout_seconds: u64,
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            default_backend: BackendType::Hybrid,
            retry_policy: RetryPolicy {
                max_retries: 3,
                base_delay_ms: 100,
                max_delay_ms: 5000,
                backoff_factor: 2.0,
            },
            timeouts: TimeoutConfig {
                element_wait_ms: 5000,
                window_wait_ms: 10000,
                action_ms: 30000,
                verification_ms: 15000,
            },
            verification: VerificationConfig {
                screenshot_before: true,
                screenshot_after: true,
                verify_element_state: true,
                strict_bounds_check: false,
            },
            dpi_aware: true,
            multi_monitor: true,
            openclaw: OpenClawConfig {
                endpoint: "http://127.0.0.1:9876".into(),
                api_key: None,
                timeout_seconds: 30,
            },
        }
    }
}
