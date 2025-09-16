Feature: Restore coreutils with keep-replacements (dry-run)
  As an operator
  I want to preview restore with keeping RS packages installed

  Scenario: Dry-run restore coreutils with --keep-replacements
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    When I run `oxidizr-deb restore coreutils --keep-replacements`
    Then the command exits 0
    And output contains `[dry-run] would run: apt-get install -y coreutils`
    And output does not contain `[dry-run] would run: apt-get purge -y uutils-coreutils`
