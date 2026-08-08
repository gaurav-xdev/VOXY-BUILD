use crate::types::{ContextSnapshot, ContextSource, FreshnessConfig};
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Configuration for the context cache.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of snapshots to cache per source.
    pub max_per_source: usize,

    /// Maximum total number of snapshots across all sources.
    pub max_total: usize,

    /// TTL for cached snapshots. After this duration, the snapshot is considered stale.
    pub ttl: Duration,

    /// Freshness configuration for staleness checks.
    pub freshness: FreshnessConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_per_source: 5,
            max_total: 50,
            ttl: Duration::from_secs(60),
            freshness: FreshnessConfig::default(),
        }
    }
}

/// Entry in the cache with metadata.
#[derive(Debug, Clone)]
struct CacheEntry {
    snapshot: ContextSnapshot,
    inserted_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}

/// LRU context cache that stores snapshots by source.
///
/// The cache supports TTL-based expiration, per-source limits,
/// and global capacity limits with LRU eviction.
pub struct ContextCache {
    entries: RwLock<HashMap<ContextSource, VecDeque<CacheEntry>>>,
    config: CacheConfig,
    stats: RwLock<CacheStats>,
}

/// Cache statistics for monitoring.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub insertions: u64,
    pub invalidations: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl ContextCache {
    /// Create a new cache with the given configuration.
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            config,
            stats: RwLock::new(CacheStats::default()),
        }
    }

    /// Create a cache with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Insert a snapshot into the cache.
    pub fn insert(&self, snapshot: ContextSnapshot) {
        let mut entries = self.entries.write();
        let source = snapshot.source.clone();
        let queue = entries.entry(source).or_default();

        // Evict oldest if at capacity for this source
        while queue.len() >= self.config.max_per_source {
            queue.pop_front();
            self.stats.write().evictions += 1;
        }

        let now = Instant::now();
        queue.push_back(CacheEntry {
            snapshot,
            inserted_at: now,
            access_count: 0,
            last_accessed: now,
        });

        self.stats.write().insertions += 1;

        // Check global capacity and evict if needed
        self.evict_if_needed(&mut entries);
    }

    /// Get the most recent valid (non-stale) snapshot for a source.
    pub fn get_latest(&self, source: &ContextSource) -> Option<ContextSnapshot> {
        let result = {
            let mut entries = self.entries.write();
            let queue = match entries.get_mut(source) {
                Some(q) => q,
                None => {
                    // Source not in cache — update stats and return None
                    drop(entries);
                    self.stats.write().misses += 1;
                    return None;
                }
            };

            // Find the most recent non-stale entry
            let now = Instant::now();
            let ttl = self.config.ttl;

            let mut found = None;
            for entry in queue.iter_mut().rev() {
                if now.duration_since(entry.inserted_at) <= ttl {
                    entry.access_count += 1;
                    entry.last_accessed = now;
                    found = Some(entry.snapshot.clone());
                    break;
                }
            }
            found
        };

        // Update stats outside the entries lock
        if result.is_some() {
            self.stats.write().hits += 1;
        } else {
            self.stats.write().misses += 1;
        }

        result
    }

    /// Get a snapshot by ID.
    pub fn get_by_id(&self, id: &str) -> Option<ContextSnapshot> {
        let result = {
            let mut entries = self.entries.write();
            let now = Instant::now();
            let ttl = self.config.ttl;

            let mut found = None;
            for queue in entries.values_mut() {
                for entry in queue.iter_mut() {
                    if entry.snapshot.id.0 == id && now.duration_since(entry.inserted_at) <= ttl {
                        entry.access_count += 1;
                        entry.last_accessed = now;
                        found = Some(entry.snapshot.clone());
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            found
        };

        if result.is_some() {
            self.stats.write().hits += 1;
        } else {
            self.stats.write().misses += 1;
        }

        result
    }

    /// Get all valid (non-stale) snapshots for a source.
    pub fn get_all_for_source(&self, source: &ContextSource) -> Vec<ContextSnapshot> {
        let entries = self.entries.read();
        let queue = match entries.get(source) {
            Some(q) => q,
            None => return vec![],
        };

        let now = Instant::now();
        let ttl = self.config.ttl;

        queue
            .iter()
            .filter(|e| now.duration_since(e.inserted_at) <= ttl)
            .map(|e| e.snapshot.clone())
            .collect()
    }

    /// Invalidate all cached snapshots for a source.
    pub fn invalidate_source(&self, source: &ContextSource) -> usize {
        let mut entries = self.entries.write();
        let removed = entries.remove(source).map(|q| q.len()).unwrap_or(0);
        if removed > 0 {
            self.stats.write().invalidations += removed as u64;
        }
        removed
    }

    /// Invalidate a specific snapshot by ID.
    pub fn invalidate_by_id(&self, id: &str) -> bool {
        let mut entries = self.entries.write();
        for queue in entries.values_mut() {
            if let Some(pos) = queue.iter().position(|e| e.snapshot.id.0 == id) {
                queue.remove(pos);
                self.stats.write().invalidations += 1;
                return true;
            }
        }
        false
    }

    /// Clear all cached snapshots.
    pub fn clear(&self) {
        let mut entries = self.entries.write();
        let count: usize = entries.values().map(|q| q.len()).sum();
        entries.clear();
        self.stats.write().invalidations += count as u64;
    }

    /// Remove stale entries (older than TTL).
    pub fn evict_stale(&self) -> usize {
        let mut entries = self.entries.write();
        let now = Instant::now();
        let ttl = self.config.ttl;
        let mut evicted = 0;

        for queue in entries.values_mut() {
            let before = queue.len();
            queue.retain(|e| now.duration_since(e.inserted_at) <= ttl);
            evicted += before - queue.len();
        }

        // Remove empty queues
        entries.retain(|_, q| !q.is_empty());

        self.stats.write().evictions += evicted as u64;
        evicted
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Get the total number of cached snapshots.
    pub fn len(&self) -> usize {
        self.entries.read().values().map(|q| q.len()).sum()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.read().values().all(|q| q.is_empty())
    }

    /// Get the number of cached snapshots for a specific source.
    pub fn len_for_source(&self, source: &ContextSource) -> usize {
        self.entries
            .read()
            .get(source)
            .map(|q| q.len())
            .unwrap_or(0)
    }

    /// Evict entries if global capacity is exceeded (LRU by last_accessed).
    fn evict_if_needed(&self, entries: &mut HashMap<ContextSource, VecDeque<CacheEntry>>) {
        let total: usize = entries.values().map(|q| q.len()).sum();
        if total <= self.config.max_total {
            return;
        }

        // Collect all entries with their source for LRU eviction
        let mut all_entries: Vec<(ContextSource, usize)> = Vec::new();
        for (source, queue) in entries.iter() {
            for i in 0..queue.len() {
                all_entries.push((source.clone(), i));
            }
        }

        // Sort by last_accessed (oldest first)
        all_entries.sort_by(|a, b| {
            let a_entry = &entries[&a.0][a.1];
            let b_entry = &entries[&b.0][b.1];
            a_entry.last_accessed.cmp(&b_entry.last_accessed)
        });

        // Remove oldest entries until under capacity
        let to_remove = total - self.config.max_total;
        for (source, idx) in all_entries.iter().take(to_remove.min(all_entries.len())) {
            if let Some(queue) = entries.get_mut(source) {
                if idx < &queue.len() {
                    queue.remove(*idx);
                    self.stats.write().evictions += 1;
                }
            }
        }
    }
}

impl Default for ContextCache {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(source: ContextSource) -> ContextSnapshot {
        ContextSnapshot::new(source, serde_json::json!({"test": true}))
    }

    #[test]
    fn test_insert_and_get() {
        let cache = ContextCache::with_defaults();
        let snapshot = make_snapshot(ContextSource::Environment);
        let id = snapshot.id.clone();

        cache.insert(snapshot);
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get_latest(&ContextSource::Environment);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);
    }

    #[test]
    fn test_get_by_id() {
        let cache = ContextCache::with_defaults();
        let snapshot = make_snapshot(ContextSource::Environment);
        let id = snapshot.id.clone();

        cache.insert(snapshot);
        let retrieved = cache.get_by_id(&id.0);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_invalidate_source() {
        let cache = ContextCache::with_defaults();
        cache.insert(make_snapshot(ContextSource::Environment));
        cache.insert(make_snapshot(ContextSource::Environment));

        let removed = cache.invalidate_source(&ContextSource::Environment);
        assert_eq!(removed, 2);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_lru_eviction() {
        let config = CacheConfig {
            max_per_source: 2,
            max_total: 100,
            ttl: Duration::from_secs(300),
            freshness: FreshnessConfig::default(),
        };
        let cache = ContextCache::new(config);

        cache.insert(make_snapshot(ContextSource::Environment));
        cache.insert(make_snapshot(ContextSource::Environment));
        cache.insert(make_snapshot(ContextSource::Environment)); // Should evict oldest

        assert_eq!(cache.len_for_source(&ContextSource::Environment), 2);
    }

    #[test]
    fn test_stats() {
        let cache = ContextCache::with_defaults();
        cache.insert(make_snapshot(ContextSource::Environment));

        let result1 = cache.get_latest(&ContextSource::Environment);
        assert!(result1.is_some(), "Expected hit for Environment");

        let result2 = cache.get_latest(&ContextSource::Conversation);
        assert!(result2.is_none(), "Expected miss for Conversation");

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 0.5).abs() < f64::EPSILON);
    }
}
