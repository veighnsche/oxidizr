Feature: Replace sudo (dry-run)
  As an operator
  I want to see apt/dpkg operations previewed for sudo when running replace in dry-run

  Scenario: Dry-run replace sudo
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    When I run `oxidizr-deb replace sudo`
    Then the command exits 0
    And output contains `[dry-run] would run: apt-get purge -y sudo`
