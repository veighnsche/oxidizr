# Policy gating and preflight

- Category: Safety
- Maturity: Silver

## Summary

Preflight computes per-action policy status and emits facts. Apply refuses Commit when policy gates fail unless `override_preflight=true`.

## Implementation

- Orchestrator: `cargo/switchyard/src/api/preflight/mod.rs` builds rows, detects preservation, ownership, suid/sgid, hardlink hazards, mount rw+exec, immutability.
- Policy evaluation helper: `cargo/switchyard/src/policy/gating.rs::gating_errors()` mirrors checks for Commit enforcement.
- Policy model: `cargo/switchyard/src/policy/config.rs` (allow_roots, forbid_paths, strict_ownership, require_preservation, allow_hardlink_breakage, allow_suid_sgid_mutation, rescue knobs, overrides).

## Wiring Assessment

- `Switchyard::preflight()` emits per-action facts + summary.
- `Switchyard::apply()` fails closed on gating errors (E_POLICY) unless overridden.
- OwnershipOracle path honored under `strict_ownership`.
- Conclusion: wired correctly; gating exercised in both stages.

## Evidence and Proof

- Preflight and Apply tests in `cargo/switchyard/src/api.rs::tests` assert facts and rollback semantics.
- Preflight YAML stable sort via `rows.sort_by` for determinism.

## Gaps and Risks

- Some checks are best-effort (immutability via `lsattr`), may be inconclusive.
- Missing explicit schema validation for preflight rows.

## Next Steps to Raise Maturity

- Add golden fixtures for positive/negative gates; CI gate.
- Schema validation for preflight rows.

## Related

- SPEC v1.1 (policy defaults, preflight summary, fail-closed).
- `cargo/switchyard/src/preflight/checks.rs`.
