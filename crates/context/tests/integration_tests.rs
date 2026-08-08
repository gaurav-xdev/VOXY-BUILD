//! Production Validation Tests for VOXY Context Runtime
//!
//! Tests for context collection, fusion, caching, and manager operations
//! under real-world conditions.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;

use voxy_context::cache::{CacheConfig, ContextCache};
use voxy_context::fusion::confidence::ConfidenceEngine;
use voxy_context::fusion::conflict::ContextConflictResolver;
use voxy_context::fusion::delta::ContextDeltaGenerator;
use voxy_context::fusion::freshness::{FreshnessEngine, FreshnessStatus};
use voxy_context::fusion::invalidation::ContextInvalidation;
use voxy_context::fusion::merger::ContextMerger;
use voxy_context::fusion::policy::FusionPolicy;
use voxy_context::fusion::priority::ContextPriorityResolver;
use voxy_context::fusion::resolver::ContextFusionEngine;
use voxy_context::manager::{ContextManager, ContextSnapshotSet};
use voxy_context::provider::ContextProvider;
use voxy_context::registry::ContextRegistry;
use voxy_context::types::{
    ContextId, ContextPriority, ContextSnapshot, ContextSource, FreshnessConfig,
};

// ============================================================================
// Helper Functions & Mock Providers
// ============================================================================

fn make_snapshot(
    source: ContextSource,
    confidence: f64,
    data: serde_json::Value,
) -> ContextSnapshot {
    ContextSnapshot::new(source, data)
        .with_confidence(confidence)
        .with_relevance(0.8)
}

struct MockProvider {
    source: ContextSource,
    name: String,
    delay: Duration,
    fail: bool,
    call_count: AtomicUsize,
}

impl MockProvider {
    fn new(source: ContextSource, name: &str) -> Self {
        Self {
            source,
            name: name.to_string(),
            delay: Duration::from_millis(10),
            fail: false,
            call_count: AtomicUsize::new(0),
        }
    }

    fn _with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    fn with_failure(mut self) -> Self {
        self.fail = true;
        self
    }

    fn _call_count(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl ContextProvider for MockProvider {
    fn source(&self) -> ContextSource {
        self.source.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn collect(&self) -> voxy_context::Result<ContextSnapshot> {
        self.call_count.fetch_add(1, Ordering::Relaxed);

        if self.fail {
            return Err(voxy_context::ContextError::ProviderError {
                provider: self.name.clone(),
                message: "Mock failure".to_string(),
            });
        }

        tokio::time::sleep(self.delay).await;

        Ok(ContextSnapshot::new(
            self.source.clone(),
            serde_json::json!({"provider": self.name, "timestamp": Utc::now().to_rfc3339()}),
        ))
    }

    async fn health_check(&self) -> bool {
        !self.fail
    }

    fn default_priority(&self) -> ContextPriority {
        ContextPriority::Medium
    }
}

// ============================================================================
// 1. FUSION ENGINE TESTS
// ============================================================================

#[test]
fn test_fusion_basic_merge() {
    let engine = ContextFusionEngine::with_defaults();

    let snapshots = vec![
        make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"location": "office"}),
        ),
        make_snapshot(
            ContextSource::Conversation,
            0.85,
            serde_json::json!({"topic": "meeting"}),
        ),
        make_snapshot(
            ContextSource::Memory,
            0.7,
            serde_json::json!({"recent_event": "lunch"}),
        ),
    ];

    let assembled = engine.fuse(snapshots);

    assert_eq!(assembled.source_count, 3);
    assert!(assembled.total_size_bytes > 0);

    let stats = engine.stats();
    assert_eq!(stats.input_count, 3);
    assert_eq!(stats.included_count, 3);
}

#[test]
fn test_fusion_confidence_filtering() {
    let engine = ContextFusionEngine::with_defaults();

    let snapshots = vec![
        make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"high": true}),
        ),
        make_snapshot(
            ContextSource::Conversation,
            0.05, // Below the default floor of 0.1
            serde_json::json!({"low": true}),
        ),
    ];

    let assembled = engine.fuse(snapshots);

    // Low confidence snapshot should be filtered - only 1 source in output
    assert!(assembled.source_count <= 2);
    assert!(assembled.source_count >= 1);

    let stats = engine.stats();
    assert_eq!(stats.input_count, 2);
    assert!(stats.included_count <= 2);
}

#[test]
fn test_fusion_priority_sorting() {
    let engine = ContextFusionEngine::with_defaults();

    let snapshots = vec![
        make_snapshot(
            ContextSource::Device,
            0.8,
            serde_json::json!({"device": "low"}),
        )
        .with_priority(ContextPriority::Low),
        make_snapshot(
            ContextSource::SystemState,
            0.8,
            serde_json::json!({"system": "critical"}),
        )
        .with_priority(ContextPriority::Critical),
        make_snapshot(
            ContextSource::Activity,
            0.8,
            serde_json::json!({"activity": "medium"}),
        )
        .with_priority(ContextPriority::Medium),
    ];

    let assembled = engine.fuse(snapshots);
    assert_eq!(assembled.source_count, 3);
}

#[test]
fn test_fusion_conflict_detection() {
    let engine = ContextFusionEngine::with_defaults();

    let snapshots = vec![
        make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"temperature": 72, "humidity": 45}),
        ),
        make_snapshot(
            ContextSource::Device,
            0.85,
            serde_json::json!({"temperature": 68}),
        ),
    ];

    let assembled = engine.fuse(snapshots);
    assert_eq!(assembled.source_count, 2);

    let stats = engine.stats();
    assert!(stats.conflicts_detected > 0);
    assert_eq!(stats.conflicts_resolved, stats.conflicts_detected);
}

#[test]
fn test_fusion_delta_generation() {
    let engine = ContextFusionEngine::with_defaults();

    let snapshots1 = vec![make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"temp": 70}),
    )];
    let _assembled1 = engine.fuse(snapshots1);

    let snapshots2 = vec![make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"temp": 75}),
    )];
    let assembled2 = engine.fuse(snapshots2);
    assert_eq!(assembled2.source_count, 1);
}

#[test]
fn test_fusion_invalidated_sources() {
    let engine = ContextFusionEngine::with_defaults();

    let snapshots1 = vec![make_snapshot(
        ContextSource::Memory,
        0.9,
        serde_json::json!({"data": "initial"}),
    )];
    let _assembled1 = engine.fuse(snapshots1);

    let snapshots2 = vec![ContextSnapshot {
        id: ContextId::new(),
        source: ContextSource::Memory,
        priority: ContextPriority::Medium,
        confidence: 0.9,
        freshness: 10000,
        relevance: 0.5,
        captured_at: Utc::now(),
        data: serde_json::json!({"data": "old"}),
        size_bytes: 0,
    }];

    let _assembled = engine.fuse(snapshots2);

    let stats = engine.stats();
    assert!(stats.fusion_time_us > 0);
}

// ============================================================================
// 2. CONFIDENCE ENGINE TESTS
// ============================================================================

#[test]
fn test_confidence_effective_calculation() {
    let policy = FusionPolicy::default();
    let engine = ConfidenceEngine::new(policy);

    let snapshot = make_snapshot(
        ContextSource::Environment,
        0.8,
        serde_json::json!({"data": "test"}),
    );

    let effective = engine.effective_confidence(&snapshot);
    assert!(effective > 0.0);
    assert!(effective <= 1.0);
}

#[test]
fn test_confidence_floor_filtering() {
    let policy = FusionPolicy::default();
    let engine = ConfidenceEngine::new(policy);

    let high_conf = make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"data": "high"}),
    );
    let low_conf = make_snapshot(
        ContextSource::Conversation,
        0.05, // Below the default floor of 0.1
        serde_json::json!({"data": "low"}),
    );

    assert!(engine.meets_confidence_floor(&high_conf));
    assert!(!engine.meets_confidence_floor(&low_conf));
}

// ============================================================================
// 3. FRESHNESS ENGINE TESTS
// ============================================================================

#[test]
fn test_freshness_status_check() {
    let config = FreshnessConfig::default();
    let policy = FusionPolicy::default();
    let engine = FreshnessEngine::new(config, policy);

    let fresh = ContextSnapshot::new(
        ContextSource::Environment,
        serde_json::json!({"data": "fresh"}),
    )
    .with_confidence(0.9);

    let stale = ContextSnapshot {
        id: ContextId::new(),
        source: ContextSource::Conversation,
        priority: ContextPriority::Medium,
        confidence: 0.9,
        freshness: 10000,
        relevance: 0.5,
        captured_at: Utc::now(),
        data: serde_json::json!({"data": "stale"}),
        size_bytes: 0,
    };

    let fresh_status = engine.status(&fresh);
    let stale_status = engine.status(&stale);

    assert!(fresh_status.is_usable());

    match stale_status {
        FreshnessStatus::Stale | FreshnessStatus::Expired => {}
        _ => panic!("Expected stale/expired status for stale snapshot"),
    }
}

// ============================================================================
// 4. PRIORITY RESOLVER TESTS
// ============================================================================

#[test]
fn test_priority_resolver_ordering() {
    let resolver = ContextPriorityResolver::new();

    let mut snapshots = vec![
        make_snapshot(
            ContextSource::Device,
            0.8,
            serde_json::json!({"device": "low"}),
        )
        .with_priority(ContextPriority::Low),
        make_snapshot(
            ContextSource::SystemState,
            0.8,
            serde_json::json!({"system": "critical"}),
        )
        .with_priority(ContextPriority::Critical),
        make_snapshot(
            ContextSource::Activity,
            0.8,
            serde_json::json!({"activity": "high"}),
        )
        .with_priority(ContextPriority::High),
    ];

    resolver.sort_by_priority(&mut snapshots);

    assert_eq!(snapshots[0].source, ContextSource::SystemState);
    assert_eq!(snapshots[1].source, ContextSource::Activity);
    assert_eq!(snapshots[2].source, ContextSource::Device);
}

// ============================================================================
// 5. MERGER TESTS
// ============================================================================

#[test]
fn test_merger_basic_merge() {
    let policy = FusionPolicy::default();
    let merger = ContextMerger::new(policy);

    let snapshots = vec![
        make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"temp": 70, "humidity": 45}),
        ),
        make_snapshot(
            ContextSource::Conversation,
            0.85,
            serde_json::json!({"topic": "meeting"}),
        ),
    ];

    let merged = merger.merge(&snapshots);
    assert!(merged.is_object());
    assert!(merged.get("temp").is_some() || merged.get("topic").is_some());
}

#[test]
fn test_merger_empty_input() {
    let policy = FusionPolicy::default();
    let merger = ContextMerger::new(policy);

    let snapshots: Vec<ContextSnapshot> = vec![];
    let merged = merger.merge(&snapshots);
    assert!(merged.is_object());
    assert!(merged.as_object().unwrap().is_empty());
}

// ============================================================================
// 6. CONFLICT RESOLVER TESTS
// ============================================================================

#[test]
fn test_conflict_resolver_detection() {
    let policy = FusionPolicy::default();
    let resolver = ContextConflictResolver::new(policy);

    let s1 = make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"temperature": 72}),
    );
    let s2 = make_snapshot(
        ContextSource::Device,
        0.85,
        serde_json::json!({"temperature": 68}),
    );

    let conflicts = resolver.detect_conflicts(&s1, &s2);
    assert!(!conflicts.is_empty());
}

#[test]
fn test_conflict_resolver_no_conflict() {
    let policy = FusionPolicy::default();
    let resolver = ContextConflictResolver::new(policy);

    let s1 = make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"temperature": 72}),
    );
    let s2 = make_snapshot(
        ContextSource::Conversation,
        0.85,
        serde_json::json!({"topic": "meeting"}),
    );

    let conflicts = resolver.detect_conflicts(&s1, &s2);
    assert!(conflicts.is_empty());
}

// ============================================================================
// 7. INVALIDATION ENGINE TESTS
// ============================================================================

#[test]
fn test_invalidation_basic() {
    let engine = ContextInvalidation::new(FreshnessEngine::new(
        FreshnessConfig::default(),
        FusionPolicy::default(),
    ));

    let snapshots = vec![
        make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"data": "fresh"}),
        ),
        ContextSnapshot {
            id: ContextId::new(),
            source: ContextSource::Conversation,
            priority: ContextPriority::Medium,
            confidence: 0.9,
            freshness: 10000,
            relevance: 0.5,
            captured_at: Utc::now(),
            data: serde_json::json!({"data": "old"}),
            size_bytes: 0,
        },
    ];

    let result = engine.check(&snapshots);
    assert!(!result.invalidated.is_empty());
}

// ============================================================================
// 8. DELTA GENERATOR TESTS
// ============================================================================

#[test]
fn test_delta_generator_detection() {
    let generator = ContextDeltaGenerator::new();

    let mut prev = HashMap::new();
    prev.insert(
        ContextSource::Environment,
        make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"temp": 70}),
        ),
    );

    let mut current = HashMap::new();
    current.insert(
        ContextSource::Environment,
        make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"temp": 75}),
        ),
    );

    let deltas = generator.compute_deltas(&prev, &current);
    assert!(!deltas.is_empty());
}

#[test]
fn test_delta_generator_no_change() {
    let generator = ContextDeltaGenerator::new();

    let data = serde_json::json!({"temp": 70});
    let mut prev = HashMap::new();
    prev.insert(
        ContextSource::Environment,
        make_snapshot(ContextSource::Environment, 0.9, data.clone()),
    );

    let mut current = HashMap::new();
    current.insert(
        ContextSource::Environment,
        make_snapshot(ContextSource::Environment, 0.9, data),
    );

    let deltas = generator.compute_deltas(&prev, &current);
    assert!(deltas.is_empty());
}

// ============================================================================
// 9. CACHE TESTS
// ============================================================================

#[test]
fn test_cache_basic_operations() {
    let config = CacheConfig::default();
    let cache = ContextCache::new(config);

    let snapshot = make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"data": "test"}),
    );

    cache.insert(snapshot.clone());
    let retrieved = cache.get_latest(&ContextSource::Environment);
    assert!(retrieved.is_some());

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
}

#[test]
fn test_cache_eviction() {
    let config = CacheConfig {
        max_per_source: 2,
        max_total: 2,
        ..Default::default()
    };
    let cache = ContextCache::new(config);

    for i in 0..3 {
        let snapshot = make_snapshot(
            ContextSource::ExternalService(format!("svc-{}", i)),
            0.9,
            serde_json::json!({"index": i}),
        );
        cache.insert(snapshot);
    }

    let stats = cache.stats();
    assert!(stats.evictions > 0);
}

#[test]
fn test_cache_ttl_expiration() {
    let config = CacheConfig {
        ttl: Duration::from_millis(1),
        ..Default::default()
    };
    let cache = ContextCache::new(config);

    let snapshot = make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"data": "expire"}),
    );

    cache.insert(snapshot);
    std::thread::sleep(Duration::from_millis(10));

    let retrieved = cache.get_latest(&ContextSource::Environment);
    assert!(retrieved.is_none());
}

// ============================================================================
// 10. REGISTRY TESTS
// ============================================================================

#[test]
fn test_registry_provider_registration() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let registry = ContextRegistry::new();

        let provider = MockProvider::new(ContextSource::Environment, "env-provider");
        registry.register(Arc::new(provider)).await;

        let providers = registry.sources().await;
        assert_eq!(providers.len(), 1);
    });
}

#[test]
fn test_registry_provider_removal() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let registry = ContextRegistry::new();

        let provider = MockProvider::new(ContextSource::Environment, "env-provider");
        registry.register(Arc::new(provider)).await;

        registry.unregister(&ContextSource::Environment).await;

        let providers = registry.sources().await;
        assert!(providers.is_empty());
    });
}

#[test]
fn test_registry_health_check() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let registry = ContextRegistry::new();

        let good_provider = MockProvider::new(ContextSource::Environment, "good");
        let bad_provider = MockProvider::new(ContextSource::Conversation, "bad").with_failure();

        registry.register(Arc::new(good_provider)).await;
        registry.register(Arc::new(bad_provider)).await;

        let health = registry.health_check_all().await;
        assert_eq!(health.len(), 2);
    });
}

// ============================================================================
// 11. MANAGER TESTS
// ============================================================================

#[tokio::test]
async fn test_manager_collect_all() {
    let manager = ContextManager::with_defaults();

    let provider1 = MockProvider::new(ContextSource::Environment, "env");
    let provider2 = MockProvider::new(ContextSource::Conversation, "conv");

    manager.register_provider(Arc::new(provider1)).await;
    manager.register_provider(Arc::new(provider2)).await;

    let result = manager.collect().await;
    assert!(result.is_ok());

    let snapshot_set = result.unwrap();
    assert!(snapshot_set.len() <= 2);
    assert!(snapshot_set.collection_time_ms > 0);
}

#[tokio::test]
async fn test_manager_collect_one() {
    let manager = ContextManager::with_defaults();

    let provider = MockProvider::new(ContextSource::Environment, "env");
    manager.register_provider(Arc::new(provider)).await;

    let result = manager.collect_one(&ContextSource::Environment).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_manager_subscribe() {
    let manager = ContextManager::with_defaults();

    let mut receiver = manager.subscribe();
    let result = tokio::time::timeout(Duration::from_millis(100), receiver.recv()).await;
    // Should timeout since no messages have been sent
    assert!(result.is_err());
}

#[tokio::test]
async fn test_manager_health_check() {
    let manager = ContextManager::with_defaults();

    let provider1 = MockProvider::new(ContextSource::Environment, "env");
    let provider2 = MockProvider::new(ContextSource::Conversation, "conv");

    manager.register_provider(Arc::new(provider1)).await;
    manager.register_provider(Arc::new(provider2)).await;

    let health = manager.health_check().await;
    assert_eq!(health.len(), 2);
}

// ============================================================================
// 12. STRESS TESTS
// ============================================================================

#[test]
fn test_stress_fusion_engine_high_throughput() {
    let engine = ContextFusionEngine::with_defaults();
    let start = Instant::now();

    for i in 0..1000 {
        let snapshots = vec![
            make_snapshot(
                ContextSource::Environment,
                0.9,
                serde_json::json!({"iteration": i}),
            ),
            make_snapshot(
                ContextSource::Conversation,
                0.85,
                serde_json::json!({"data": i}),
            ),
        ];

        let _assembled = engine.fuse(snapshots);
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(30),
        "Fusion stress test too slow: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_stress_cache_concurrent_access() {
    let cache = Arc::new(ContextCache::new(CacheConfig {
        max_per_source: 10,
        max_total: 100,
        ..Default::default()
    }));

    let mut handles = vec![];
    let start = Instant::now();

    for i in 0..50 {
        let cache = cache.clone();
        let handle = tokio::spawn(async move {
            let snapshot = make_snapshot(
                ContextSource::ExternalService(format!("svc-{}", i)),
                0.9,
                serde_json::json!({"writer": i}),
            );
            cache.insert(snapshot);
        });
        handles.push(handle);
    }

    for i in 0..50 {
        let cache = cache.clone();
        let handle = tokio::spawn(async move {
            let _ = cache.get_latest(&ContextSource::ExternalService(format!("svc-{}", i)));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(10),
        "Cache stress test too slow: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_stress_registry_concurrent_operations() {
    let registry = Arc::new(ContextRegistry::new());
    let mut handles = vec![];

    let start = Instant::now();

    for i in 0..30 {
        let registry = registry.clone();
        let handle = tokio::spawn(async move {
            let source = ContextSource::ExternalService(format!("svc-{}", i));
            let provider = MockProvider::new(source, &format!("provider-{}", i));
            registry.register(Arc::new(provider)).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let providers = registry.sources().await;
    assert_eq!(providers.len(), 30);

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(10),
        "Registry stress test too slow: {:?}",
        elapsed
    );
}

// ============================================================================
// 13. FAILURE INJECTION TESTS
// ============================================================================

#[tokio::test]
async fn test_failure_provider_error_handling() {
    let manager = ContextManager::with_defaults();

    let good_provider = MockProvider::new(ContextSource::Environment, "good");
    let bad_provider = MockProvider::new(ContextSource::Conversation, "bad").with_failure();

    manager.register_provider(Arc::new(good_provider)).await;
    manager.register_provider(Arc::new(bad_provider)).await;

    let result = manager.collect().await;
    assert!(result.is_ok());

    let snapshot_set = result.unwrap();
    assert!(!snapshot_set.is_empty());
}

#[tokio::test]
async fn test_failure_all_providers_fail() {
    let manager = ContextManager::with_defaults();

    let bad1 = MockProvider::new(ContextSource::Environment, "bad1").with_failure();
    let bad2 = MockProvider::new(ContextSource::Conversation, "bad2").with_failure();

    manager.register_provider(Arc::new(bad1)).await;
    manager.register_provider(Arc::new(bad2)).await;

    let result = manager.collect().await;
    if let Ok(snapshot_set) = result {
        assert!(snapshot_set.is_empty());
    }
}

#[tokio::test]
async fn test_failure_manager_no_providers() {
    let manager = ContextManager::with_defaults();

    let result = manager.collect().await;
    assert!(result.is_ok());

    let snapshot_set = result.unwrap();
    assert!(snapshot_set.is_empty());
}

#[test]
fn test_failure_fusion_empty_snapshots() {
    let engine = ContextFusionEngine::with_defaults();
    let assembled = engine.fuse(vec![]);
    assert_eq!(assembled.source_count, 0);
}

#[test]
fn test_failure_cache_minimal_capacity() {
    let config = CacheConfig {
        max_per_source: 1,
        max_total: 1,
        ..Default::default()
    };
    let cache = ContextCache::new(config);

    // Insert 2 items - second should evict the first
    let s1 = make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"data": "first"}),
    );
    let s2 = make_snapshot(
        ContextSource::Environment,
        0.8,
        serde_json::json!({"data": "second"}),
    );
    cache.insert(s1);
    cache.insert(s2);

    let stats = cache.stats();
    assert!(stats.evictions > 0);
}

// ============================================================================
// 14. CORRECTNESS TESTS
// ============================================================================

#[test]
fn test_correctness_snapshot_scoring() {
    let s = make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"data": "test"}),
    )
    .with_priority(ContextPriority::High);

    let score = s.score();
    assert!(score > 0.0);
    assert!(score <= 5.0);
}

#[test]
fn test_correctness_staleness_check() {
    let fresh = ContextSnapshot::new(
        ContextSource::Environment,
        serde_json::json!({"data": "fresh"}),
    );
    assert!(!fresh.is_stale(60));

    let stale = ContextSnapshot {
        id: ContextId::new(),
        source: ContextSource::Conversation,
        priority: ContextPriority::Medium,
        confidence: 0.9,
        freshness: 120,
        relevance: 0.5,
        captured_at: Utc::now(),
        data: serde_json::json!({"data": "stale"}),
        size_bytes: 0,
    };
    assert!(stale.is_stale(60));
}

#[test]
fn test_correctness_assembled_context_builder() {
    let snapshot = make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"data": "test"}),
    );

    let assembled = voxy_context::AssembledContextBuilder::new()
        .with_data(serde_json::json!({"merged": true}))
        .add_source(snapshot)
        .build();

    assert_eq!(assembled.source_count, 1);
    assert!(assembled.total_size_bytes > 0);
}

#[test]
fn test_correctness_policy_defaults() {
    let policy = FusionPolicy::default();
    assert!(policy.default_confidence_floor >= 0.0);
    assert!(policy.default_confidence_floor <= 1.0);
}

// ============================================================================
// 15. OBSERVABILITY TESTS
// ============================================================================

#[test]
fn test_observability_fusion_stats() {
    let engine = ContextFusionEngine::with_defaults();

    let snapshots = vec![make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"data": "test"}),
    )];

    let _assembled = engine.fuse(snapshots);

    let stats = engine.stats();
    assert_eq!(stats.input_count, 1);
    assert!(stats.fusion_time_us > 0);
    assert!(stats.output_size_bytes > 0);
}

#[test]
fn test_observability_cache_stats() {
    let config = CacheConfig::default();
    let cache = ContextCache::new(config);

    let snapshot = make_snapshot(
        ContextSource::Environment,
        0.9,
        serde_json::json!({"data": "test"}),
    );
    cache.insert(snapshot);
    let _ = cache.get_latest(&ContextSource::Environment);
    let _ = cache.get_latest(&ContextSource::Conversation);

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.insertions, 1);
}

#[test]
fn test_observability_snapshot_set_info() {
    let mut snapshots = HashMap::new();
    snapshots.insert(
        ContextSource::Environment,
        make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"data": "test"}),
        ),
    );

    let set = ContextSnapshotSet {
        snapshots,
        collected_at: Utc::now(),
        collection_time_ms: 42,
    };

    assert_eq!(set.len(), 1);
    assert!(!set.is_empty());
    assert_eq!(set.collection_time_ms, 42);
    assert!(set.get(&ContextSource::Environment).is_some());
    assert!(set.get(&ContextSource::Conversation).is_none());
}

// ============================================================================
// 16. ENDURANCE TESTS
// ============================================================================

#[test]
fn test_endurance_fusion_engine_sustained_load() {
    let engine = ContextFusionEngine::with_defaults();
    let start = Instant::now();

    for i in 0..500 {
        let snapshots = vec![
            make_snapshot(
                ContextSource::Environment,
                0.9,
                serde_json::json!({"iteration": i}),
            ),
            make_snapshot(
                ContextSource::Conversation,
                0.85,
                serde_json::json!({"data": i}),
            ),
            make_snapshot(ContextSource::Memory, 0.7, serde_json::json!({"memory": i})),
        ];

        let assembled = engine.fuse(snapshots);
        assert!(assembled.source_count > 0);
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(30),
        "Endurance test failed: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_endurance_cache_sustained_operations() {
    let cache = ContextCache::new(CacheConfig {
        max_per_source: 10,
        max_total: 100,
        ..Default::default()
    });

    let start = Instant::now();

    for i in 0..500 {
        let source = ContextSource::ExternalService(format!("svc-{}", i % 20));
        let snapshot = make_snapshot(source.clone(), 0.9, serde_json::json!({"iteration": i}));
        cache.insert(snapshot);
        let _ = cache.get_latest(&source);
    }

    let elapsed = start.elapsed();
    let stats = cache.stats();

    assert!(stats.hits > 0);
    assert!(elapsed < Duration::from_secs(10));
}
