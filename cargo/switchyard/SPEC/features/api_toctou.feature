Feature: API safety and TOCTOU sequence
  As a library consumer and reviewer
  I want mutating APIs to require SafePath and follow a TOCTOU-safe sequence
  So that the engine resists path traversal and race attacks

  @REQ-API1
  Scenario: Mutating public APIs require SafePath
    Given a mutating public API endpoint
    When I inspect its signature
    Then the signature requires SafePath and does not accept PathBuf

  @REQ-TOCTOU1
  Scenario: TOCTOU-safe syscall sequence is normative
    Given a mutation of a final path component under a parent directory
    When the engine performs the operation
    Then it opens the parent with O_DIRECTORY|O_NOFOLLOW, uses openat on the final component, renames with renameat, and fsyncs the parent
