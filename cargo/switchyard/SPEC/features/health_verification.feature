# language: en
Feature: Health verification and auto-rollback

  @REQ-H1 @REQ-H3
  Scenario: Minimal Smoke Suite Runs In Commit
    Given a Switchyard with SmokePolicy Require
    And a configured SmokeTestRunner
    When I apply a plan in Commit mode
    Then the minimal smoke suite runs after apply

  @REQ-H2
  Scenario: Smoke Failure Triggers Auto-Rollback
    Given a failing SmokeTestRunner
    And auto_rollback is enabled
    When I apply a plan in Commit mode
    Then apply fails with error_id=E_SMOKE and exit_code=80
    And executed actions are rolled back automatically
