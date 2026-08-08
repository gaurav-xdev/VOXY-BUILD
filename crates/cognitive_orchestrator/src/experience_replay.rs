use crate::config::ExperienceReplayConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceEntry {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub input: String,
    pub output: String,
    pub context: String,
    pub scores: ExperienceScores,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub replay_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceScores {
    pub helpfulness: f32,
    pub speed: f32,
    pub correctness: f32,
    pub user_reaction: f32,
}

impl ExperienceScores {
    pub fn composite(&self) -> f32 {
        (self.helpfulness + self.speed + self.correctness + self.user_reaction) / 4.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub entry_id: Uuid,
    pub improved_decision: String,
    pub score_improvement: f32,
    pub insights: Vec<String>,
}

pub struct ExperienceReplay {
    config: ExperienceReplayConfig,
    buffer: Vec<ExperienceEntry>,
    replay_results: Vec<ReplayResult>,
}

const MAX_REPLAY_RESULTS: usize = 500;

impl ExperienceReplay {
    pub fn new(config: ExperienceReplayConfig) -> Self {
        Self {
            config,
            buffer: Vec::new(),
            replay_results: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: ExperienceEntry) {
        if self.buffer.len() >= self.config.buffer_size {
            self.buffer.swap_remove(0);
        }
        self.buffer.push(entry);
    }

    pub fn replay(&mut self, entry_id: Uuid) -> Result<Option<ReplayResult>> {
        let entry = match self.buffer.iter_mut().find(|e| e.id == entry_id) {
            Some(e) => e,
            None => return Ok(None),
        };

        entry.replay_count += 1;

        let score = entry.scores.composite();
        if score < self.config.min_replay_score {
            return Ok(Some(ReplayResult {
                entry_id,
                improved_decision: entry.output.clone(),
                score_improvement: 0.0,
                insights: vec!["Score below minimum threshold".to_string()],
            }));
        }

        let improvement = score * self.config.learning_rate;
        let mut insights = Vec::new();

        if entry.scores.helpfulness < 0.5 {
            insights.push("Response was not helpful enough".to_string());
        }
        if entry.scores.speed < 0.5 {
            insights.push("Response was too slow".to_string());
        }
        if entry.scores.correctness < 0.5 {
            insights.push("Response contained errors".to_string());
        }
        if entry.scores.user_reaction < 0.5 {
            insights.push("User was not satisfied".to_string());
        }

        if insights.is_empty() {
            insights.push("Performance was good".to_string());
        }

        let result = ReplayResult {
            entry_id,
            improved_decision: format!("Improved based on {} insights", insights.len()),
            score_improvement: improvement,
            insights,
        };

        self.replay_results.push(result.clone());

        // Evict oldest replay results if at capacity
        if self.replay_results.len() > MAX_REPLAY_RESULTS {
            self.replay_results
                .drain(0..self.replay_results.len() - MAX_REPLAY_RESULTS);
        }

        Ok(Some(result))
    }

    pub fn replay_top_k(&mut self, k: usize) -> Result<Vec<ReplayResult>> {
        let mut entries: Vec<_> = self
            .buffer
            .iter()
            .filter(|e| e.scores.composite() >= self.config.min_replay_score)
            .map(|e| (e.id, e.scores.composite()))
            .collect();

        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        entries.truncate(k);

        let mut results = Vec::new();
        for (id, _) in entries {
            if let Some(result) = self.replay(id)? {
                results.push(result);
            }
        }
        Ok(results)
    }

    pub fn get_entry(&self, id: Uuid) -> Option<&ExperienceEntry> {
        self.buffer.iter().find(|e| e.id == id)
    }

    pub fn buffer(&self) -> &[ExperienceEntry] {
        &self.buffer
    }

    pub fn replay_results(&self) -> &[ReplayResult] {
        &self.replay_results
    }

    pub fn average_score(&self) -> f32 {
        if self.buffer.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.buffer.iter().map(|e| e.scores.composite()).sum();
        sum / self.buffer.len() as f32
    }

    pub fn config(&self) -> &ExperienceReplayConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(score: f32) -> ExperienceEntry {
        ExperienceEntry {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            input: "test input".to_string(),
            output: "test output".to_string(),
            context: "test".to_string(),
            scores: ExperienceScores {
                helpfulness: score,
                speed: score,
                correctness: score,
                user_reaction: score,
            },
            timestamp: chrono::Utc::now(),
            replay_count: 0,
        }
    }

    #[test]
    fn test_experience_replay_creation() {
        let config = ExperienceReplayConfig::default();
        let replay = ExperienceReplay::new(config);
        assert_eq!(replay.buffer().len(), 0);
    }

    #[test]
    fn test_add_entry() {
        let config = ExperienceReplayConfig::default();
        let mut replay = ExperienceReplay::new(config);
        let entry = make_entry(0.8);
        let id = entry.id;
        replay.add_entry(entry);
        assert_eq!(replay.buffer().len(), 1);
        assert!(replay.get_entry(id).is_some());
    }

    #[test]
    fn test_replay_good_entry() {
        let config = ExperienceReplayConfig::default();
        let mut replay = ExperienceReplay::new(config);
        let entry = make_entry(0.8);
        let id = entry.id;
        replay.add_entry(entry);
        let result = replay.replay(id).unwrap();
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.score_improvement > 0.0);
    }

    #[test]
    fn test_replay_bad_entry() {
        let config = ExperienceReplayConfig::default();
        let mut replay = ExperienceReplay::new(config);
        let entry = make_entry(0.1);
        let id = entry.id;
        replay.add_entry(entry);
        let result = replay.replay(id).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_buffer_overflow() {
        let config = ExperienceReplayConfig {
            buffer_size: 3,
            ..Default::default()
        };
        let mut replay = ExperienceReplay::new(config);
        for _ in 0..5 {
            replay.add_entry(make_entry(0.5));
        }
        assert_eq!(replay.buffer().len(), 3);
    }

    #[test]
    fn test_replay_top_k() {
        let config = ExperienceReplayConfig::default();
        let mut replay = ExperienceReplay::new(config);
        replay.add_entry(make_entry(0.9));
        replay.add_entry(make_entry(0.8));
        replay.add_entry(make_entry(0.7));
        let results = replay.replay_top_k(2).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_average_score() {
        let config = ExperienceReplayConfig::default();
        let mut replay = ExperienceReplay::new(config);
        assert_eq!(replay.average_score(), 0.0);
        replay.add_entry(make_entry(0.8));
        replay.add_entry(make_entry(0.6));
        assert!((replay.average_score() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_experience_scores_composite() {
        let scores = ExperienceScores {
            helpfulness: 1.0,
            speed: 0.0,
            correctness: 1.0,
            user_reaction: 0.0,
        };
        assert_eq!(scores.composite(), 0.5);
    }
}
