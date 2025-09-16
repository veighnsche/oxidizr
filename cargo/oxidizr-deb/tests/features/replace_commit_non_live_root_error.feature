Feature: Replace requires live root for commit
  As an operator
  I want replace to refuse commit under non-live root

  Scenario: Commit replace coreutils under non-live root should error
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    When I run `oxidizr-deb --commit replace coreutils`
    Then the command exits 1
    And output contains `replace operations require --root=/ (live system) for apt/dpkg changes`
