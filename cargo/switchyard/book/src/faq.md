# FAQ

- What is Switchyard?
  - A Rust library for safe, atomic, and reversible swaps with strong auditability. Not a CLI.
- Can I run without a LockManager?
  - Only in development/testing. Production requires a lock to serialize mutations; otherwise, `E_LOCKING` risks.
- How do I integrate with my package manager?
  - Use adapters (e.g., OwnershipOracle, PathResolver) in your integration layer; keep Switchyard focused on filesystem safety.
- Where do I see what happened?
  - Read JSON facts (schema v2). Dry-run facts should match Commit facts after redaction.
- How does degraded EXDEV show up?
  - `degraded=true` with `degraded_reason` (e.g., "exdev_fallback"). Policy controls acceptance.
- What are the minimal smoke tests?
  - See SPEC §11. Failures trigger auto-rollback unless policy disables it.
