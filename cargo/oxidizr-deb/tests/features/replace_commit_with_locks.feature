Feature: APT/DPKG locks block replace commit
  As an operator
  I want replace to fail closed when apt/dpkg locks are present

  Scenario: Locks block commit replace coreutils
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    And dpkg/apt locks are present
    When I run `oxidizr-deb --commit replace coreutils`
    Then the command exits 1
    And output contains `Package manager busy (dpkg/apt lock detected); retry after current operation finishes.`
