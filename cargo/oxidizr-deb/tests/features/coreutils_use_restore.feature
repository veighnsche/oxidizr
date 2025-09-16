Feature: Use and restore coreutils
  As an operator
  I want to use coreutils and be able to restore

  Scenario: Commit use coreutils then restore
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    And a verified replacement artifact is available for package "coreutils"
    When I run `oxidizr-deb --commit use coreutils`
    Then the command exits 0
    And `/usr/bin/ls` is a symlink to the replacement
    When I run `oxidizr-deb --commit restore coreutils`
    Then the command exits 0
    And `/usr/bin/ls` is a regular file with content `gnu-ls`
