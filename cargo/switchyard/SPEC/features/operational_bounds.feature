Feature: Operational bounds
  As a performance-conscious operator
  I want bounded fsync timing after rename
  So that durability is assured without undue latency

  @REQ-BND1
  Scenario: fsync within 50ms after rename
    Given a rename completes for a staged swap
    When the engine performs fsync on the parent directory
    Then the fsync occurs within 50ms of the rename and is recorded in telemetry
