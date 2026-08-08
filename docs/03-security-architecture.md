# VOXY Security Architecture Design

**Status**: DRAFT — For Internal Review
**Version**: 1.0
**Author**: VOXY Architecture Team
**Date**: 2026-07-24

---

## 1. Overview

The Security layer is the **immune system** of VOXY. It provides defense-in-depth for:

- **Consumer**: Single-user desktop assistant
- **Enterprise**: Multi-tenant, RBAC, compliance, audit
- **Distributed**: Remote nodes, mobile companions, robotics
- **Plugin Ecosystem**: Sandboxed, capability-based execution

### Design Principles

1. **Zero Trust**: Every request authenticated, authorized, audited
2. **Capability-Based**: No ambient authority; explicit grants only
3. **Defense in Depth**: Multiple independent security layers
4. **Audit First**: Everything logged, tamper-evident
5. **Fail Secure**: Default deny, graceful degradation
6. **Recoverable**: Guardian mode, recovery mode, zeroization
7. **Standards Compliant**: OAuth2, OIDC, FIDO2, SPIFFE, SIGSTORE
8. **Identity-First**: Every entity has a unique identity (user, device, agent, plugin, service, home, organization)
9. **Trust Scoring**: Every connected entity has a dynamic trust level
10. **Guardian Decision Flow**: Every action passes through a complete decision pipeline before execution

---

## 2. Core Components

### 2.1 Capability Manager

Central registry of all capabilities in the system.

```rust
pub struct CapabilityManager {
    registry: CapabilityRegistry,
    grants: GrantStore,
    policies: PolicyEngine,
    auditor: AuditLog,
}

impl CapabilityManager {
    // Capability lifecycle
    pub async fn register_capability(&self, cap: Capability) -> Result<()>;
    pub async fn revoke_capability(&self, id: &str) -> Result<()>;
    
    // Grant management
    pub async fn grant(&self, subject: &Subject, capability: &str, context: GrantContext) -> Result<Grant>;
    pub async fn revoke(&self, grant_id: &GrantId) -> Result<()>;
    pub async fn check(&self, subject: &Subject, capability: &str, context: &CheckContext) -> CheckResult;
    
    // Delegation
    pub async fn delegate(&self, from: &Subject, to: &Subject, capability: &str, constraints: DelegationConstraints) -> Result<Grant>;
}
```

### 2.1.1 Capability Risk Levels

Every capability is classified into one of five risk levels. Every tool, plugin, agent, and component MUST declare which capabilities it requires.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    /// No risk. No permission needed. Always allowed.
    /// Example: read system time, get version info
    None,
    
    /// Low risk. Minor information exposure.
    /// Auto-granted for trusted/verified entities.
    /// Requires user notification (non-blocking).
    /// Example: read clipboard, read files in allowed paths, read notifications
    Low,
    
    /// Medium risk. Potential for minor harm.
    /// Requires user consent (one-time or session).
    /// Notified via toast/notification.
    /// Example: launch applications, modify user files, browser automation
    Medium,
    
    /// High risk. Potential for significant harm.
    /// Requires explicit user consent each time.
    /// Shows modal consent dialog with full context.
    /// Gated by Guardian Policy Engine before user prompt.
    /// Example: delete files, registry changes, system settings, network configuration
    High,
    
    /// Critical risk. Potential for catastrophic harm.
    /// Requires multi-factor consent (password + biometric).
    /// Always logged with full context.
    /// Requires Guardian Policy Engine approval.
    /// Gated by Threat Analysis before execution.
    /// Auto-blocked for unknown/untrusted entities.
    /// Example: disk operations, BitLocker, Windows services, security policies,
    ///          home security, door locks, emergency actions
    Critical,
}
```

**Complete Capability Taxonomy by Risk Level:**

**NONE** (`RiskLevel::None`) — No permission needed:
| ID | Description |
|----|-------------|
| `system:info` | Read system information |
| `system:time` | Read system time |
| `system:version` | Read VOXY version |
| `app:basic` | Basic application functionality |
| `ui:focus` | Get focused window info |

**LOW** (`RiskLevel::Low`) — Auto-granted for trusted:
| ID | Category | Description |
|----|----------|-------------|
| `clipboard:read` | System | Read clipboard contents |
| `clipboard:write` | System | Write to clipboard |
| `file:read` | File | Read files in allowed paths |
| `notification:read` | System | Read notifications |
| `network:status` | Network | Check network status |
| `ui:list_windows` | UI | List open windows |
| `memory:self_read` | Memory | Read own process memory |

**MEDIUM** (`RiskLevel::Medium`) — Requires user consent:
| ID | Category | Description |
|----|----------|-------------|
| `app:launch` | System | Launch applications |
| `file:write` | File | Modify files |
| `file:download` | Network | Download files to disk |
| `browser:automation` | Automation | Browser control |
| `network:http` | Network | Outbound HTTP requests |
| `audio:loopback` | Audio | Capture system audio |
| `screen:metadata` | Screen | Get screen resolution/layout |
| `model:inference` | AI | Run model inference |

**HIGH** (`RiskLevel::High`) — Requires explicit consent + Guardian check:
| ID | Category | Description |
|----|----------|-------------|
| `file:delete` | File | Delete files |
| `file:permissions` | File | Change file permissions |
| `registry:read` | System | Read registry |
| `registry:write` | System | Modify registry |
| `system:settings` | System | Change system settings |
| `network:config_read` | Network | Read network configuration |
| `network:config_write` | Network | Change network configuration |
| `audio:capture` | Audio | Microphone access |
| `screen:capture` | Screen | Screenshot/recording |
| `automation:input` | Automation | Mouse/keyboard simulation |
| `process:list` | Process | List running processes |
| `process:info` | Process | Get process details |

**CRITICAL** (`RiskLevel::Critical`) — Requires MFA + Guardian + Threat Analysis:
| ID | Category | Description |
|----|----------|-------------|
| `disk:format` | Disk | Format drives |
| `disk:partition` | Disk | Modify partitions |
| `disk:bitlocker` | Security | BitLocker operations |
| `service:manage` | System | Start/stop Windows services |
| `security:policy` | Security | Modify security policies |
| `security:firewall` | Security | Change firewall rules |
| `process:spawn` | Process | Launch processes |
| `process:inject` | Process | Code injection |
| `process:kill` | Process | Kill processes |
| `plugin:load` | Plugin | Load plugins |
| `plugin:unload` | Plugin | Unload plugins |
| `agent:spawn` | Agent | Create agents |
| `agent:control` | Agent | Control other agents |
| `home:security` | Home | Home security system |
| `home:door_lock` | Home | Electronic door locks |
| `home:emergency` | Home | Emergency actions (alarm, fire) |
| `home:camera` | Home | Security camera access |
| `network:listen` | Network | Inbound network connections |
| `identity:impersonate` | Identity | Act as another entity |

### 2.2 Permission Manager

Evaluates access requests against grants and policies.

```rust
#[async_trait]
pub trait PermissionManager: Send + Sync {
    /// Check if subject has permission for action on resource
    async fn check_permission(&self, request: &PermissionRequest) -> PermissionDecision;
    
    /// Get all effective permissions for a subject
    async fn get_effective_permissions(&self, subject: &Subject) -> Vec<EffectivePermission>;
    
    /// Request user consent for elevated permission
    async fn request_consent(&self, request: &ConsentRequest) -> Result<ConsentDecision>;
}

pub struct PermissionRequest {
    pub subject: Subject,
    pub resource: Resource,
    pub action: Action,
    pub context: RequestContext,
}

pub enum PermissionDecision {
    Allow { reason: String, grants: Vec<GrantId> },
    Deny { reason: String, required: Vec<Capability> },
    RequireConsent { request: ConsentRequest },
    RequireMFA { methods: Vec<MfaMethod> },
}
```

### 2.3 Consent Manager

Handles user consent for sensitive operations.

```rust
pub struct ConsentManager {
    store: ConsentStore,
    notifier: ConsentNotifier,
    policy: ConsentPolicy,
}

impl ConsentManager {
    /// Request consent for a capability
    pub async fn request(&self, request: &ConsentRequest) -> Result<ConsentDecision>;
    
    /// Get consent status
    pub async fn status(&self, subject: &Subject, capability: &str) -> ConsentStatus;
    
    /// Revoke consent
    pub async fn revoke(&self, consent_id: &ConsentId) -> Result<()>;
    
    /// Auto-expire old consents
    pub async fn cleanup_expired(&self) -> Result<usize>;
}

pub struct ConsentRequest {
    pub capability: String,
    pub requester: Subject,
    pub context: ConsentContext,
    pub duration: Option<Duration>,
    pub requires_reauth: bool,
}

pub enum ConsentDecision {
    Granted { consent_id: ConsentId, expires_at: Option<DateTime<Utc>> },
    Denied { reason: String },
    Deferred { reason: String },
}
```

**Consent UI Flows**:
- **Modal**: Immediate blocking consent (e.g., microphone access)
- **Toast**: Non-blocking, dismissible (e.g., analytics)
- **Settings**: Persistent management page
- **CLI/API**: For headless/remote

### 2.4 Secret Vault

Secure storage for secrets with zeroization.

```rust
pub struct SecretVault {
    backend: VaultBackend,
    key_manager: KeyManager,
    audit: AuditLog,
}

#[async_trait]
pub trait VaultBackend: Send + Sync {
    async fn seal(&self, secret: &Secret) -> Result<SealedSecret>;
    async fn unseal(&self, sealed: &SealedSecret) -> Result<Secret>;
    async fn delete(&self, id: &SecretId) -> Result<()>;
    async fn list(&self, prefix: &str) -> Result<Vec<SecretMetadata>>;
    async fn rotate_key(&self) -> Result<()>;
}

pub struct Secret {
    pub id: SecretId,
    pub namespace: String,
    pub value: SecretValue,
    pub metadata: SecretMetadata,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub version: u32,
}

pub enum SecretValue {
    String(String),
    Bytes(Vec<u8>),
    KeyPair { private: Vec<u8>, public: Vec<u8> },
    Certificate { cert: Vec<u8>, chain: Vec<Vec<u8>> },
}

impl SecretValue {
    /// Zeroize on drop
    fn zeroize(&mut self);
}
```

**Vault Backends**:
| Backend | Use Case | Encryption |
|---------|----------|------------|
| `memory` | Testing, ephemeral | AES-256-GCM |
| `file` | Single-user desktop | AES-256-GCM + Argon2id |
| `sqlite` | Embedded, encrypted DB | SQLCipher |
| `hashicorp` | Enterprise | Vault Transit |
| `aws_kms` | Cloud | KMS |
| `azure_keyvault` | Cloud | Key Vault |
| `tpm` | Hardware-backed | TPM 2.0 |

### 2.5 Token Manager

Manages capability tokens (JWT-like, but capability-based).

```rust
pub struct TokenManager {
    issuer: TokenIssuer,
    validator: TokenValidator,
    store: TokenStore,
    revocation: RevocationList,
}

impl TokenManager {
    /// Issue a capability token
    pub async fn issue(&self, request: &TokenIssueRequest) -> Result<CapabilityToken>;
    
    /// Validate and extract claims
    pub async fn validate(&self, token: &str) -> Result<TokenClaims>;
    
    /// Refresh token (if renewable)
    pub async fn refresh(&self, token: &str) -> Result<CapabilityToken>;
    
    /// Revoke token
    pub async fn revoke(&self, token_id: &TokenId, reason: RevocationReason) -> Result<()>;
    
    /// Introspect token
    pub async fn introspect(&self, token: &str) -> Result<TokenIntrospection>;
}

pub struct CapabilityToken {
    pub token: String,           // Opaque or JWT
    pub claims: TokenClaims,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct TokenClaims {
    pub sub: Subject,                    // Who
    pub caps: Vec<Capability>,           // What
    pub scope: TokenScope,               // Where
    pub constraints: TokenConstraints,   // Conditions
    pub nonce: String,                   // Replay protection
}

pub struct TokenConstraints {
    pub max_uses: Option<u32>,
    pub allowed_origins: Vec<String>,
    pub required_mfa: bool,
    pub ip_whitelist: Vec<IpNetwork>,
    pub time_window: Option<TimeWindow>,
}
```

### 2.6 Policy Engine

OPA-compatible policy evaluation for complex authorization.

```rust
pub struct PolicyEngine {
    store: PolicyStore,
    evaluator: RegoEvaluator,
    cache: PolicyCache,
}

impl PolicyEngine {
    /// Evaluate policy for request
    pub async fn evaluate(&self, input: &PolicyInput) -> PolicyDecision;
    
    /// Load policy bundle
    pub async fn load_policy(&self, bundle: PolicyBundle) -> Result<()>;
    
    /// Test policy with input
    pub async fn test(&self, policy: &str, input: &Value) -> Result<Value>;
}

pub struct PolicyInput {
    pub subject: Subject,
    pub resource: Resource,
    pub action: Action,
    pub context: RequestContext,
    pub environment: EnvironmentContext,
}

pub struct PolicyDecision {
    pub allow: bool,
    pub obligations: Vec<Obligation>,
    pub advice: Vec<Advice>,
    pub explanation: Option<String>,
}
```

**Policy Examples** (Rego):

```rego
# Enterprise: Only admins can spawn agents with network access
allow {
    input.subject.role == "admin"
    input.action == "agent:spawn"
    input.resource.capabilities[_] == "network:http"
}

# Consumer: Require consent for screen capture
allow {
    input.action == "screen:capture"
    consent_granted(input.subject, "screen:capture")
}

# Time-based: No automation after hours
deny {
    input.action == "automation:input"
    not business_hours(now())
}

# Rate limiting
deny {
    count_requests(input.subject, 1min) > 100
}
```

### 2.7 Integrity Verification

Ensures system integrity at boot and runtime.

```rust
pub struct IntegrityVerifier {
    measurements: MeasurementStore,
    policy: IntegrityPolicy,
    notifier: AlertNotifier,
}

impl IntegrityVerifier {
    /// Verify boot integrity (measured boot)
    pub async fn verify_boot(&self) -> Result<BootIntegrityReport>;
    
    /// Verify runtime integrity (files, memory, config)
    pub async fn verify_runtime(&self) -> Result<RuntimeIntegrityReport>;
    
    /// Verify component signature
    pub async fn verify_component(&self, component: &Component) -> Result<VerificationResult>;
    
    /// Attest to remote party
    pub async fn attest(&self, challenge: &[u8]) -> Result<Attestation>;
}

pub struct BootIntegrityReport {
    pub pcr_values: HashMap<u32, Vec<u8>>,    // TPM PCRs
    pub event_log: Vec<BootEvent>,
    pub verified: bool,
    pub anomalies: Vec<Anomaly>,
}

pub struct RuntimeIntegrityReport {
    pub file_integrity: FileIntegrityResult,
    pub memory_integrity: MemoryIntegrityResult,
    pub config_integrity: ConfigIntegrityResult,
    pub process_integrity: ProcessIntegrityResult,
    pub verified: bool,
}
```

### 2.8 Audit Log

Tamper-evident audit trail for all security events.

```rust
pub struct AuditLog {
    backend: AuditBackend,
    signer: LogSigner,
    indexer: LogIndexer,
}

#[async_trait]
pub trait AuditBackend: Send + Sync {
    async fn append(&self, entry: &AuditEntry) -> Result<EntryId>;
    async fn query(&self, query: &AuditQuery) -> Result<Vec<AuditEntry>>;
    async fn verify_chain(&self, from: EntryId, to: EntryId) -> Result<VerificationResult>;
    async fn export(&self, format: ExportFormat, query: &AuditQuery) -> Result<Vec<u8>>;
}

pub struct AuditEntry {
    pub id: EntryId,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub subject: Subject,
    pub resource: Option<Resource>,
    pub action: String,
    pub result: AuditResult,
    pub details: Value,
    pub risk_level: RiskLevel,
    pub correlation_id: Option<CorrelationId>,
    pub signature: Signature,  // Hash chain + signature
}

pub enum AuditEventType {
    Authentication { method: AuthMethod, success: bool },
    Authorization { decision: PermissionDecision },
    Consent { capability: String, granted: bool },
    SecretAccess { secret_id: SecretId, operation: SecretOperation },
    PolicyChange { policy_id: PolicyId, change: PolicyChange },
    ConfigurationChange { config: String, old: Value, new: Value },
    ComponentLoad { component: ComponentId, hash: Hash },
    ComponentUnload { component: ComponentId },
    IntegrityCheck { result: IntegrityResult },
    ThreatDetected { threat: ThreatEvent },
    GuardianModeActivated { reason: String },
    RecoveryModeActivated { reason: String },
    Zeroization { reason: String },
}
```

**Storage**: Append-only, hash-chained (Merkle tree), signed periodically.

### 2.9 Threat Detection

Real-time anomaly detection.

```rust
pub struct ThreatDetector {
    rules: DetectionRules,
    ml_engine: Option<MLEngine>,
    alerting: AlertManager,
    state: ThreatState,
}

impl ThreatDetector {
    pub async fn analyze_event(&self, event: &AuditEntry) -> Vec<ThreatSignal>;
    pub async fn analyze_session(&self, session: &Session) -> Vec<ThreatSignal>;
    pub async fn get_risk_score(&self, subject: &Subject) -> RiskScore;
}

pub struct ThreatSignal {
    pub rule_id: String,
    pub severity: Severity,
    pub description: String,
    pub indicators: Vec<Indicator>,
    pub recommended_action: RecommendedAction,
}

pub enum RecommendedAction {
    LogOnly,
    RequireMFA,
    RequireConsent,
    RateLimit,
    Block,
    Quarantine,
    AlertAdmin,
}
```

**Built-in Rules**:
- Brute force detection
- Impossible travel
- Privilege escalation attempts
- Data exfiltration patterns
- Plugin behavior anomalies
- Resource exhaustion

### 2.10 Guardian Mode

Emergency lockdown mode.

```rust
pub struct GuardianMode {
    state: GuardianState,
    triggers: GuardianTriggers,
    policies: GuardianPolicies,
    notifier: GuardianNotifier,
}

impl GuardianMode {
    pub async fn activate(&self, reason: &str, triggered_by: &Subject) -> Result<()>;
    pub async fn deactivate(&self, authorized_by: &Subject) -> Result<()>;
    pub async fn status(&self) -> GuardianStatus;
}

pub struct GuardianPolicies {
    pub block_all_network: bool,
    pub block_all_file_write: bool,
    pub block_plugin_load: bool,
    pub block_agent_spawn: bool,
    pub allow_read_only: bool,
    pub require_hardware_auth: bool,
    pub notify_contacts: Vec<Contact>,
}

pub enum GuardianTrigger {
    Manual { authorized_by: Subject },
    IntegrityFailure { details: IntegrityFailure },
    ThreatDetected { threat: ThreatEvent },
    ConsentViolation { details: ConsentViolation },
    HardwareTrigger { source: HardwareSource },
    Scheduled { schedule: Schedule },
}
```

### 2.11 Recovery Mode

Secure recovery from compromise.

```rust
pub struct RecoveryMode {
    vault: SecretVault,
    policy_engine: PolicyEngine,
    auditor: AuditLog,
    notifier: RecoveryNotifier,
}

impl RecoveryMode {
    /// Enter recovery mode (requires hardware auth or multi-party)
    pub async fn enter(&self, auth: RecoveryAuth) -> Result<RecoverySession>;
    
    /// Rotate all secrets
    pub async fn rotate_all_secrets(&self, session: &RecoverySession) -> Result<RotationReport>;
    
    /// Revoke all tokens
    pub async fn revoke_all_tokens(&self, session: &RecoverySession) -> Result<RevocationReport>;
    
    /// Re-verify all components
    pub async fn verify_all_components(&self, session: &RecoverySession) -> Result<VerificationReport>;
    
    /// Generate recovery report
    pub async fn generate_report(&self, session: &RecoverySession) -> Result<RecoveryReport>;
    
    /// Exit recovery mode
    pub async fn exit(&self, session: &RecoverySession, authorized_by: &Subject) -> Result<()>;
}
```

### 2.12 Secure Shutdown

Ordered, audited shutdown with zeroization.

```rust
pub struct SecureShutdown {
    sequence: ShutdownSequence,
    zeroizer: MemoryZeroizer,
    auditor: AuditLog,
}

impl SecureShutdown {
    pub async fn initiate(&self, reason: ShutdownReason) -> Result<ShutdownReport>;
    
    async fn zeroize_secrets(&self) -> Result<()>;
    async fn zeroize_memory(&self) -> Result<()>;
    async fn flush_audit_log(&self) -> Result<()>;
    async fn seal_vault(&self) -> Result<()>;
    async fn revoke_tokens(&self) -> Result<()>;
}
```

### 2.14 Guardian Policy Engine

The Guardian is the **central decision engine** for every security-relevant action. It is NOT just a permission checker — it is a multi-stage pipeline that evaluates risk, threat, context, and trust before any action executes.

#### 2.14.1 Guardian Decision Flow

```
Request
  │
  ▼
┌─────────────┐
│  1. Capability Check  │  ← Does the subject have the required capability?
└──────┬──────┘
       │ FAIL → DENY + AUDIT
       ▼ PASS
┌─────────────┐
│  2. Policy Check      │  ← Does any policy allow/deny this?
└──────┬──────┘         │  ← OPA/Rego: RBAC, ABAC, time-based, location, rate limits
       │ DENY → DENY + REASON + AUDIT
       ▼ PASS
┌─────────────┐
│  3. User Consent      │  ← Does risk level require user consent?
└──────┬──────┘         │  ← LOW = auto, MEDIUM = toast, HIGH = modal, CRITICAL = MFA
       │ DENY → DENY + AUDIT
       ▼ PASS / N/A
┌─────────────┐
│  4. Risk Analysis     │  ← Calculate dynamic risk score
└──────┬──────┘         │  ← Trust level of subject + sensitivity of action +
       │                 │     current context + historical behavior
       │ HIGH → Escalate to Guardian Mode
       ▼ PASS
┌─────────────┐
│  5. Threat Analysis   │  ← Real-time threat detection
└──────┬──────┘         │  ← Anomaly patterns, brute force, privilege escalation
       │ THREAT → BLOCK + GUARDIAN MODE + AUDIT
       ▼ CLEAR
┌─────────────┐
│  6. Execute           │  ← Action is executed with full context
└──────┬──────┘
       ▼
┌─────────────┐
│  7. Verify            │  ← Post-execution verification
└──────┬──────┘         │  ← Did the action succeed as expected?
       │                 │  ← Integrity check
       │ FAIL → ROLLBACK + AUDIT + ALERT
       ▼ PASS
┌─────────────┐
│  8. Audit             │  ← Full audit log entry
└─────────────┘         │  ← Action, result, context, risk score, trust level
                         │  ← AI Action Journal entry (if AI-driven)
```

#### 2.14.2 Guardian Implementation

```rust
pub struct GuardianEngine {
    capability_checker: Arc<CapabilityManager>,
    policy_engine: Arc<PolicyEngine>,
    consent_manager: Arc<ConsentManager>,
    risk_analyzer: Arc<RiskAnalyzer>,
    threat_detector: Arc<ThreatDetector>,
    audit_log: Arc<AuditLog>,
    ai_journal: Arc<AiActionJournal>,
    rollback_manager: Arc<RollbackManager>,
}

impl GuardianEngine {
    pub async fn evaluate(&self, request: &GuardianRequest) -> Result<GuardianDecision> {
        // 1. Capability Check
        let cap_result = self.capability_checker.check(
            &request.subject, &request.capability, &request.context
        ).await?;
        if !cap_result.granted {
            self.audit_log.log(&AuditEntry::denied(request, "capability")).await?;
            return Ok(GuardianDecision::Denied { reason: cap_result.reason });
        }

        // 2. Policy Check
        let policy_input = PolicyInput::from(request);
        let policy_result = self.policy_engine.evaluate(&policy_input).await?;
        if !policy_result.allow {
            self.audit_log.log(&AuditEntry::denied(request, "policy")).await?;
            return Ok(GuardianDecision::Denied { reason: policy_result.explanation });
        }

        // 3. User Consent (based on risk level)
        let risk_level = request.capability.risk_level();
        if risk_level >= RiskLevel::Medium {
            let consent = self.consent_manager.request(&ConsentRequest {
                capability: request.capability.id.clone(),
                requester: request.subject.clone(),
                context: request.context.clone(),
                duration: Some(Duration::from_secs(3600)),
                requires_reauth: risk_level >= RiskLevel::Critical,
            }).await?;
            if matches!(consent, ConsentDecision::Denied(_)) {
                self.audit_log.log(&AuditEntry::consent_denied(request)).await?;
                return Ok(GuardianDecision::Denied { reason: "Consent denied".into() });
            }
        }

        // 4. Risk Analysis
        let risk_score = self.risk_analyzer.analyze(&RiskInput {
            subject: &request.subject,
            capability: &request.capability,
            trust_level: request.subject.trust_level(),
            context: &request.context,
            history: self.audit_log.get_history(&request.subject).await?,
        }).await?;
        if risk_score > RiskThreshold::Critical {
            self.guardian_mode.activate("Risk threshold exceeded", &request.subject).await?;
            return Ok(GuardianDecision::Denied { reason: "Risk too high".into() });
        }

        // 5. Threat Analysis
        let threats = self.threat_detector.analyze_event(&AuditEntry::from(request)).await?;
        if !threats.is_empty() {
            let critical = threats.iter().any(|t| t.severity >= Severity::High);
            if critical {
                self.guardian_mode.activate("Threat detected", &request.subject).await?;
                return Ok(GuardianDecision::Denied { reason: "Threat blocked".into() });
            }
        }

        // 6. Execute
        let execution_id = Uuid::new_v4();
        let result = request.execute().await;
        
        // 7. Verify
        let verified = self.verify_execution(&request, &result).await?;
        if !verified {
            self.rollback_manager.rollback(execution_id).await?;
            self.audit_log.log(&AuditEntry::execution_failed(request, execution_id)).await?;
            return Ok(GuardianDecision::Failed { execution_id, reason: "Verification failed".into() });
        }

        // 8. Audit + AI Journal
        self.audit_log.log(&AuditEntry::allowed(request, execution_id)).await?;
        if request.is_ai_driven {
            self.ai_journal.record(AiAction::from(request, result)).await?;
        }

        Ok(GuardianDecision::Allowed { execution_id })
    }
}

pub struct GuardianRequest {
    pub subject: Subject,
    pub capability: Capability,
    pub action: Box<dyn ExecutableAction>,
    pub context: RequestContext,
    pub is_ai_driven: bool,
    pub rollback_strategy: Option<RollbackStrategy>,
}

pub enum GuardianDecision {
    Allowed { execution_id: Uuid },
    Denied { reason: String },
    Failed { execution_id: Uuid, reason: String },
}
```

#### 2.14.3 Risk Analyzer

```rust
pub struct RiskAnalyzer {
    base_risk: HashMap<CapabilityId, RiskLevel>,
    trust_modifier: TrustModifier,
    context_modifier: ContextModifier,
    history_modifier: HistoryModifier,
}

impl RiskAnalyzer {
    /// Calculate dynamic risk score for an action.
    /// Formula: base_risk × trust_modifier × context_modifier × history_modifier
    pub async fn analyze(&self, input: &RiskInput) -> RiskScore {
        let mut score = self.base_risk[&input.capability.id].base_score();
        
        // Trust modifier: untrusted subjects increase risk
        score *= self.trust_modifier.factor(input.trust_level);
        
        // Context modifier: sensitive contexts increase risk
        score *= self.context_modifier.factor(&input.context);
        
        // History modifier: frequent failures increase risk
        score *= self.history_modifier.factor(&input.history);
        
        score
    }
}
```

---

### 2.15 Trust Levels

Every connected entity (device, agent, plugin, remote node) has a **dynamic trust score** that evolves over time.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrustLevel {
    /// Verified by hardware attestation (TPM/Secure Enclave).
    /// Full access to capabilities up to HIGH.
    /// CRITICAL requires additional consent.
    Verified,
    
    /// Known entity with positive history.
    /// Auto-granted LOW capabilities.
    /// MEDIUM requires consent. HIGH blocked.
    Trusted,
    
    /// Previously seen but not trusted.
    /// LOW auto-granted. MEDIUM requires consent.
    /// HIGH and CRITICAL blocked.
    Known,
    
    /// First-time or unrecognized entity.
    /// LOW requires consent. MEDIUM+ blocked.
    Unknown,
    
    /// Explicitly blocked by user or policy.
    /// All capabilities denied.
    Blocked,
    
    /// Security incident detected on this entity.
    /// All capabilities blocked. Guardian Mode triggered.
    /// Requires recovery to clear.
    Compromised,
}

impl TrustLevel {
    /// What's the maximum capability risk level this trust level allows?
    pub fn max_allowed_risk(&self) -> RiskLevel {
        match self {
            Self::Verified => RiskLevel::High,      // Critical needs extra
            Self::Trusted => RiskLevel::Medium,      // High needs consent
            Self::Known => RiskLevel::Low,           // Medium blocked
            Self::Unknown => RiskLevel::None,        // Low blocked
            Self::Blocked | Self::Compromised => RiskLevel::None,
        }
    }
    
    /// Can this level auto-grant a capability without user prompt?
    pub fn auto_grants(&self, risk: RiskLevel) -> bool {
        match self {
            Self::Verified => risk <= RiskLevel::Low,
            Self::Trusted => risk <= RiskLevel::Low,
            Self::Known => risk == RiskLevel::None,
            _ => false,
        }
    }
}
```

**Trust Evolution**:

```rust
pub struct TrustManager {
    store: TrustStore,
    auditor: AuditLog,
}

impl TrustManager {
    /// Record a positive interaction (increases trust)
    pub async fn report_positive(&self, subject: &SubjectId, action: &str) -> Result<()>;
    
    /// Record a negative interaction (decreases trust)
    pub async fn report_negative(&self, subject: &SubjectId, action: &str) -> Result<()>;
    
    /// Calculate current trust level from history
    pub async fn calculate_trust(&self, subject: &SubjectId) -> TrustLevel;
    
    /// Escalate to Guardan mode on compromise
    pub async fn report_compromise(&self, subject: &SubjectId, threat: &ThreatEvent) -> Result<()>;
}
```

| Trust Level | Default Max Risk | Auto-Grant | Escalation Path |
|-------------|------------------|------------|-----------------|
| Verified | HIGH | LOW | → Trusted (if inactive) |
| Trusted | MEDIUM | LOW | → Known (if 3+ failures) |
| Known | LOW | NONE | → Blocked (if 5+ failures) |
| Unknown | NONE | NONE | → Known (if 1+ positive) |
| Blocked | NONE | NONE | → Manually unblocked |
| Compromised | NONE | NONE | → Recovery required |

---

### 2.16 Identity Layer

Every entity in the VOXY ecosystem has a **unified identity model**. This is designed for future expansion to homes, organizations, and distributed systems.

```rust
/// Unique identifier for any VOXY entity.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct IdentityId(Uuid);

/// Entity type discriminator.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// Human user (local or remote)
    User,
    /// Physical device (desktop, laptop, phone, server, robot)
    Device,
    /// AI agent running within VOXY
    Agent,
    /// Installed plugin or extension
    Plugin,
    /// System service or daemon component
    Service,
    /// Home/space (physical location with devices)
    Home,
    /// Organization/company (multi-user, multi-device)
    Organization,
}

/// Identity of any entity in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: IdentityId,
    pub entity_type: EntityType,
    pub name: String,
    pub display_name: Option<String>,
    
    // Hierarchy
    pub parent: Option<IdentityId>,         // User belongs to Organization, Device belongs to Home
    pub children: Vec<IdentityId>,           // Reverse lookup
    
    // Trust
    pub trust_level: TrustLevel,
    pub verified_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    
    // Crypto
    pub public_key: Option<Vec<u8>>,         // Ed25519 or ECDSA
    pub attestation: Option<Attestation>,    // TPM/SE attestation statement
    
    // Metadata
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Identity {
    pub fn is_human(&self) -> bool { self.entity_type == EntityType::User }
    pub fn is_device(&self) -> bool { self.entity_type == EntityType::Device }
    pub fn is_agent(&self) -> bool { self.entity_type == EntityType::Agent }
    
    /// Check if this identity is authorized to act on behalf of parent
    pub fn acts_for(&self, target: &Identity) -> bool {
        self.parent == Some(target.id) || target.children.contains(&self.id)
    }
}

/// Identity registry and management.
pub struct IdentityManager {
    store: IdentityStore,
    attestation: AttestationVerifier,
    auditor: AuditLog,
}

impl IdentityManager {
    // CRUD
    pub async fn register(&self, identity: &Identity) -> Result<()>;
    pub async fn get(&self, id: &IdentityId) -> Result<Option<Identity>>;
    pub async fn update(&self, identity: &Identity) -> Result<()>;
    pub async fn deactivate(&self, id: &IdentityId, reason: &str) -> Result<()>;
    
    // Discovery
    pub async fn find_by_type(&self, entity_type: EntityType) -> Result<Vec<Identity>>;
    pub async fn find_by_trust(&self, min_trust: TrustLevel) -> Result<Vec<Identity>>;
    pub async fn find_children(&self, parent: &IdentityId) -> Result<Vec<Identity>>;
    
    // Hierarchy
    pub async fn set_parent(&self, child: &IdentityId, parent: &IdentityId) -> Result<()>;
    pub async fn get_hierarchy(&self, id: &IdentityId) -> Result<IdentityHierarchy>;
    
    // Authentication
    pub async fn authenticate(&self, id: &IdentityId, challenge: &[u8]) -> Result<AuthResult>;
    pub async fn attest_device(&self, id: &IdentityId, attestation: &Attestation) -> Result<()>;
}
```

**Identity Hierarchy Examples**:

```
Consumer:
  Organization: "Gaurav's Home"
    ├── Home: "My Home"
    │   ├── Device: "Desktop-PC" (TrustLevel::Verified)
    │   ├── Device: "Laptop" (TrustLevel::Trusted)
    │   ├── Device: "Phone" (TrustLevel::Trusted)
    │   ├── Agent: "VoiceAssistant" (TrustLevel::Verified)
    │   └── Agent: "AutomationBot" (TrustLevel::Trusted)
    ├── Plugin: "Spotify" (TrustLevel::Trusted)
    ├── Plugin: "HomeAssistant" (TrustLevel::Verified)
    └── User: "Gaurav" (TrustLevel::Verified)

Enterprise:
  Organization: "Acme Corp"
    ├── Organization: "Engineering"
    │   ├── User: "Alice" (TrustLevel::Verified)
    │   ├── User: "Bob" (TrustLevel::Trusted)
    │   └── Device: "Alice-MacBook" (TrustLevel::Verified)
    ├── Organization: "Security"
    │   ├── User: "Carol" (TrustLevel::Verified) [admin]
    │   └── Service: "AuditLogger" (TrustLevel::Verified)
    ├── Home: "Floor 3 Conference Room"
    │   ├── Device: "Room-Speaker" (TrustLevel::Trusted)
    │   └── Device: "Room-Camera" (TrustLevel::Verified)
    └── Service: "AuthProxy" (TrustLevel::Verified)
```

---

### 2.17 AI Action Journal

Every AI-driven action (agent decision, model inference, automation task) is recorded in an immutable journal. This is critical for debugging, auditing, and compliance.

```rust
pub struct AiActionJournal {
    store: JournalStore,
    auditor: AuditLog,
}

impl AiActionJournal {
    /// Record an AI decision with full context
    pub async fn record(&self, action: AiAction) -> Result<JournalEntryId>;
    
    /// Query actions by criteria
    pub async fn query(&self, query: &JournalQuery) -> Result<Vec<AiAction>>;
    
    /// Get the full decision trace for a given action
    pub async fn get_trace(&self, id: &JournalEntryId) -> Result<DecisionTrace>;
    
    /// Export journal for compliance/audit
    pub async fn export(&self, format: ExportFormat, range: TimeRange) -> Result<Vec<u8>>;
    
    /// Replay actions from journal (for recovery)
    pub async fn replay(&self, ids: &[JournalEntryId]) -> Result<ReplayReport>;
}

pub struct AiAction {
    pub id: JournalEntryId,
    pub timestamp: DateTime<Utc>,
    
    // Decision
    pub reason: String,           // Why was this action taken?
    pub context: ActionContext,   // What was the state when decided?
    
    // Model
    pub model_id: String,         // Which model made the decision?
    pub model_version: String,    // Model version
    pub confidence: f64,          // Model confidence score (0.0 - 1.0)
    
    // Tool/Action
    pub tool: String,             // Which tool/function was called?
    pub tool_input: Value,        // Input parameters
    pub tool_output: Value,       // Raw output
    
    // Result
    pub result: ActionResult,     // Success, failure, partial
    pub error: Option<String>,
    pub execution_time_ms: u64,
    
    // Rollback
    pub rollback_strategy: Option<RollbackStrategy>,
    pub rollback_status: Option<RollbackStatus>,
    pub rollback_executed_at: Option<DateTime<Utc>>,
    
    // Trace
    pub trace_id: TraceId,
    pub parent_journal_id: Option<JournalEntryId>,
    
    // Security
    pub guardian_decision_id: Option<Uuid>,
    pub risk_score: Option<RiskScore>,
}

pub enum ActionResult {
    Success,
    Partial { completed: Vec<String>, failed: Vec<String> },
    Failed { reason: String },
    RolledBack { original: String, restored: String },
}

pub struct DecisionTrace {
    pub action: AiAction,
    pub guardian_log: Vec<GuardianLogEntry>,
    pub policy_results: Vec<PolicyDecision>,
    pub model_logprobs: Option<Vec<f64>>,
    pub alternative_actions: Vec<AlternativeAction>,
}

pub enum ExportFormat {
    Json,          // Human-readable, portable
    Jsonl,         // Line-delimited for streaming
    Csv,           // Spreadsheet-friendly
    Parquet,       // Analytics
    AuditBundle,   // Signed, encrypted for compliance
}
```

**Journal Entry Structure** (stored in `audit.ai_journal` namespace):

```json
{
  "id": "jour_abc123",
  "timestamp": "2026-07-24T12:00:00Z",
  "reason": "User asked to summarize today's emails",
  "context": {
    "conversation_id": "conv_xyz",
    "message_count": 42,
    "last_model_interaction": "2026-07-24T11:55:00Z"
  },
  "model_id": "claude-4-opus",
  "model_version": "2026-07-01",
  "confidence": 0.92,
  "tool": "voxy.memory.search",
  "tool_input": {"query": "today's emails summary", "top_k": 5},
  "tool_output": {"results": ["..."], "total": 12},
  "result": "Success",
  "execution_time_ms": 450,
  "rollback_strategy": null,
  "trace_id": "trace_def456",
  "parent_journal_id": null,
  "guardian_decision_id": "gd_789",
  "risk_score": 0.15
}
```

---

### 2.18 Rollback Framework

Every destructive action (or any action with a `RollbackStrategy`) can be rolled back. The framework ensures that actions are reversible.

```rust
pub struct RollbackManager {
    store: RollbackStore,
    executor: RollbackExecutor,
    auditor: AuditLog,
}

impl RollbackManager {
    /// Register a rollback strategy before executing an action
    pub async fn prepare(&self, action_id: Uuid, strategy: RollbackStrategy) -> Result<()>;
    
    /// Execute rollback for a given action
    pub async fn rollback(&self, action_id: Uuid) -> Result<RollbackResult>;
    
    /// Bulk rollback all actions after a point (crash recovery)
    pub async fn rollback_since(&self, since: DateTime<Utc>, reason: &str) -> Result<Vec<RollbackResult>>;
    
    /// Check if an action can be rolled back
    pub async fn can_rollback(&self, action_id: Uuid) -> bool;
    
    /// Get rollback status for an action
    pub async fn status(&self, action_id: Uuid) -> Option<RollbackStatus>;
}

pub enum RollbackStrategy {
    /// Before: copy file to backup location. After: restore from backup.
    FileBackup {
        original_path: PathBuf,
        backup_path: PathBuf,
    },
    /// Before: snapshot registry key. After: restore snapshot.
    RegistrySnapshot {
        key_path: String,
        snapshot: Vec<u8>,
    },
    /// Before: save current config. After: restore config.
    ConfigSnapshot {
        component: String,
        previous_value: Value,
    },
    /// Before: record DB state. After: undo transaction.
    DatabaseTransaction {
        transaction_id: Uuid,
        undo_log: Vec<UndoEntry>,
    },
    /// Before: create system restore point. After: invoke restore.
    SystemRestorePoint {
        point_name: String,
    },
    /// Custom: user-provided undo function.
    Custom {
        name: String,
        undo: Box<dyn UndoFunction>,
    },
    /// No rollback possible (logging only).
    None { reason: String },
}

pub enum RollbackStatus {
    /// Rollback available
    Available,
    /// Rollback in progress
    InProgress,
    /// Rollback completed successfully
    Completed,
    /// Rollback failed
    Failed { error: String },
    /// Expired (rollback window passed)
    Expired,
}

pub struct RollbackResult {
    pub action_id: Uuid,
    pub success: bool,
    pub strategy: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub error: Option<String>,
    pub restored_entities: Vec<String>,
}
```

**Rollback Flow**:

```
Action Requested (e.g., Delete File)
  │
  ▼
Prepare Rollback Strategy
  │  ┌─ FileBackup: copy file → backup_path
  │  └─ Record snapshot metadata
  │
  ▼
Execute Action (delete file)
  │
  ▼
Verify Action
  │  ✅ → Mark rollback as Available
  │  ❌ → Auto-rollback
  │
  ▼
[Later: User or Recovery requests rollback]
  │
  ▼
Execute Rollback
  │  ┌─ Restore file from backup_path
  │  └─ Verify integrity
  │
  ▼
Audit Rollback Result
```

**Standard Rollback Strategies by Capability**:

| Capability | Rollback Strategy | Retention |
|------------|-------------------|-----------|
| `file:delete` | FileBackup | 30 days |
| `file:write` | FileBackup (before overwrite) | 7 days |
| `registry:write` | RegistrySnapshot | 30 days |
| `config:set` | ConfigSnapshot | Indefinite |
| `app:launch` | None (only log) | — |
| `network:config_write` | ConfigSnapshot | 30 days |
| `database:*` | DatabaseTransaction | Transaction scope |
| `system:settings` | ConfigSnapshot | 30 days |
| `disk:format` | SystemRestorePoint | 90 days |
| `plugin:unload` | Plugin backup + metadata | 7 days |
| `agent:kill` | Agent state snapshot | 24 hours |

### 3.1 Consumer (Single User)

```
┌──────────────────────────────────────┐
│           VOXY Desktop                │
├──────────────────────────────────────┤
│  Identity Manager (local)             │
│  Trust Manager (local)                │
│  Guardian Engine (local)              │
│  Capability Manager (local)           │
│  Permission Manager (local)           │
│  Consent Manager (UI prompts)         │
│  Risk Analyzer (local)                │
│  Secret Vault (file, encrypted)       │
│  Token Manager (local)                │
│  Policy Engine (built-in rules)       │
│  Integrity Verifier (TPM if avail)    │
│  Audit Log (local, signed)            │
│  AI Action Journal (local)            │
│  Rollback Manager (local)             │
│  Threat Detector (basic rules)        │
│  Guardian Mode (manual + auto)        │
│  Recovery Mode (hardware auth)        │
│  Home Automation Interfaces (future)  │
└──────────────────────────────────────┘
```

### 3.2 Enterprise (Multi-Tenant)

```
┌──────────────────────────────────────┐
│         VOXY Enterprise Cluster       │
├──────────────────────────────────────┤
│  Identity Manager (LDAP/OIDC sync)    │
│  Trust Manager (centralized)          │
│  Guardian Engine (distributed)        │
│  Capability Manager (centralized)     │
│  Permission Manager (OPA cluster)     │
│  Consent Manager (delegated)          │
│  Risk Analyzer (ML-based)             │
│  Secret Vault (HashiCorp Vault)       │
│  Token Manager (SPIFFE/SPIRE)         │
│  Policy Engine (OPA + Bundles)        │
│  Integrity Verifier (Remote Attest)   │
│  Audit Log (Centralized, Immutable)   │
│  AI Action Journal (centralized)      │
│  Rollback Manager (enterprise)        │
│  Threat Detector (ML + Rules)         │
│  Guardian Mode (SOC Integration)      │
│  Recovery Mode (Multi-Party Auth)     │
│  Home Automation Interfaces (future)  │
└──────────────────────────────────────┘
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
    ┌────────┐  ┌────────┐  ┌────────┐
    │ Tenant │  │ Tenant │  │ Tenant │
    │   A    │  │   B    │  │   C    │
    └────────┘  └────────┘  └────────┘
```

### 3.3 Distributed (Remote Nodes)

- Each node runs full security stack
- Mutual TLS with SPIFFE identities
- Central policy distribution
- Federated audit log
- Cross-node threat correlation

---

## 4. API Surface

### 4.1 Core Security Traits

```rust
// Capability checking (used by all components)
#[async_trait]
pub trait CapabilityChecker: Send + Sync {
    async fn check(&self, subject: &Subject, capability: &str, context: &CheckContext) -> CheckResult;
    async fn check_all(&self, subject: &Subject, capabilities: &[&str], context: &CheckContext) -> Vec<CheckResult>;
}

// Permission evaluation
#[async_trait]
pub trait PermissionEvaluator: Send + Sync {
    async fn evaluate(&self, request: &PermissionRequest) -> PermissionDecision;
}

// Secret access
#[async_trait]
pub trait SecretAccessor: Send + Sync {
    async fn get_secret(&self, id: &SecretId, context: &AccessContext) -> Result<Secret>;
    async fn put_secret(&self, secret: &Secret, context: &AccessContext) -> Result<()>;
}

// Audit logging
#[async_trait]
pub trait AuditLogger: Send + Sync {
    async fn log(&self, entry: &AuditEntry) -> Result<EntryId>;
    async fn query(&self, query: &AuditQuery) -> Result<Vec<AuditEntry>>;
}
```

### 4.2 Integration Points

| Component | Security Integration |
|-----------|---------------------|
| **Kernel** | Service registration requires capabilities |
| **IPC** | Every message carries capability token |
| **Plugin Runtime** | Sandbox enforced by capability grants |
| **Agent Runtime** | Agents spawn with minimal capabilities |
| **Voice/Vision/Automation** | Capability-gated backends |
| **Model Router** | Model access via capabilities |
| **Database** | Row-level security via capabilities |
| **Platform** | OS permission mapping to capabilities |

---

## 5. Threat Model

### 5.1 Assets

| Asset | Classification | Protection |
|-------|----------------|------------|
| User conversations | PII/Confidential | Encryption, access control |
| API keys / tokens | Secret | Vault, zeroization |
| Model weights | IP/Confidential | Encryption, integrity |
| Plugin code | Code | Signature verification |
| Audit logs | Compliance | Immutable, signed |
| User consent records | Legal | Immutable, auditable |
| System integrity | Critical | Measured boot, runtime verification |

### 5.2 Threat Actors

| Actor | Capabilities | Motivation |
|-------|--------------|------------|
| Malicious plugin | Code execution in sandbox | Data theft, persistence |
| Compromised network | MITM, replay | Credential theft |
| Malicious insider | Admin access | Data exfiltration |
| Physical attacker | Device access | Secret extraction |
| Supply chain | Compromised dependency | Backdoor |
| Nation state | Advanced persistent | Espionage |

### 5.3 Mitigations

| Threat | Mitigation |
|--------|------------|
| Plugin escape | WASM sandbox + capability deny-by-default |
| Token theft | Short-lived, bound to device, rotatable |
| MITM | mTLS everywhere, certificate pinning |
| Secret leakage | Vault, zeroization, no logs |
| Persistence | Immutable audit, integrity verification |
| Privilege escalation | Policy engine, least privilege |
| Data exfil | Egress control, DLP rules |

---

## 6. Compliance Mapping

| Requirement | Implementation |
|-------------|----------------|
| GDPR Art. 32 | Encryption, access control, audit |
| SOC 2 Type II | Audit log, integrity, monitoring |
| HIPAA | Audit, access control, encryption |
| FedRAMP | FIPS 140-2, continuous monitoring |
| ISO 27001 | Risk assessment, controls, audit |
| NIST 800-53 | AC, AU, SC, SI families |
| Zero Trust (NIST 800-207) | Never trust, always verify |

---

## 7. Implementation Phases

### Phase 1: Core Identity & Capabilities (Week 1-2)
- [ ] `IdentityManager` with entity registration, hierarchy, lookup
- [ ] `CapabilityManager` with risk-level taxonomy (NONE → CRITICAL)
- [ ] `TrustManager` with trust level calculation and evolution
- [ ] `PermissionManager` with basic evaluation
- [ ] `SecretVault` with file backend + zeroization
- [ ] `TokenManager` with JWT issuance/validation

### Phase 2: Guardian Engine (Week 2-3)
- [ ] `GuardianEngine` with 8-step decision flow
- [ ] `RiskAnalyzer` with dynamic risk scoring
- [ ] `ConsentManager` with UI integration + consent flows
- [ ] `PolicyEngine` with OPA/Rego
- [ ] Built-in policy bundles for all risk levels
- [ ] Policy testing framework

### Phase 3: Hardening (Week 3-4)
- [ ] `IntegrityVerifier` with TPM
- [ ] `ThreatDetector` with rule engine
- [ ] `GuardianMode` with multiple triggers
- [ ] `RecoveryMode` with multi-party auth
- [ ] `SecureShutdown` with zeroization sequence

### Phase 4: Journal & Rollback (Week 4-5)
- [ ] `AiActionJournal` with full context recording
- [ ] `RollbackManager` with rollback strategy framework
- [ ] Standard rollback strategies for all destructive capabilities
- [ ] Journal replay for crash recovery
- [ ] `AuditLog` with hash chain + tamper evidence

### Phase 5: Enterprise (Week 5-6)
- [ ] Vault backends (HashiCorp, AWS, Azure)
- [ ] SPIFFE/SPIRE integration for identity
- [ ] Centralized audit log aggregation
- [ ] RBAC/ABAC policies with hierarchy
- [ ] Compliance reporting (SOC2, HIPAA, FedRAMP)

### Phase 6: Advanced (Week 6-7)
- [ ] ML-based threat detection
- [ ] Remote attestation
- [ ] Federated audit across nodes
- [ ] Automated response playbooks
- [ ] Home Automation security provider interfaces

---

## 8. Review Checklist

- [ ] Capability taxonomy with 5 risk levels complete
- [ ] Every tool/agent/plugin declares capabilities
- [ ] Guardian 8-step decision flow documented
- [ ] Identity model covers all entity types
- [ ] Trust levels defined with evolution paths
- [ ] Consent flows for LOW → CRITICAL
- [ ] Risk analysis formula defined
- [ ] Vault backends cover all deployments
- [ ] Token format supports all use cases
- [ ] Policy language expressive enough
- [ ] Integrity verification measurable
- [ ] Audit log tamper-evident
- [ ] AI Action Journal schema complete
- [ ] Rollback strategies for all critical capabilities
- [ ] Threat detection rules documented
- [ ] Guardian/Recovery modes testable
- [ ] Zeroization guaranteed
- [ ] Enterprise features designed
- [ ] Home automation provider interfaces ready
- [ ] Compliance gaps identified
- [ ] Performance budgets set
- [ ] Integration points documented

---

**Next Step**: Internal review → approve → implement Phase 1