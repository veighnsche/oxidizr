use crate::logging::{AuditSink, FactsEmitter};
use crate::policy::Policy;

/// Builder for constructing a Switchyard with ergonomic chaining.
/// Mirrors `Switchyard::new(...).with_*` but avoids duplication at call sites.
pub struct ApiBuilder<E: FactsEmitter, A: AuditSink> {
    facts: E,
    audit: A,
    policy: Policy,
}

impl<E: FactsEmitter, A: AuditSink> ApiBuilder<E, A> {
    pub fn new(facts: E, audit: A, policy: Policy) -> Self {
        Self { facts, audit, policy }
    }

    pub fn build(self) -> super::Switchyard<E, A> {
        super::Switchyard::new(self.facts, self.audit, self.policy)
    }
}
