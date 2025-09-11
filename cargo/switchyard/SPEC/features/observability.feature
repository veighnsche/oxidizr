# language: en
Feature: Observability, audit, and determinism
  As an operator and auditor
  I want complete, structured, and deterministic facts
  So that I can trust and validate system behavior over time

  @REQ-O1 @REQ-O3 @REQ-VERS1
  Scenario: Every step emits a structured fact conforming to schema v1
    Given a plan with at least one action
    When I run in real mode
    Then every stage emits a JSON fact that validates against /SPEC/audit_event.schema.json
    And each fact carries schema_version=1

  @REQ-O2 @REQ-D2
  Scenario: Dry-run facts are byte-identical to real-run facts after redaction
    Given a plan with at least one action
    When I run in dry-run mode
    And I run in real mode
    Then the emitted facts for plan and preflight are byte-identical after timestamp redaction
    And the emitted facts for apply.result per-action events are byte-identical after redaction

  @REQ-O5
  Scenario: Before/after hashes are recorded for mutated files
    Given a plan that mutates a file
    When I apply the plan
    Then the resulting facts include hash_alg=sha256 and both before_hash and after_hash

  @REQ-O6
  Scenario: Secret masking is enforced across all sinks
    Given a plan with environment-derived values that may be sensitive
    When I apply the plan
    Then no unmasked secret values appear in any emitted fact or log sink

  @REQ-O7 @xfail
  Scenario: Provenance fields are complete
    Given a plan that uses an external helper
    When I apply the plan
    Then facts include origin, helper, uid, gid, pkg, and env_sanitized=true
