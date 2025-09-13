# language: en
Feature: Rollback guarantees and idempotence

  @REQ-R1 @REQ-R4 @REQ-R5
  Scenario: Automatic Reverse-Order Rollback On Failure
    Given a plan with three actions A, B, C where B will fail
    When I apply the plan in Commit mode
    Then the engine rolls back A in reverse order automatically
    And emitted facts include partial restoration state if any rollback step fails

  @REQ-R2 @REQ-R3
  Scenario: Idempotent Rollback Restores Exact Topology
    Given a plan that replaces a symlink then restores it
    When I apply the plan and then apply a rollback plan twice
    Then the final link/file topology is identical to the prior state
