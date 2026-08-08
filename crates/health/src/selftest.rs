use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
pub struct SelfTestResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: f64,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl SelfTestResult {
    pub fn new(name: impl Into<String>, passed: bool) -> Self {
        Self {
            name: name.into(),
            passed,
            duration_ms: 0.0,
            details: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_duration(mut self, duration_ms: f64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

pub type SelfTestFn =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = SelfTestResult> + Send + Sync>> + Send + Sync>;

pub struct SelfTestRunner {
    tests: RwLock<HashMap<String, SelfTestFn>>,
}

impl SelfTestRunner {
    pub fn new() -> Self {
        Self {
            tests: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, name: &str, test: SelfTestFn) {
        let mut tests = self.tests.write().await;
        tests.insert(name.to_string(), test);
    }

    pub async fn run_all(&self) -> Vec<SelfTestResult> {
        let tests = self.tests.read().await;
        let mut results = Vec::new();

        for test in tests.values() {
            let start = Instant::now();
            let mut result = test().await;
            result.duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            results.push(result);
        }

        results
    }

    pub async fn run_one(&self, name: &str) -> Option<SelfTestResult> {
        let tests = self.tests.read().await;
        let test = tests.get(name)?;
        let start = Instant::now();
        let mut result = test().await;
        result.duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        Some(result)
    }

    pub async fn results_summary(&self) -> SelfTestSummary {
        let results = self.run_all().await;
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        SelfTestSummary {
            total,
            passed,
            failed,
            results,
        }
    }

    pub async fn test_count(&self) -> usize {
        self.tests.read().await.len()
    }
}

impl Default for SelfTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SelfTestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<SelfTestResult>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn register_and_run_all() {
        let runner = SelfTestRunner::new();
        runner
            .register(
                "ping",
                Box::new(|| Box::pin(async { SelfTestResult::new("ping", true) })),
            )
            .await;
        runner
            .register(
                "pong",
                Box::new(|| Box::pin(async { SelfTestResult::new("pong", false) })),
            )
            .await;

        let results = runner.run_all().await;
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn run_one() {
        let runner = SelfTestRunner::new();
        runner
            .register(
                "check1",
                Box::new(|| {
                    Box::pin(async { SelfTestResult::new("check1", true).with_details("ok") })
                }),
            )
            .await;

        let result = runner.run_one("check1").await.unwrap();
        assert!(result.passed);
        assert_eq!(result.details.as_deref(), Some("ok"));

        let missing = runner.run_one("nonexistent").await;
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn results_summary() {
        let runner = SelfTestRunner::new();
        runner
            .register(
                "pass",
                Box::new(|| Box::pin(async { SelfTestResult::new("pass", true) })),
            )
            .await;
        runner
            .register(
                "fail",
                Box::new(|| Box::pin(async { SelfTestResult::new("fail", false) })),
            )
            .await;

        let summary = runner.results_summary().await;
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
    }

    #[tokio::test]
    async fn empty_runner() {
        let runner = SelfTestRunner::new();
        assert_eq!(runner.test_count().await, 0);
        let results = runner.run_all().await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_duration_is_measured() {
        let runner = SelfTestRunner::new();
        runner
            .register(
                "slow",
                Box::new(|| {
                    Box::pin(async {
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        SelfTestResult::new("slow", true)
                    })
                }),
            )
            .await;

        let result = runner.run_one("slow").await.unwrap();
        assert!(result.passed);
        assert!(result.duration_ms >= 10.0);
    }

    #[tokio::test]
    async fn concurrent_registration() {
        let runner = Arc::new(SelfTestRunner::new());
        let mut handles = Vec::new();

        for i in 0..10 {
            let runner = runner.clone();
            handles.push(tokio::spawn(async move {
                runner
                    .register(
                        &format!("test_{}", i),
                        Box::new(|| Box::pin(async { SelfTestResult::new("test", true) })),
                    )
                    .await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(runner.test_count().await, 10);
    }
}
