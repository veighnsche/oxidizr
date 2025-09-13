# language: en
Feature: Cross-filesystem EXDEV degraded fallback

  @REQ-F1 @REQ-F2
  Scenario: Degraded Fallback When Policy Allows
    Given staging and target parents reside on different filesystems (EXDEV)
    And policy allow_degraded_fs is true
    When I apply a symlink replacement plan
    Then the operation completes via safe copy + fsync + rename preserving atomic visibility
    And emitted facts record degraded=true with degraded_reason="exdev_fallback"

  @REQ-F1 @REQ-F2
  Scenario: Disallowed Degraded Fallback Fails With Classification
    Given EXDEV conditions
    And policy allow_degraded_fs is false
    When I apply a symlink replacement plan
    Then the apply fails with error_id=E_EXDEV and exit_code=50
    And emitted facts include degraded=false with degraded_reason="exdev_fallback"

  @REQ-F3
  Scenario: Supported Filesystems Verified
    Given an environment matrix with ext4, xfs, btrfs, and tmpfs
    When I run acceptance tests
    Then semantics for rename and degraded path are verified per filesystem
