Feature: Coverage check
  Scenario: full coverage OK
    When I call coverage_check with distro "ls cat" and repl "ls cat echo"
    Then the result is Ok

  Scenario: missing applets reported
    When I call coverage_check with distro "ls cat mv" and repl "ls"
    Then the result is Err with missing "cat mv"
