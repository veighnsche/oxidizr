# language: en
Feature: Error taxonomy and exit codes

  @REQ-E1
  Scenario: Stable Error Identifiers Emitted In Facts
    Given failures during preflight or apply
    When facts are emitted
    Then error identifiers such as E_POLICY or E_LOCKING are stable strings

  @REQ-E2
  Scenario: Preflight Summary Maps To Policy Exit Code
    Given preflight STOP conditions are present
    When I compute the process exit
    Then preflight summary carries error_id=E_POLICY and exit_code=10
