# language: en
Feature: Determinism and redaction

  @REQ-D1
  Scenario: Deterministic IDs Are UUIDv5 Over Normalized Inputs
    Given a plan built from a stable set of inputs
    When I compute plan_id and action_id
    Then they are deterministic UUIDv5 values under the project namespace

  @REQ-D2
  Scenario: Dry-Run Facts Byte-Identical After Redaction
    Given a plan with at least one action
    When I run in DryRun and Commit modes
    Then facts are byte-identical after timestamp redaction
