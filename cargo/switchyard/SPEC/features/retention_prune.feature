# language: en
Feature: Retention and prune backups

  @REQ-PN1
  Scenario: Prune Retains Newest Backup
    Given a target with multiple backup artifacts
    When I prune backups under policy
    Then the newest backup is never deleted

  @REQ-PN2
  Scenario: Prune Deletes Payload And Sidecar And Fsyncs Parent
    Given eligible backups older than retention limits
    When I prune backups under policy
    Then deletions remove payload and sidecar pairs and fsync the parent directory

  @REQ-PN3
  Scenario: Prune Emits Result Summary Event
    Given a prune operation completed
    When I inspect emitted facts
    Then a prune.result event includes path, policy_used, pruned_count, and retained_count
