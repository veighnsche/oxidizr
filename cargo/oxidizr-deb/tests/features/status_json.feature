Feature: Status JSON output
  As an operator
  I want machine-readable status

  Scenario: Status --json returns keys
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    When I run `oxidizr-deb status --json`
    Then the command exits 0
    And output contains `{"coreutils":"unset","findutils":"unset","sudo":"unset"}`
