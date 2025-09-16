Feature: Use requires live root for missing artifact in commit
  As an operator
  I want use to error if the replacement artifact is missing and we're not on live root during commit

  Scenario: Commit use coreutils without verified artifact under non-live root should error
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    When I run `oxidizr-deb --commit use coreutils`
    Then the command exits 1
    And output contains `replacement artifact missing at`
    And output contains `requires --root=/ (live system)`
