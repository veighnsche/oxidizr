use crate::api::{DebugAttestor, DebugLockManager, DebugOwnershipOracle, DebugSmokeTestRunner};
use crate::logging::{AuditSink, FactsEmitter};
use crate::policy::Policy;
use crate::constants::DEFAULT_LOCK_TIMEOUT_MS;

/// Builder for constructing a Switchyard with ergonomic chaining.
/// Mirrors `Switchyard::new(...).with_*` but avoids duplication at call sites.
#[derive(Debug)]
pub struct ApiBuilder<E: FactsEmitter, A: AuditSink> {
    facts: E,
    audit: A,
    policy: Policy,
    // Optional adapters/handles
    lock: Option<Box<dyn DebugLockManager>>,              // None in dev/test; required in production
    owner: Option<Box<dyn DebugOwnershipOracle>>,         // strict ownership gating
    attest: Option<Box<dyn DebugAttestor>>,               // final summary attestation
    smoke: Option<Box<dyn DebugSmokeTestRunner>>,         // post-apply health verification
    lock_timeout_ms: Option<u64>,
}
    
impl<E: FactsEmitter, A: AuditSink> ApiBuilder<E, A> {
    pub fn new(facts: E, audit: A, policy: Policy) -> Self {
        Self {
            facts,
            audit,
            policy,
            lock: None,
            owner: None,
            attest: None,
            smoke: None,
            lock_timeout_ms: None,
        }
    }

    /// Build a `Switchyard` with the configured options.
    ///
    /// Example
    /// ```rust
    /// use switchyard::api::ApiBuilder;
    /// use switchyard::policy::Policy;
    /// use switchyard::logging::JsonlSink;
    ///
    /// let facts = JsonlSink::default();
    /// let audit = JsonlSink::default();
    /// let api = ApiBuilder::new(facts, audit, Policy::default())
    ///     .with_lock_timeout_ms(500)
    ///     .build();
    /// ```
    pub fn build(self) -> super::Switchyard<E, A> {
        // Construct directly to avoid recursion when Switchyard::new delegates to ApiBuilder
        let mut api = super::Switchyard {
            facts: self.facts,
            audit: self.audit,
            policy: self.policy,
            lock: None,
            owner: None,
            attest: None,
            smoke: None,
            lock_timeout_ms: self.lock_timeout_ms.unwrap_or(DEFAULT_LOCK_TIMEOUT_MS),
        };
        if let Some(lock) = self.lock { api.lock = Some(lock); }
        if let Some(owner) = self.owner { api.owner = Some(owner); }
        if let Some(att) = self.attest { api.attest = Some(att); }
        if let Some(smoke) = self.smoke { api.smoke = Some(smoke); }
        api
    }

    pub fn with_lock_manager(mut self, lock: Box<dyn DebugLockManager>) -> Self {
        self.lock = Some(lock);
        self
    }

    pub fn with_ownership_oracle(mut self, owner: Box<dyn DebugOwnershipOracle>) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_attestor(mut self, attest: Box<dyn DebugAttestor>) -> Self {
        self.attest = Some(attest);
        self
    }

    pub fn with_smoke_runner(mut self, smoke: Box<dyn DebugSmokeTestRunner>) -> Self {
        self.smoke = Some(smoke);
        self
    }

    pub fn with_lock_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.lock_timeout_ms = Some(timeout_ms);
        self
    }
}
