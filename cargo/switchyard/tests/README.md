# Switchyard crate tests

This directory contains unit/integration/golden/schema/trybuild tests for the `switchyard` crate. Tests are grouped by domain for discoverability and faster iteration.

Layout
- common.rs: shared helpers (TestEmitter/TestAudit, temp roots, canon helpers)
- audit/: audit schema, provenance, and summary-id tests
- locking/: lock wait/attempts/required/timeout/stage parity
- preflight/: preservation, suid/sgid, YAML export
- apply/: API flows, smoke tests, perf aggregation, attestation, error paths
- fs/: restore round-trip, prune
- trybuild/: compile-time surface checks
- golden/: canonical fixtures for JSON/YAML comparisons

Conventions
- At the top of each test file, add `mod common;` to import helpers.
- Prefer integration tests for flows that touch the filesystem; keep inline unit tests small and local.
