Feature: Safety preconditions and gating
  As a platform security engineer
  I want strict path safety and environment gating
  So that mutations cannot escape allowed roots or violate policy

  @REQ-S1
  Scenario: SafePath rejects escaping paths
    Given a candidate path containing .. segments or symlink escapes
    When I attempt to construct a SafePath
    Then SafePath normalization rejects the path as unsafe

  @REQ-S2
  Scenario: Fail on unsupported filesystem state
    Given the target filesystem is read-only or noexec or immutable
    When I attempt to apply a plan
    Then operations fail closed with a policy violation error

  @REQ-S3
  Scenario: Source ownership gating
    Given a source file that is not root-owned or is world-writable
    When I run preflight
    Then preflight fails closed unless an explicit policy override is present

  @REQ-S4
  Scenario: Strict package ownership for targets
    Given strict_ownership=true policy
    And a target that is not package-owned per the ownership oracle
    When I run preflight
    Then preflight fails closed

  @REQ-S5
  Scenario: Preservation capability gating in preflight
    Given the policy requires preserving owner, mode, timestamps, xattrs, ACLs, and caps
    And the filesystem or environment lacks support for one or more of these
    When I run preflight
    Then preflight stops with a fail-closed decision unless an explicit override is set
