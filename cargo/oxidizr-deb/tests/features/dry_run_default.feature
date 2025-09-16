Feature: Dry-run is default for use
  As a cautious operator
  I want oxidizr-deb to run in dry-run by default
  So that I can preview changes without mutations

  Scenario: Dry-run use of coreutils
    Given a staging root at /tmp/fakeroot
    And a fakeroot with stock coreutils applets
    And a verified replacement artifact is available for package "coreutils"
    When I run `oxidizr-deb use coreutils`
    Then the command exits 0
    And it reports a dry-run with a non-zero planned action count
