use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiActionEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub subject: String,
    pub action: String,
    pub resource: Option<String>,
    pub reason: String,
    pub model: String,
    pub confidence: f64,
    pub tool: String,
    pub input_summary: String,
    pub output_summary: String,
    pub rollback_strategy: Option<String>,
    pub rollback_status: Option<String>,
    pub rollback_executed_at: Option<DateTime<Utc>>,
    pub risk_level: String,
    pub execution_time_ms: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct AiActionJournal {
    entries: Vec<AiActionEntry>,
}

impl AiActionJournal {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        subject: &str,
        action: &str,
        resource: Option<&str>,
        reason: &str,
        model: &str,
        confidence: f64,
        tool: &str,
        input_summary: &str,
        output_summary: &str,
        rollback_strategy: Option<&str>,
        risk_level: &str,
        execution_time_ms: u64,
    ) -> Uuid {
        let entry = AiActionEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            subject: subject.to_string(),
            action: action.to_string(),
            resource: resource.map(|r| r.to_string()),
            reason: reason.to_string(),
            model: model.to_string(),
            confidence,
            tool: tool.to_string(),
            input_summary: input_summary.to_string(),
            output_summary: output_summary.to_string(),
            rollback_strategy: rollback_strategy.map(|r| r.to_string()),
            rollback_status: None,
            rollback_executed_at: None,
            risk_level: risk_level.to_string(),
            execution_time_ms,
            metadata: std::collections::HashMap::new(),
        };
        let id = entry.id;
        self.entries.push(entry);
        id
    }

    pub fn mark_rollback(&mut self, id: Uuid, status: &str) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.rollback_status = Some(status.to_string());
            entry.rollback_executed_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    pub fn query_by_subject(&self, subject: &str) -> Vec<&AiActionEntry> {
        self.entries
            .iter()
            .filter(|e| e.subject == subject)
            .collect()
    }

    pub fn query_by_tool(&self, tool: &str) -> Vec<&AiActionEntry> {
        self.entries.iter().filter(|e| e.tool == tool).collect()
    }

    pub fn query_by_time_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<&AiActionEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .collect()
    }

    pub fn entries(&self) -> &[AiActionEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for AiActionJournal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_records_action() {
        let mut journal = AiActionJournal::new();
        let id = journal.record(
            "agent-1",
            "file:write",
            Some("/tmp/test.txt"),
            "User requested file save",
            "gpt-4o",
            0.95,
            "write_file",
            "Write to /tmp/test.txt",
            "File written successfully",
            Some("file_backup"),
            "medium",
            150,
        );
        assert!(!id.is_nil());
        assert_eq!(journal.len(), 1);
    }

    #[test]
    fn journal_query_by_subject() {
        let mut journal = AiActionJournal::new();
        journal.record(
            "agent-1", "action-1", None, "reason", "model", 0.9, "tool-1", "in", "out", None,
            "low", 100,
        );
        journal.record(
            "agent-2", "action-2", None, "reason", "model", 0.8, "tool-2", "in", "out", None,
            "low", 200,
        );
        assert_eq!(journal.query_by_subject("agent-1").len(), 1);
        assert_eq!(journal.query_by_subject("agent-2").len(), 1);
    }

    #[test]
    fn journal_query_by_tool() {
        let mut journal = AiActionJournal::new();
        journal.record(
            "agent",
            "action",
            None,
            "reason",
            "model",
            0.9,
            "write_file",
            "in",
            "out",
            None,
            "low",
            100,
        );
        journal.record(
            "agent",
            "action",
            None,
            "reason",
            "model",
            0.9,
            "read_file",
            "in",
            "out",
            None,
            "low",
            100,
        );
        assert_eq!(journal.query_by_tool("write_file").len(), 1);
        assert_eq!(journal.query_by_tool("read_file").len(), 1);
    }

    #[test]
    fn journal_mark_rollback() {
        let mut journal = AiActionJournal::new();
        let id = journal.record(
            "agent",
            "file:delete",
            Some("/tmp/x"),
            "Cleanup",
            "model",
            0.9,
            "delete_file",
            "Delete /tmp/x",
            "Deleted",
            Some("file_backup"),
            "high",
            50,
        );
        assert!(journal.mark_rollback(id, "completed"));
        let entry = journal
            .query_by_subject("agent")
            .into_iter()
            .find(|e| e.id == id)
            .unwrap();
        assert_eq!(entry.rollback_status.as_deref(), Some("completed"));
        assert!(entry.rollback_executed_at.is_some());
    }

    #[test]
    fn journal_entry_confidence() {
        let mut journal = AiActionJournal::new();
        journal.record(
            "agent", "action", None, "reason", "model", 0.95, "tool", "in", "out", None, "low", 100,
        );
        journal.record(
            "agent", "action", None, "reason", "model", 0.50, "tool", "in", "out", None, "low", 100,
        );
        let entries = journal.entries();
        assert!(entries[0].confidence > entries[1].confidence);
    }
}
