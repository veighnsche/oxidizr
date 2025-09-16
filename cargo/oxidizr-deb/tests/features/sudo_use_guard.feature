Feature: Sudo setuid guard
  As an operator
  I want commit to refuse sudo replacement without setuid 4755

  Scenario: Commit sudo without setuid fails
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    And a verified replacement artifact is available for package "sudo"
    When I run `oxidizr-deb --commit use sudo`
    Then the command exits 1
    And output contains `sudo replacement must be root:root with mode=4755 (setuid root)`

  Scenario: Commit sudo with setuid passes (owner relaxed for tests)
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    And a verified replacement artifact is available for package "sudo"
    And the sudo artifact has setuid 4755
    And non-root sudo owner is allowed in tests
    When I run `oxidizr-deb --commit use sudo`
    Then the command exits 0
    And `/usr/bin/sudo` is a symlink to the replacement
