# language: en
Feature: Observability and audit (schema v2)

  @REQ-O1 @REQ-O3 @REQ-VERS1
  Scenario: Every Step Emits Schema V2 Facts
    Given a plan with at least one action
    When I run in Commit mode
    Then each stage emits a JSON fact that validates against /SPEC/audit_event.v2.schema.json
    And each fact carries schema_version=2

  @REQ-O2 @REQ-D2
  Scenario: Dry-Run Facts Byte-Identical To Real-Run After Redaction
    Given a plan with at least one action
    When I run in DryRun and Commit modes
    Then emitted facts are byte-identical after timestamp redaction

  @REQ-O5
  Scenario: Before And After Hashes Recorded For Mutations
    Given a plan that mutates a file
    When I apply the plan
    Then apply.result includes hash_alg=sha256 and both before_hash and after_hash

  @REQ-O6
  Scenario: Secret Masking Enforced Across Sinks
    Given environment-derived sensitive values might appear in facts
    When I apply the plan
    Then no unmasked secret values appear in any emitted fact or sink

  @REQ-O8
  Scenario: Summary Error Chain Present On Failures
    Given a failing preflight or apply stage
    When I inspect summary events
    Then summary_error_ids is present and ordered from specific to general

  @REQ-O4
  Scenario: Attestation Emitted On Apply Success
    Given an attestor is configured and apply succeeds in Commit mode
    When I inspect apply.result
    Then attestation fields (sig_alg, signature, bundle_hash, public_key_id) are present

  @REQ-PF1
  Scenario: Preflight YAML Dry-Run Byte-Identical To Real
    Given a plan with at least one action
    When I run preflight in DryRun and Commit modes
    Then the exported preflight YAML rows are byte-identical between runs
