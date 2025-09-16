Feature: Restore coreutils without keep (dry-run)
  As an operator
  I want to preview restore that removes RS packages by default

  Scenario: Dry-run restore coreutils without --keep-replacements
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    When I run `oxidizr-deb restore coreutils`
    Then the command exits 0
    And output contains `[dry-run] would run: apt-get install -y coreutils`
    And output contains `[dry-run] would run: apt-get purge -y uutils-coreutils`
