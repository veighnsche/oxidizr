# language: en
Feature: Rescue profile and fallback toolset

  @REQ-RC1
  Scenario: Rescue Profile Always Available
    Given a system with configured rescue profile
    When I inspect preflight and emitted facts
    Then the presence of a rescue symlink set is recorded

  @REQ-RC2 @REQ-RC3
  Scenario: Preflight Verifies At Least One Fallback Path
    Given no BusyBox but GNU core utilities are present on PATH
    When I run preflight
    Then preflight verifies at least one functional fallback path is executable
