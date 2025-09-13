# language: en
Feature: Atomic swap and no broken visibility
  Ensures that swaps are atomic with crash safety and never expose a broken or missing path.

  @REQ-A1 @REQ-A2
  Scenario: Enable And Rollback Remains Atomic And Visible
    Given a plan with a single symlink replacement action
    And the target path currently resolves to providerA/ls
    When I apply the plan in Commit mode
    Then the target path resolves to providerB/ls without any intermediate missing path visible
    And if a crash is simulated immediately after rename, recovery yields a valid link

  @REQ-A3
  Scenario: All-Or-Nothing Per Plan
    Given a plan with two actions where the second will fail
    When I apply the plan in Commit mode
    Then the engine performs reverse-order rollback of any executed actions
    And no visible mutations remain on the filesystem
