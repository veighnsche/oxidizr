# Ownership and provenance

- Category: Safety
- Maturity: Silver
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Enforce strict ownership policy and record provenance (uid/gid/pkg where available) for targets. Used to reduce risk of hijacking and ensure controlled sources.

## Behaviors

- Queries filesystem metadata for uid/gid and (optionally) package provenance via an injected oracle.
- Attaches provenance fields to preflight rows for operator visibility.
- Enforces `strict_ownership` by adding STOPs when provenance does not meet policy.
- Leaves behavior advisory when no oracle is configured (best-effort enrichment only).

## Implementation

- Adapter: `cargo/switchyard/src/adapters/ownership/fs.rs::{FsOwnershipOracle, OwnershipOracle}` provides uid/gid via filesystem metadata.
- Policy: `cargo/switchyard/src/policy/config.rs` — `strict_ownership` toggles enforcement; provenance fields included in facts when available.
- Preflight: `cargo/switchyard/src/api/preflight/mod.rs` consults oracle to emit provenance (uid/gid) and compute `policy_ok` under strict ownership.

## Wiring Assessment

- `Switchyard` can be constructed with an `OwnershipOracle`. Preflight checks policy and attaches provenance.
- Apply respects preflight STOP decisions (E_POLICY) unless overridden.
- Conclusion: wired correctly; provenance is surfaced where supported.

## Evidence and Proof

- Unit: `adapters/ownership/fs.rs` basic behavior.
- Integration: preflight rows include ownership provenance fields; apply enforces E_POLICY for violations.

## Gaps and Risks

- Package ownership (`pkg`) not populated by default oracle; requires environment-specific oracle.
- Non-Unix platforms not supported by default oracle.

## Next Steps to Raise Maturity

- Provide package DB oracle example; expand tests for ownership edge cases (symlink, broken links).

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- PLAN/15-policy-and-adapters.md; ADR-0002 error strategy; ADR-0008 safepath-toctou.
