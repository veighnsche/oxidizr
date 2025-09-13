# language: en
Feature: Thread safety

  @REQ-T1
  Scenario: Core Types Are Send And Sync
    Given the Switchyard core types
    Then they are Send + Sync for safe use across threads

  @REQ-T2
  Scenario: Only One Mutator Proceeds Under Lock
    Given two threads invoking apply() concurrently
    And a LockManager is configured
    When both apply() calls run
    Then only one mutator proceeds at a time under the lock
