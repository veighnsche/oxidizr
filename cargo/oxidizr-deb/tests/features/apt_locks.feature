Feature: APT/DPKG locks prevent commit
  As a Debian/Ubuntu operator
  I want oxidizr-deb to refuse to commit when package manager locks are present

  Scenario: dpkg/apt lock blocks commit use
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    And dpkg/apt locks are present
    And a verified replacement artifact is available for package "coreutils"
    When I run `oxidizr-deb --commit use coreutils`
    Then the command exits 1
    And output contains `Package manager busy (dpkg/apt lock detected); retry after current operation finishes.`
