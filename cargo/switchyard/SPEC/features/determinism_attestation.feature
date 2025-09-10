Feature: Determinism and attestation
  As an auditor
  I want stable identifiers and signed bundles
  So that runs are reproducible and attestable

  @REQ-D1
  Scenario: Deterministic UUIDv5 plan and action IDs
    Given normalized plan input and a stable namespace
    When I generate plan_id and action_id
    Then they are UUIDv5 values derived from the normalized input and namespace

  @REQ-O4
  Scenario: Signed attestation per apply bundle
    Given an apply bundle
    When I complete an apply
    Then an attestation is generated and signed with ed25519 and attached to the facts
