# VOXY Storage Architecture Design

**Status**: DRAFT — For Internal Review
**Version**: 1.0
**Author**: VOXY Architecture Team
**Date**: 2026-07-24

---

## 1. Overview

The Storage layer provides a **unified persistence abstraction** for all VOXY data. The core system **never depends directly on SQLite** — it only depends on the `StorageProvider` trait.

### Data Categories

| Category | Examples | Access Pattern | Consistency |
|----------|----------|----------------|-------------|
| **Configuration** | App config, plugin config | Read-heavy, rare writes | Strong |
| **Agent Memory** | Episodic, semantic, procedural | Mixed, vector search | Eventual OK |
| **Conversations** | Chat history, context | Append-heavy, time-range queries | Strong |
| **Plugin Data** | Per-plugin isolated storage | Varies by plugin | Strong |
| **Metrics/Telemetry** | Counters, histograms, traces | High-write, time-series | Eventual |
| **Model Cache** | Embeddings, quantized weights | Read-heavy, large blobs | Eventual |
| **Audit Logs** | Security, compliance | Append-only, tamper-evident | Strong |
| **Secrets** | API keys, tokens | Rare access, encrypted | Strong |

---

## 2. StorageProvider Trait (Complete)

```rust
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait StorageProvider: Send + Sync + 'static {
    // === Identity ===
    fn provider_name(&self) -> &'static str;
    fn provider_version(&self) -> &str;
    fn supported_features(&self) -> ProviderFeatures;
    
    // === Lifecycle ===
    async fn initialize(&self, config: &ProviderConfig) -> Result<()>;
    async fn health_check(&self) -> HealthStatus;
    async fn shutdown(&self) -> Result<()>;
    
    // === Transaction Support ===
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>>;
    async fn execute_in_transaction<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut dyn Transaction) -> Fut + Send,
        Fut: Future<Output = Result<T>> + Send,
        T: Send;
    
    // === Key-Value (Namespaced) ===
    async fn kv_get<K, V>(&self, namespace: &str, key: &K) -> Result<Option<V>>
    where K: Serialize + Send, V: DeserializeOwned + Send;
    
    async fn kv_set<K, V>(&self, namespace: &str, key: &K, value: &V, ttl: Option<Duration>) -> Result<()>
    where K: Serialize + Send, V: Serialize + Send;
    
    async fn kv_delete<K>(&self, namespace: &str, key: &K) -> Result<bool>
    where K: Serialize + Send;
    
    async fn kv_list<K>(&self, namespace: &str, prefix: &K, limit: usize) -> Result<Vec<K>>
    where K: Serialize + Send;
    
    async fn kv_exists<K>(&self, namespace: &str, key: &K) -> Result<bool>
    where K: Serialize + Send;
    
    // === Vector Search ===
    async fn vector_upsert(&self, namespace: &str, id: &str, vector: &[f32], metadata: &Metadata) -> Result<()>;
    async fn vector_search(&self, namespace: &str, query: &[f32], top_k: usize, filter: Option<&Filter>) -> Result<Vec<VectorMatch>>;
    async fn vector_delete(&self, namespace: &str, id: &str) -> Result<bool>;
    async fn vector_get(&self, namespace: &str, id: &str) -> Result<Option<VectorRecord>>;
    
    // === Time-Series ===
    async fn ts_append(&self, namespace: &str, series: &str, point: TimeSeriesPoint) -> Result<()>;
    async fn ts_query(&self, namespace: &str, series: &str, range: TimeRange, agg: Option<Aggregation>) -> Result<Vec<TimeSeriesPoint>>;
    async fn ts_delete_range(&self, namespace: &str, series: &str, range: TimeRange) -> Result<u64>;
    
    // === Blobs ===
    async fn blob_put(&self, namespace: &str, id: &str, data: &[u8], metadata: &BlobMetadata) -> Result<()>;
    async fn blob_get(&self, namespace: &str, id: &str) -> Result<Option<Blob>>;
    async fn blob_delete(&self, namespace: &str, id: &str) -> Result<bool>;
    async fn blob_exists(&self, namespace: &str, id: &str) -> Result<bool>;
    
    // === Admin ===
    async fn backup(&self, destination: &BackupDestination) -> Result<BackupInfo>;
    async fn restore(&self, source: &BackupSource) -> Result<()>;
    async fn vacuum(&self) -> Result<()>;
    async fn stats(&self) -> Result<StorageStats>;
    async fn migrate(&self, migrations: &[Migration]) -> Result<MigrationReport>;
}

// === Transaction Trait ===
#[async_trait]
pub trait Transaction: Send + Sync {
    async fn commit(self: Box<Self>) -> Result<()>;
    async fn rollback(self: Box<Self>) -> Result<()>;
    
    // All KV, Vector, TS, Blob methods available in transaction context
    async fn kv_get<K, V>(&mut self, namespace: &str, key: &K) -> Result<Option<V>>
    where K: Serialize + Send, V: DeserializeOwned + Send;
    // ... etc
}
```

---

## 3. Provider Implementations

### 3.1 SQLite Provider (Default)

```rust
pub struct SqliteProvider {
    pool: SqlitePool,
    config: SqliteConfig,
    vector_extension: Option<SqliteVecExtension>,
}

impl SqliteProvider {
    pub async fn new(config: SqliteConfig) -> Result<Self> {
        let pool = SqlitePool::connect_with(config.connect_options())
            .await?;
        
        // Enable WAL mode
        sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
        sqlx::query("PRAGMA synchronous=NORMAL").execute(&pool).await?;
        sqlx::query("PRAGMA busy_timeout=5000").execute(&pool).await?;
        
        // Load sqlite-vec if available
        let vector_extension = if config.enable_vectors {
            SqliteVecExtension::load(&pool).await.ok()
        } else { None };
        
        Ok(Self { pool, config, vector_extension })
    }
}
```

**Schema Strategy:**
- One table per namespace type (kv, vector, ts, blob)
- JSONB for flexible metadata
- Indexes on namespace + key prefixes
- Triggers for updated_at, TTL cleanup

```sql
-- Key-Value
CREATE TABLE kv_store (
    namespace TEXT NOT NULL,
    key TEXT NOT NULL,
    value BLOB NOT NULL,
    metadata JSON DEFAULT '{}',
    expires_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (namespace, key)
);
CREATE INDEX idx_kv_expires ON kv_store(expires_at) WHERE expires_at IS NOT NULL;

-- Vectors (with sqlite-vec)
CREATE VIRTUAL TABLE vec_store USING vec0(
    namespace TEXT,
    id TEXT PRIMARY KEY,
    vector FLOAT[768],  -- dimension configurable
    metadata JSON
);

-- Time-Series
CREATE TABLE ts_store (
    namespace TEXT NOT NULL,
    series TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    value REAL NOT NULL,
    tags JSON DEFAULT '{}',
    PRIMARY KEY (namespace, series, timestamp)
);
CREATE INDEX idx_ts_range ON ts_store(namespace, series, timestamp);

-- Blobs
CREATE TABLE blob_store (
    namespace TEXT NOT NULL,
    id TEXT NOT NULL,
    data BLOB NOT NULL,
    metadata JSON DEFAULT '{}',
    size INTEGER NOT NULL,
    content_type TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (namespace, id)
);
```

### 3.2 sqlite-vec Provider (Vector-Optimized)

```rust
pub struct SqliteVecProvider {
    inner: SqliteProvider,
    hnsw_index: Option<HnswIndex>,
}

impl SqliteVecProvider {
    // Uses sqlite-vec extension for HNSW indexing
    // Supports: L2, cosine, dot product
    // Automatic index rebuild on threshold
}
```

### 3.3 PostgreSQL Provider (Enterprise)

```rust
pub struct PostgresProvider {
    pool: PgPool,
    config: PostgresConfig,
    prepared_statements: PreparedStatementCache,
}

impl PostgresProvider {
    // Connection pooling with PgBouncer support
    // Advisory locks for distributed coordination
    // LISTEN/NOTIFY for real-time invalidation
    // Partitioned tables for time-series
    // pgvector extension for vectors
}
```

**Schema:**
```sql
-- Uses native JSONB, BRIN indexes for time-series
-- Partitioned by month for ts_store
-- pgvector for HNSW/IVFFlat indexes
```

### 3.4 DuckDB Provider (Analytics)

```rust
pub struct DuckDbProvider {
    conn: DuckDbConnection,
    config: DuckDbConfig,
}

impl DuckDbProvider {
    // Columnar storage for analytical queries
    // Direct Parquet/Arrow interop
    // No persistent server — embedded
    // Excellent for: metric aggregation, memory analysis
}
```

### 3.5 Remote Provider (Distributed)

```rust
pub struct RemoteProvider {
    client: StorageGrpcClient,
    cache: Arc<CacheLayer>,
    config: RemoteConfig,
}

impl RemoteProvider {
    // gRPC to remote storage service
    // Local cache with write-through/write-behind
    // Optimistic locking with version vectors
    // Supports: multi-region, read replicas
}
```

**Protocol:**
```protobuf
service Storage {
  rpc KvGet(KvGetRequest) returns (KvGetResponse);
  rpc KvSet(KvSetRequest) returns (KvSetResponse);
  rpc VectorSearch(VectorSearchRequest) returns (VectorSearchResponse);
  rpc TsQuery(TsQueryRequest) returns (stream TsPoint);
  rpc BlobGet(BlobGetRequest) returns (stream BlobChunk);
  rpc Transaction(stream TxnRequest) returns (stream TxnResponse);
}
```

---

## 4. Namespace & Isolation

```
voxy/
├── config/                 # App & plugin configs
│   ├── app.v1
│   └── plugins.{plugin_id}.v1
├── memory/
│   ├── episodic/           # Agent episodic memories
│   ├── semantic/           # Semantic knowledge
│   └── procedural/         # Skills, procedures
├── conversation/
│   ├── sessions/           # Per-session messages
│   └── summaries/          # Session summaries
├── plugins/
│   └── {plugin_id}/        # Plugin-isolated storage
│       ├── kv/
│       ├── vectors/
│       └── blobs/
├── metrics/
│   ├── counters/
│   ├── histograms/
│   └── traces/
├── models/
│   ├── embeddings/         # Cached embeddings
│   └── weights/            # Quantized model weights
├── audit/
│   ├── security/           # Security events
│   └── compliance/         # Compliance events
└── secrets/                # Encrypted secrets (provider-encrypted)
    └── {tenant_id}/
```

**Plugin Isolation:**
- Each plugin gets `plugins.{plugin_id}/` namespace
- Cannot access other plugin namespaces
- Quota enforced per-plugin (configurable)

---

## 5. Encryption

### 5.1 At-Rest Encryption

| Provider | Method |
|----------|--------|
| SQLite | SQLCipher (AES-256) or transparent page encryption |
| PostgreSQL | pg_tde or column-level pgcrypto |
| DuckDB | File-level encryption (AES-256-GCM) |
| Remote | TLS 1.3 + provider-side encryption |

### 5.2 Key Management

```rust
pub struct EncryptionConfig {
    pub master_key: KeySource,
    pub key_rotation: RotationPolicy,
    pub per_namespace_keys: bool,
}

pub enum KeySource {
    File(PathBuf),           // Encrypted key file
    EnvVar(String),          // Base64 encoded
    Vault(VaultConfig),      // HashiCorp Vault
    Tpm(TpmConfig),          // Hardware TPM
    Kms(KmsConfig),          // AWS KMS, Azure Key Vault, GCP KMS
}
```

### 5.3 Field-Level Encryption (Optional)

For extra-sensitive fields (API keys, tokens):

```rust
#[derive(Serialize, Deserialize)]
struct EncryptedField {
    #[serde(with = "encrypted_serde")]
    value: SecretValue,
    algorithm: EncryptionAlgorithm,
    key_id: KeyId,
}
```

---

## 6. Migrations

### 6.1 Migration System

```rust
pub struct Migration {
    pub version: u64,
    pub name: String,
    pub up: String,           // SQL or Rust function
    pub down: Option<String>, // Rollback
    pub provider: ProviderFilter, // Which providers this applies to
    pub requires_downtime: bool,
}

pub struct MigrationRunner {
    provider: Arc<dyn StorageProvider>,
    migrations: Vec<Migration>,
    lock: MigrationLock,
}
```

### 6.2 Version Tracking

```sql
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    checksum TEXT NOT NULL,
    execution_time_ms INTEGER,
    success BOOLEAN NOT NULL
);
```

### 6.3 Migration Rules

1. **Forward only in production** — down migrations tested in CI only
2. **Idempotent** — safe to re-run
3. **Backward compatible** — old code works with new schema
4. **Tested** — each migration has integration test
5. **Ordered** — strict version ordering

---

## 7. Backup & Restore

### 7.1 Backup Strategies

| Strategy | Frequency | Retention | Use Case |
|----------|-----------|-----------|----------|
| **Continuous** | WAL shipping | 7 days | Point-in-time recovery |
| **Snapshot** | Daily | 30 days | Full restore |
| **Incremental** | Hourly | 7 days | Fast recovery |
| **Archive** | Weekly | 1 year | Compliance |

### 7.2 Backup Destination

```rust
pub enum BackupDestination {
    Local(PathBuf),
    S3(S3Config),
    Gcs(GcsConfig),
    Azure(AzureConfig),
    RemoteGrpc(RemoteConfig),
}
```

### 7.3 Restore Process

1. Stop writers (drain connections)
2. Verify backup integrity (checksums)
3. Restore to new database
4. Run migrations if needed
5. Verify data integrity
6. Swap connections
7. Resume writers

---

## 8. Performance Targets

| Operation | Target (p99) | Target (p50) |
|-----------|--------------|--------------|
| KV Get (local) | < 5 ms | < 1 ms |
| KV Set (local) | < 10 ms | < 2 ms |
| Vector Search (100K vecs) | < 50 ms | < 10 ms |
| TS Append (batch 100) | < 20 ms | < 5 ms |
| Blob Get (1 MB) | < 100 ms | < 20 ms |
| Transaction (10 ops) | < 30 ms | < 10 ms |
| Backup (1 GB) | < 60 s | < 30 s |
| Restore (1 GB) | < 120 s | < 60 s |

---

## 9. Observability

### 9.1 Metrics

```prometheus
# Storage operations
voxy_storage_operations_total{provider, operation, status}
voxy_storage_operation_duration_seconds{provider, operation}
voxy_storage_active_connections{provider}
voxy_storage_pool_usage{provider}

# Data volumes
voxy_storage_kv_keys{provider, namespace}
voxy_storage_vectors{provider, namespace}
voxy_storage_blob_bytes{provider, namespace}
voxy_storage_ts_points{provider, namespace}

# Health
voxy_storage_health_status{provider}
voxy_storage_migration_status{provider, version}
voxy_storage_backup_status{provider, status}
```

### 9.2 Tracing

All operations traced with:
- `storage.provider`
- `storage.operation`
- `storage.namespace`
- `storage.duration_ms`
- `storage.rows_affected`

### 9.3 Slow Query Detection

```rust
const SLOW_QUERY_THRESHOLD_MS: u64 = 100;

async fn log_slow_query(operation: &str, duration: Duration, params: &str) {
    if duration.as_millis() > SLOW_QUERY_THRESHOLD_MS as u128 {
        tracing::warn!(
            operation = %operation,
            duration_ms = duration.as_millis(),
            params = %params,
            "Slow storage query detected"
        );
    }
}
```

---

## 10. Testing Strategy

### 10.1 Provider Conformance Tests

```rust
#[cfg(test)]
mod conformance_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_kv_crud() {
        let provider = create_test_provider().await;
        test_kv_crud(&provider).await;
    }
    
    #[tokio::test]
    async fn test_vector_search() {
        let provider = create_test_provider().await;
        test_vector_search(&provider).await;
    }
    
    // Run against ALL providers
    fn create_test_provider() -> Box<dyn StorageProvider> {
        match std::env::var("TEST_PROVIDER").as_deref() {
            Ok("sqlite") => Box::new(SqliteProvider::new_test().await.unwrap()),
            Ok("postgres") => Box::new(PostgresProvider::new_test().await.unwrap()),
            Ok("duckdb") => Box::new(DuckDbProvider::new_test().await.unwrap()),
            _ => Box::new(SqliteProvider::new_test().await.unwrap()),
        }
    }
}
```

### 10.2 Property-Based Tests

```rust
#[quickcheck]
fn kv_set_get_is_idempotent(key: String, value: Vec<u8>) -> bool {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let provider = SqliteProvider::new_test().await.unwrap();
        provider.kv_set("test", &key, &value, None).await.unwrap();
        let got = provider.kv_get("test", &key).await.unwrap();
        got == Some(value)
    })
}
```

### 10.3 Chaos Testing

- Random connection failures
- Disk full simulation
- Concurrent migration + operations
- Backup during heavy write load

---

## 11. Implementation Roadmap

### Phase 1: Core Trait + SQLite (Week 1)
- [ ] Define `StorageProvider` trait
- [ ] Implement `SqliteProvider`
- [ ] Basic KV, Vector, TS, Blob
- [ ] Migrations framework
- [ ] Unit + conformance tests

### Phase 2: Encryption + Backup (Week 2)
- [ ] SQLCipher integration
- [ ] Key management abstraction
- [ ] Backup/Restore API
- [ ] Local + S3 destinations

### Phase 3: Advanced Providers (Week 3)
- [ ] `PostgresProvider` (pgvector)
- [ ] `DuckDbProvider`
- [ ] `RemoteProvider` (gRPC)
- [ ] Provider registry/factory

### Phase 4: Production Hardening (Week 4)
- [ ] Connection pooling tuning
- [ ] Query optimization
- [ ] Slow query logging
- [ ] Chaos testing
- [ ] Performance benchmarks
- [ ] Documentation

---

## 12. Review Checklist

- [ ] Trait covers all access patterns
- [ ] SQLite implementation complete
- [ ] Vector search works with sqlite-vec
- [ ] Encryption at rest for all providers
- [ ] Migration system tested
- [ ] Backup/restore verified
- [ ] Plugin isolation enforced
- [ ] Performance targets measurable
- [ ] Observability comprehensive
- [ ] Conformance tests pass for all providers
- [ ] No direct SQLite dependencies in core crates

---

**Next Step**: Internal review → approve → implement Phase 1