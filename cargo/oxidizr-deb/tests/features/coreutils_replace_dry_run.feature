Feature: Replace coreutils (dry-run)
  As an operator
  I want to see the apt/dpkg operations previewed when running replace in dry-run

  Scenario: Dry-run replace coreutils
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    When I run `oxidizr-deb replace coreutils`
    Then the command exits 0
    And output contains `[dry-run] would run: apt-get install -y uutils-coreutils`
    And output contains `[dry-run] would run: apt-get purge -y coreutils`
