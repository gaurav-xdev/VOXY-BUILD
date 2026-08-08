use crate::config::{KnowledgeValidationConfig, RiskLevel};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: Uuid,
    pub content: String,
    pub source: String,
    pub source_type: SourceType,
    pub trust_score: f32,
    pub risk_level: RiskLevel,
    pub cross_references: Vec<String>,
    pub validation_status: ValidationStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    Internet,
    UserProvided,
    SystemGenerated,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    Pending,
    Validated,
    Rejected,
    Quarantined,
    PartialTrust,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub item_id: Uuid,
    pub trust_score: f32,
    pub risk_level: RiskLevel,
    pub status: ValidationStatus,
    pub cross_references_found: usize,
    pub hallucination_score: f32,
    pub flags: Vec<String>,
}

pub struct KnowledgeValidator {
    config: KnowledgeValidationConfig,
    validated_items: Vec<KnowledgeItem>,
    quarantined_items: Vec<KnowledgeItem>,
    trust_history: Vec<(Uuid, f32)>,
}

const MAX_TRUST_HISTORY: usize = 500;
const MAX_VALIDATED_ITEMS: usize = 1000;
const MAX_QUARANTINED_ITEMS: usize = 200;

impl KnowledgeValidator {
    pub fn new(config: KnowledgeValidationConfig) -> Self {
        Self {
            config,
            validated_items: Vec::new(),
            quarantined_items: Vec::new(),
            trust_history: Vec::new(),
        }
    }

    pub fn validate(&mut self, mut item: KnowledgeItem) -> Result<ValidationResult> {
        let mut flags = Vec::new();
        let mut trust = item.trust_score;

        if item.source_type == SourceType::Internet {
            trust *= 0.8;
            flags.push("Internet source - reduced trust".to_string());
        }

        let cross_ref_count = item.cross_references.len();
        if cross_ref_count < self.config.required_cross_references {
            trust *= 0.7;
            flags.push(format!(
                "Insufficient cross-references: {}/{}",
                cross_ref_count, self.config.required_cross_references
            ));
        }

        let hallucination_score = self.detect_hallucination(&item.content);
        if hallucination_score > self.config.hallucination_threshold {
            trust *= 0.3;
            flags.push("Possible hallucination detected".to_string());
        }

        let risk = self.assess_risk(&item.content, &item.source_type);

        let risk_level_num = match risk {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
            RiskLevel::Critical => 3,
        };
        let max_risk_num = match self.config.max_risk_level {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
            RiskLevel::Critical => 3,
        };

        let status = if trust < self.config.min_trust_score {
            ValidationStatus::Rejected
        } else if risk_level_num > max_risk_num {
            ValidationStatus::Quarantined
        } else if hallucination_score > self.config.hallucination_threshold * 0.7 {
            ValidationStatus::PartialTrust
        } else {
            ValidationStatus::Validated
        };

        let item_id = item.id;

        let result = ValidationResult {
            item_id,
            trust_score: trust,
            risk_level: risk,
            status: status.clone(),
            cross_references_found: cross_ref_count,
            hallucination_score,
            flags,
        };

        item.trust_score = trust;
        item.risk_level = result.risk_level;
        item.validation_status = status.clone();

        match status {
            ValidationStatus::Validated | ValidationStatus::PartialTrust => {
                self.validated_items.push(item);
                // Evict oldest validated items if at capacity
                if self.validated_items.len() > MAX_VALIDATED_ITEMS {
                    self.validated_items
                        .drain(0..self.validated_items.len() - MAX_VALIDATED_ITEMS);
                }
            }
            ValidationStatus::Quarantined | ValidationStatus::Rejected => {
                self.quarantined_items.push(item);
                // Evict oldest quarantined items if at capacity
                if self.quarantined_items.len() > MAX_QUARANTINED_ITEMS {
                    self.quarantined_items
                        .drain(0..self.quarantined_items.len() - MAX_QUARANTINED_ITEMS);
                }
            }
            _ => {}
        }

        self.trust_history.push((item_id, trust));
        if self.trust_history.len() > MAX_TRUST_HISTORY {
            self.trust_history
                .drain(..self.trust_history.len() - MAX_TRUST_HISTORY);
        }

        Ok(result)
    }

    fn detect_hallucination(&self, content: &str) -> f32 {
        let mut score = 0.0f32;

        let suspicious_patterns = [
            "according to my knowledge",
            "I believe",
            "it is said that",
            "some sources suggest",
            "approximately",
            "roughly",
        ];

        for pattern in &suspicious_patterns {
            if content.to_lowercase().contains(&pattern.to_lowercase()) {
                score += 0.1;
            }
        }

        if content.len() < 20 {
            score += 0.2;
        }

        if content.contains("??") || content.contains("!!!") {
            score += 0.1;
        }

        score.min(1.0)
    }

    fn assess_risk(&self, content: &str, source_type: &SourceType) -> RiskLevel {
        let content_lower = content.to_lowercase();

        let critical_keywords = ["malware", "virus", "hack", "exploit", "bomb"];
        for keyword in &critical_keywords {
            if content_lower.contains(keyword) {
                return RiskLevel::Critical;
            }
        }

        let high_keywords = ["password", "secret", "private key", "ssn"];
        for keyword in &high_keywords {
            if content_lower.contains(keyword) {
                return RiskLevel::High;
            }
        }

        if *source_type == SourceType::Internet {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    pub fn get_validated(&self) -> &[KnowledgeItem] {
        &self.validated_items
    }

    pub fn get_quarantined(&self) -> &[KnowledgeItem] {
        &self.quarantined_items
    }

    pub fn release_from_quarantine(&mut self, item_id: Uuid) -> bool {
        if let Some(pos) = self.quarantined_items.iter().position(|i| i.id == item_id) {
            let mut item = self.quarantined_items.remove(pos);
            item.validation_status = ValidationStatus::Validated;
            self.validated_items.push(item);
            true
        } else {
            false
        }
    }

    pub fn trust_history(&self) -> &[(Uuid, f32)] {
        &self.trust_history
    }

    pub fn config(&self) -> &KnowledgeValidationConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(source: SourceType, trust: f32) -> KnowledgeItem {
        KnowledgeItem {
            id: Uuid::new_v4(),
            content: "This is a factual statement about Rust programming language.".to_string(),
            source: "test".to_string(),
            source_type: source,
            trust_score: trust,
            risk_level: RiskLevel::Low,
            cross_references: vec!["ref1".to_string(), "ref2".to_string()],
            validation_status: ValidationStatus::Pending,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_validator_creation() {
        let config = KnowledgeValidationConfig::default();
        let validator = KnowledgeValidator::new(config);
        assert_eq!(validator.get_validated().len(), 0);
        assert_eq!(validator.get_quarantined().len(), 0);
    }

    #[test]
    fn test_validate_good_item() {
        let config = KnowledgeValidationConfig::default();
        let mut validator = KnowledgeValidator::new(config);
        let item = make_item(SourceType::Verified, 0.9);
        let result = validator.validate(item).unwrap();
        assert_eq!(result.status, ValidationStatus::Validated);
        assert!(result.trust_score > 0.5);
    }

    #[test]
    fn test_validate_internet_item() {
        let config = KnowledgeValidationConfig::default();
        let mut validator = KnowledgeValidator::new(config);
        let item = make_item(SourceType::Internet, 0.9);
        let result = validator.validate(item).unwrap();
        assert!(result.flags.iter().any(|f| f.contains("Internet")));
    }

    #[test]
    fn test_validate_insufficient_cross_references() {
        let config = KnowledgeValidationConfig::default();
        let mut validator = KnowledgeValidator::new(config);
        let mut item = make_item(SourceType::Verified, 0.9);
        item.cross_references = vec!["ref1".to_string()];
        let result = validator.validate(item).unwrap();
        assert!(result.flags.iter().any(|f| f.contains("cross-references")));
    }

    #[test]
    fn test_quarantine_release() {
        let config = KnowledgeValidationConfig::default();
        let mut validator = KnowledgeValidator::new(config);
        let item = make_item(SourceType::Verified, 0.1);
        let id = item.id;
        validator.validate(item).unwrap();
        assert_eq!(validator.get_quarantined().len(), 1);
        assert!(validator.release_from_quarantine(id));
        assert_eq!(validator.get_quarantined().len(), 0);
        assert_eq!(validator.get_validated().len(), 1);
    }

    #[test]
    fn test_hallucination_detection() {
        let config = KnowledgeValidationConfig::default();
        let validator = KnowledgeValidator::new(config);
        let score = validator.detect_hallucination("according to my knowledge, this is true");
        assert!(score > 0.0);
    }

    #[test]
    fn test_risk_assessment() {
        let config = KnowledgeValidationConfig::default();
        let validator = KnowledgeValidator::new(config);
        assert_eq!(
            validator.assess_risk("malware distribution", &SourceType::Verified),
            RiskLevel::Critical
        );
        assert_eq!(
            validator.assess_risk("my password is secret", &SourceType::Verified),
            RiskLevel::High
        );
        assert_eq!(
            validator.assess_risk("normal content", &SourceType::Internet),
            RiskLevel::Medium
        );
        assert_eq!(
            validator.assess_risk("normal content", &SourceType::Verified),
            RiskLevel::Low
        );
    }
}
