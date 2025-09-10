Feature: Conservatism and CI gates
  As a maintainer
  I want dry-run by default, fail-closed policy, and strict CI gates
  So that changes are safe and regressions are caught early

  @REQ-C1
  Scenario: Dry-run is the default mode
    Given no explicit approval flag is provided
    When I run the engine
    Then it runs in dry-run mode by default

  @REQ-C2
  Scenario: Fail-closed on critical violations
    Given a critical compatibility violation is detected in preflight
    When I run the engine with default policy
    Then the operation fails closed unless an explicit override is present

  @REQ-CI1 @REQ-CI2 @REQ-CI3
  Scenario: CI gates for golden fixtures and zero-SKIP
    Given golden fixtures for plan, preflight, apply, and rollback
    And a required test is marked SKIP or a fixture diff is not byte-identical
    When CI runs
    Then the CI job fails
