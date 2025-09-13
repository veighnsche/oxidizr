# language: en
Feature: Locking and single mutator

  @REQ-L1 @REQ-L4
  Scenario: Single Mutator Enforced In Commit
    Given a Switchyard built with a LockManager
    And two apply() operations targeting overlapping paths
    When both apply() are started in Commit mode
    Then only one mutator proceeds at a time

  @REQ-L2
  Scenario: Warn Fact When No Lock Manager
    Given a Switchyard without a LockManager
    When I apply a plan in Commit mode
    Then a WARN fact is emitted stating concurrent apply is unsupported

  @REQ-L3
  Scenario: Bounded Lock Wait Emits Timeout And Metrics
    Given a LockManager configured with a short timeout
    And another process holds the lock
    When I attempt apply in Commit mode
    Then the stage fails with error_id=E_LOCKING and exit_code=30
    And apply.attempt includes lock_wait_ms

  @REQ-L5
  Scenario: Lock Attempts Included In Apply Attempt
    Given a contended lock with retries
    When I apply a plan in Commit mode
    Then apply.attempt includes lock_attempts approximating retry count
