use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tracing::{debug, info, warn};

use crate::types::SessionId;

const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(3600); // 1 hour
const DEFAULT_MAX_SESSIONS: usize = 100;
const PRUNE_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

/// Configuration for the SessionManager.
#[derive(Debug, Clone)]
pub struct SessionManagerConfig {
    /// Time-to-live for inactive sessions.
    pub session_ttl: Duration,
    /// Maximum number of active sessions before oldest are evicted.
    pub max_sessions: usize,
    /// How often to run the background pruner.
    pub prune_interval: Duration,
}

impl Default for SessionManagerConfig {
    fn default() -> Self {
        Self {
            session_ttl: DEFAULT_SESSION_TTL,
            max_sessions: DEFAULT_MAX_SESSIONS,
            prune_interval: PRUNE_INTERVAL,
        }
    }
}

/// Per-session metadata tracked by the session manager.
#[derive(Debug, Clone)]
pub struct ManagedSession {
    pub session_id: SessionId,
    pub created_at: Instant,
    pub last_active: Instant,
    pub turn_count: usize,
    pub error_count: usize,
}

impl ManagedSession {
    fn new(session_id: SessionId) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            created_at: now,
            last_active: now,
            turn_count: 0,
            error_count: 0,
        }
    }

    /// Check if this session has expired (no activity within TTL).
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.last_active.elapsed() > ttl
    }

    /// Touch the session to mark it as recently active.
    pub fn touch(&mut self) {
        self.last_active = Instant::now();
    }
}

/// Manages brain sessions with TTL-based expiration and capacity limits.
pub struct SessionManager {
    config: SessionManagerConfig,
    sessions: RwLock<HashMap<SessionId, ManagedSession>>,
    total_created: AtomicU64,
    total_expired: AtomicU64,
    total_evicted: AtomicU64,
}

impl SessionManager {
    pub fn new(config: SessionManagerConfig) -> Self {
        Self {
            config,
            sessions: RwLock::new(HashMap::new()),
            total_created: AtomicU64::new(0),
            total_expired: AtomicU64::new(0),
            total_evicted: AtomicU64::new(0),
        }
    }

    /// Create a new session or return an existing one.
    pub fn get_or_create(&self, session_id: &SessionId) -> ManagedSession {
        let mut sessions = self.sessions.write();

        // Return existing if present
        if let Some(session) = sessions.get_mut(session_id) {
            session.touch();
            return session.clone();
        }

        // Enforce capacity limit by evicting oldest
        if sessions.len() >= self.config.max_sessions {
            self.evict_oldest(&mut sessions);
        }

        // Create new session
        let session = ManagedSession::new(session_id.clone());
        sessions.insert(session_id.clone(), session.clone());
        self.total_created.fetch_add(1, Ordering::Relaxed);

        debug!(
            session_id = %session_id.0,
            total = sessions.len(),
            "Session created"
        );

        session
    }

    /// Touch a session to mark it active. Returns false if session doesn't exist.
    pub fn touch(&self, session_id: &SessionId) -> bool {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.touch();
            true
        } else {
            false
        }
    }

    /// Record a completed turn for a session.
    pub fn record_turn(&self, session_id: &SessionId) {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.turn_count += 1;
            session.touch();
        }
    }

    /// Record an error for a session.
    pub fn record_error(&self, session_id: &SessionId) {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.error_count += 1;
            session.touch();
        }
    }

    /// Manually remove a session.
    pub fn remove(&self, session_id: &SessionId) -> Option<ManagedSession> {
        self.sessions.write().remove(session_id)
    }

    /// Get a session snapshot without modifying it.
    pub fn get(&self, session_id: &SessionId) -> Option<ManagedSession> {
        self.sessions.read().get(session_id).cloned()
    }

    /// Get the total number of active sessions.
    pub fn active_count(&self) -> usize {
        self.sessions.read().len()
    }

    /// Get session statistics.
    pub fn stats(&self) -> SessionManagerStats {
        let sessions = self.sessions.read();
        let total_turns: usize = sessions.values().map(|s| s.turn_count).sum();
        let total_errors: usize = sessions.values().map(|s| s.error_count).sum();
        let oldest_age = sessions.values().map(|s| s.created_at.elapsed()).max();

        SessionManagerStats {
            active_sessions: sessions.len(),
            total_created: self.total_created.load(Ordering::Relaxed),
            total_expired: self.total_expired.load(Ordering::Relaxed),
            total_evicted: self.total_evicted.load(Ordering::Relaxed),
            total_turns,
            total_errors,
            oldest_session_age: oldest_age,
        }
    }

    /// Run the pruner to remove expired sessions.
    /// Returns the number of sessions pruned.
    pub fn prune(&self) -> usize {
        let mut sessions = self.sessions.write();
        let before = sessions.len();

        sessions.retain(|_id, session| {
            if session.is_expired(self.config.session_ttl) {
                self.total_expired.fetch_add(1, Ordering::Relaxed);
                false
            } else {
                true
            }
        });

        let pruned = before - sessions.len();
        if pruned > 0 {
            info!(
                pruned,
                remaining = sessions.len(),
                "Pruned expired sessions"
            );
        }
        pruned
    }

    fn evict_oldest(&self, sessions: &mut HashMap<SessionId, ManagedSession>) {
        if let Some(oldest_id) = sessions
            .iter()
            .min_by_key(|(_, s)| s.last_active)
            .map(|(id, _)| id.clone())
        {
            sessions.remove(&oldest_id);
            self.total_evicted.fetch_add(1, Ordering::Relaxed);
            warn!(
                evicted_session = %oldest_id.0,
                "Evicted oldest session due to capacity limit"
            );
        }
    }

    /// Shutdown the session manager, clearing all sessions.
    pub fn shutdown(&self) {
        let count = self.sessions.read().len();
        self.sessions.write().clear();
        info!(cleared = count, "Session manager shut down");
    }
}

#[derive(Debug, Clone)]
pub struct SessionManagerStats {
    pub active_sessions: usize,
    pub total_created: u64,
    pub total_expired: u64,
    pub total_evicted: u64,
    pub total_turns: usize,
    pub total_errors: usize,
    pub oldest_session_age: Option<Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let mgr = SessionManager::new(SessionManagerConfig::default());
        let id = SessionId::new();
        let session = mgr.get_or_create(&id);
        assert_eq!(session.session_id, id);
        assert_eq!(session.turn_count, 0);
        assert_eq!(mgr.active_count(), 1);
    }

    #[test]
    fn test_session_reuse() {
        let mgr = SessionManager::new(SessionManagerConfig::default());
        let id = SessionId::new();
        let s1 = mgr.get_or_create(&id);
        let s2 = mgr.get_or_create(&id);
        assert_eq!(s1.session_id, s2.session_id);
        assert_eq!(mgr.active_count(), 1);
    }

    #[test]
    fn test_record_turn() {
        let mgr = SessionManager::new(SessionManagerConfig::default());
        let id = SessionId::new();
        mgr.get_or_create(&id);
        mgr.record_turn(&id);
        mgr.record_turn(&id);
        let session = mgr.get(&id).unwrap();
        assert_eq!(session.turn_count, 2);
    }

    #[test]
    fn test_record_error() {
        let mgr = SessionManager::new(SessionManagerConfig::default());
        let id = SessionId::new();
        mgr.get_or_create(&id);
        mgr.record_error(&id);
        let session = mgr.get(&id).unwrap();
        assert_eq!(session.error_count, 1);
    }

    #[test]
    fn test_session_expiration() {
        let config = SessionManagerConfig {
            session_ttl: Duration::from_millis(1),
            ..Default::default()
        };
        let mgr = SessionManager::new(config);
        let id = SessionId::new();
        mgr.get_or_create(&id);

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(5));

        let pruned = mgr.prune();
        assert_eq!(pruned, 1);
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn test_capacity_eviction() {
        let config = SessionManagerConfig {
            max_sessions: 2,
            ..Default::default()
        };
        let mgr = SessionManager::new(config);

        let id1 = SessionId::new();
        let id2 = SessionId::new();
        let id3 = SessionId::new();

        mgr.get_or_create(&id1);
        mgr.get_or_create(&id2);
        mgr.get_or_create(&id3); // Should evict oldest

        assert_eq!(mgr.active_count(), 2);
        assert!(mgr.get(&id1).is_none()); // evicted
        assert!(mgr.get(&id2).is_some());
        assert!(mgr.get(&id3).is_some());
    }

    #[test]
    fn test_remove() {
        let mgr = SessionManager::new(SessionManagerConfig::default());
        let id = SessionId::new();
        mgr.get_or_create(&id);
        assert!(mgr.remove(&id).is_some());
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn test_stats() {
        let mgr = SessionManager::new(SessionManagerConfig::default());
        let id = SessionId::new();
        mgr.get_or_create(&id);
        mgr.record_turn(&id);
        mgr.record_turn(&id);

        let stats = mgr.stats();
        assert_eq!(stats.active_sessions, 1);
        assert_eq!(stats.total_created, 1);
        assert_eq!(stats.total_turns, 2);
    }

    #[test]
    fn test_shutdown() {
        let mgr = SessionManager::new(SessionManagerConfig::default());
        let id = SessionId::new();
        mgr.get_or_create(&id);
        mgr.shutdown();
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn test_session_ttl_default() {
        let config = SessionManagerConfig::default();
        assert_eq!(config.session_ttl, Duration::from_secs(3600));
        assert_eq!(config.max_sessions, 100);
    }

    #[test]
    fn test_multiple_sessions() {
        let mgr = SessionManager::new(SessionManagerConfig::default());
        let mut ids = Vec::new();
        for _ in 0..10 {
            let id = SessionId::new();
            mgr.get_or_create(&id);
            ids.push(id);
        }
        assert_eq!(mgr.active_count(), 10);

        // Touch first and third
        mgr.touch(&ids[0]);
        mgr.touch(&ids[2]);

        // Prune with very short TTL - should evict untouched
        let config = SessionManagerConfig {
            session_ttl: Duration::from_millis(1),
            ..Default::default()
        };
        let mgr2 = SessionManager::new(config);
        for id in &ids {
            mgr2.get_or_create(id);
        }
        std::thread::sleep(Duration::from_millis(5));
        let pruned = mgr2.prune();
        assert_eq!(pruned, 10);
    }
}
