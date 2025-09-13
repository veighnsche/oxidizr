# language: en
Feature: Conservatism and modes

  @REQ-C1
  Scenario: Dry-Run Is Default Mode
    Given a newly constructed Switchyard
    When I execute without explicit commit approval
    Then side effects are not performed (DryRun is default)

  @REQ-C2
  Scenario: Fail-Closed On Critical Violations Unless Overridden
    Given a policy requiring strict ownership and unsupported preservation
    When I run preflight and apply in Commit mode
    Then the operation fails closed unless an explicit policy override is set
