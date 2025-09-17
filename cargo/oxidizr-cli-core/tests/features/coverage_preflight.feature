Feature: Coverage preflight
  Scenario: missing applets fails
    Given a staging root at /tmp/fakeroot
    And a replacement artifact lists applets "ls echo mv" at `/opt/repl/bin`
    And the distro commands for package "coreutils" are "ls cat"
    When I call coverage_preflight for package "coreutils"
    Then the result is Err with missing "cat"

  Scenario: no distro enumeration passes (non-live root behavior)
    Given a staging root at /tmp/fakeroot
    And a replacement artifact lists applets "ls" at `/opt/repl/bin`
    And the distro commands for package "coreutils" are ""
    When I call coverage_preflight for package "coreutils"
    Then the result is Ok
