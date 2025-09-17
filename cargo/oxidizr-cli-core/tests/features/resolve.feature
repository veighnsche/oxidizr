Feature: Resolve applets for use
  Scenario: intersection with distro when present
    Given a staging root at /tmp/fakeroot
    And a replacement artifact lists applets "ls du echo cat" at `/opt/repl/bin`
    And the distro commands for package "coreutils" are "ls cat"
    When I call resolve_applets_for_use for package "coreutils"
    Then the vector equals "cat ls"

  Scenario: no distro list returns replacement set
    Given a staging root at /tmp/fakeroot
    And a replacement artifact lists applets "ls cat mv du" at `/opt/repl/bin`
    And the distro commands for package "coreutils" are ""
    When I call resolve_applets_for_use for package "coreutils"
    Then the vector contains "ls cat mv du"
