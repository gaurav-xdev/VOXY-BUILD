use std::time::Instant;

use crate::types::MemoryKind;

/// A memory moment — a reference to past shared work.
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub text: String,
    pub kind: MemoryKind,
    pub timestamp: Instant,
    pub relevance: f64,
    pub used_count: usize,
}

/// Shared journey engine — remembers and references past work.
pub struct SharedJourney {
    entries: Vec<MemoryEntry>,
    max_entries: usize,
}

impl SharedJourney {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Record a milestone or event.
    pub fn record(&mut self, text: &str, kind: MemoryKind) {
        self.entries.push(MemoryEntry {
            text: text.to_string(),
            kind,
            timestamp: Instant::now(),
            relevance: 0.8,
            used_count: 0,
        });
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Find a relevant memory to reference.
    pub fn find_relevant(&mut self, context: &str) -> Option<String> {
        let lower_context = context.to_lowercase();

        let mut candidates: Vec<(usize, f64)> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.used_count < 3)
            .map(|(i, e)| {
                let text_match = if lower_context.contains(&e.text.to_lowercase()) {
                    0.3
                } else {
                    0.0
                };
                let recency = (1.0 - (e.timestamp.elapsed().as_secs_f64() / 86400.0)).max(0.0);
                let relevance = e.relevance * 0.5 + recency * 0.3 + text_match;
                (i, relevance)
            })
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((idx, _score)) = candidates.first() {
            let entry = &mut self.entries[*idx];
            entry.used_count += 1;
            Some(entry.text.clone())
        } else {
            None
        }
    }

    /// Generate a natural sentence referencing past work.
    pub fn generate_reference(&mut self, context: &str) -> Option<String> {
        self.find_relevant(context).map(|text| {
            let templates = [
                format!("Yesterday: {}.", text),
                format!("We finished {}.", text),
                format!("The {} is stable now.", text),
                format!("This is related to {}.", text),
            ];
            let idx = (Instant::now().elapsed().as_millis() as usize) % templates.len();
            templates[idx].clone()
        })
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for SharedJourney {
    fn default() -> Self {
        Self::new(50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_retrieve() {
        let mut journey = SharedJourney::new(10);
        journey.record("Context Fusion Engine", MemoryKind::Milestone);
        let reference = journey.generate_reference("Context");
        assert!(reference.is_some());
        assert!(reference.unwrap().contains("Context Fusion Engine"));
    }

    #[test]
    fn test_max_entries() {
        let mut journey = SharedJourney::new(3);
        for i in 0..5 {
            journey.record(&format!("Event {}", i), MemoryKind::Achievement);
        }
        assert_eq!(journey.entry_count(), 3);
    }

    #[test]
    fn test_used_count_limits() {
        let mut journey = SharedJourney::new(10);
        journey.record("Test", MemoryKind::Milestone);
        for _ in 0..5 {
            journey.find_relevant("Test");
        }
        let result = journey.find_relevant("Test");
        assert!(result.is_none());
    }
}
