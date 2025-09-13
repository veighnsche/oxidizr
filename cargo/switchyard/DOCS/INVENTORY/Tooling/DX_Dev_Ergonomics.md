# Developer ergonomics and hygiene

- Category: DX
- Maturity: Silver

## Summary

Crate forbids unsafe code and enables strict clippy lints; modular layout with focused modules; tests cover core atoms.

## Implementation

- Root crate flags: `cargo/switchyard/src/lib.rs` sets `#![forbid(unsafe_code)]`, denies unwrap/expect in non-test, enables clippy pedantic.
- Clear module boundaries (`api/`, `fs/`, `logging/`, `policy/`, `types/`, `adapters/`).

## Wiring Assessment

- All modules compile cleanly; tests validate atoms and stages.
- Conclusion: wired correctly.

## Evidence and Proof

- Successful compilation with lints on; unit tests across fs/* and api.rs tests.

## Gaps and Risks

- Some planned refactors marked with comments in fs/backup.rs and logging/audit.rs.

## Next Steps to Raise Maturity

- Complete planned refactors and remove BEGIN/END REMOVE markers.

## Related

- zrefactor plans under `cargo/switchyard/zrefactor/`.
