use crate::error::Result;
use crate::types::{AdaptiveThreshold, CalibrationSample};
use async_trait::async_trait;

#[async_trait]
pub trait ConfidenceCalibration: Send + Sync {
    async fn record_sample(&self, sample: CalibrationSample) -> Result<()>;
    async fn calibrate(&self, estimator_id: &str) -> Result<AdaptiveThreshold>;
    async fn get_calibrated_threshold(&self, estimator_id: &str) -> Result<f64>;
    async fn get_recent_samples(
        &self,
        estimator_id: &str,
        count: usize,
    ) -> Result<Vec<CalibrationSample>>;
    async fn calibration_quality(&self, estimator_id: &str) -> Result<f64>;
    async fn reset_calibration(&self, estimator_id: &str) -> Result<()>;
}
