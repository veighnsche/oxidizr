Feature: Discover applets with allow
  Scenario: dynamic discovery returns intersection when large enough
    Given a staging root at /tmp/fakeroot
    And a replacement artifact lists applets "ls cat echo mv" at `/opt/repl/bin`
    When I call discover_applets_with_allow with allow "ls cat mv"
    Then the vector equals "ls cat mv"

  Scenario: fallback to allow when discovery is too small
    Given a staging root at /tmp/fakeroot
    And a replacement artifact lists applets "ls" at `/opt/repl/bin`
    When I call discover_applets_with_allow with allow "ls cat mv"
    Then the vector equals "ls cat mv"
