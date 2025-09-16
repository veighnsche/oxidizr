Feature: Use and restore findutils
  As an operator
  I want to rustify findutils and be able to restore

  Scenario: Commit use findutils then restore
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    And a verified replacement artifact is available for package "findutils"
    When I run `oxidizr-deb --commit use findutils`
    Then the command exits 0
    And `/usr/bin/find` is a symlink to the replacement
    When I run `oxidizr-deb --commit restore findutils`
    Then the command exits 0
    And `/usr/bin/find` is a regular file with content `gnu-find`
