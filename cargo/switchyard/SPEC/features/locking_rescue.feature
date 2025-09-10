# language: en
Feature: Locking and rescue behavior
  As a deployer in production
  I want safe concurrent behavior and a guaranteed rescue profile
  So that only one mutator runs and recovery is always possible

  @REQ-L1 @REQ-L3 @REQ-L4
  Scenario: Bounded locking in production
    Given a production deployment with a LockManager
    And another apply() is already holding the lock
    When I attempt to apply a plan
    Then lock acquisition uses a bounded wait and times out with E_LOCKING when exceeded
    And facts record lock_wait_ms

  @REQ-L2
  Scenario: No LockManager in dev/test emits WARN
    Given a development environment without a LockManager
    When two apply() calls overlap in time
    Then concurrent apply is UNSUPPORTED and a WARN fact is emitted

  @REQ-RC1 @REQ-RC2 @REQ-RC3
  Scenario: Rescue profile and fallback toolset verified
    Given a configured rescue profile consisting of backup symlinks
    And at least one fallback binary set (GNU or BusyBox) is installed and on PATH
    When I run preflight
    Then preflight verifies a functional fallback path
    And a rescue profile remains available for recovery
