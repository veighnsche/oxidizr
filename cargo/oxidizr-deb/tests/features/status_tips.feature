Feature: Status tips suggest restore and replace
  As an operator
  I want status to suggest restore and replace next steps

  Scenario: Status prints tips when active
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    And a verified replacement artifact is available for package "coreutils"
    When I run `oxidizr-deb --commit use coreutils`
    Then the command exits 0
    When I run `oxidizr-deb status`
    Then the command exits 0
    And output contains `restore coreutils --commit`
    And output contains `--commit replace coreutils`
