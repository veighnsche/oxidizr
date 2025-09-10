# language: en
Feature: Atomic swap and recovery
  As a system integrator
  I want atomic, reversible filesystem swaps with clear audit and fallback behavior
  So that upgrades are safe, observable, and recoverable

  @REQ-A1 @REQ-A2 @REQ-R1 @REQ-R2 @REQ-R3
  Scenario: Enable and rollback
    Given /usr/bin/ls is a symlink to providerA/ls
    When I plan a swap to providerB
    And I apply the plan
    Then /usr/bin/ls resolves to providerB/ls atomically
    And rollback restores providerA/ls

  @REQ-F1 @REQ-F2
  Scenario: Cross-filesystem EXDEV fallback
    Given the target and staging directories reside on different filesystems
    When I apply a plan that replaces /usr/bin/cp
    Then the operation handles EXDEV by copy+sync+rename into place atomically
    And facts record degraded=true when policy allow_degraded_fs is enabled

  @REQ-R4 @REQ-R5 @REQ-A3
  Scenario: Automatic rollback on mid-plan failure
    Given a plan with three actions A, B, C
    And action B will fail during apply
    When I apply the plan
    Then the engine automatically rolls back A in reverse order
    And facts clearly indicate partial restoration state if any rollback step fails

  @REQ-H1 @REQ-H2 @REQ-H3
  Scenario: Smoke test failure triggers rollback
    Given the minimal post-apply smoke suite is configured
    And at least one smoke command will fail with a non-zero exit
    When I apply the plan
    Then the smoke suite runs and detects the failure
    And automatic rollback occurs unless policy explicitly disables it
