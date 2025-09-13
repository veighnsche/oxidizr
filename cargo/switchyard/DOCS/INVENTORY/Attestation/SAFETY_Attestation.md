# Attestation

- Category: Safety
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Optional emission of an attestation bundle after successful apply (non-dry-run), signed by an injected `Attestor` implementation.

## Behaviors

- On successful Commit (not DryRun), constructs a minimal bundle with `plan_id`, executed count, and `rolled_back`.
- Calls `Attestor::sign()` to produce a signature; does nothing if no attestor is configured.
- Emits `attestation` fields on `apply.result` summary: `sig_alg`, `signature` (base64), `bundle_hash` (sha256), `public_key_id`.
- Never fails the apply stage on attestation errors; omission is allowed by design.

## Implementation

- Trait: `cargo/switchyard/src/adapters/attest.rs::{Attestor, Signature}` provides an interface for signing bundles.
- Integration: `cargo/switchyard/src/api/apply/mod.rs` builds a minimal bundle (plan_id, executed count, rolled_back) and adds `attestation` to `apply.result` extra on success.

## Wiring Assessment

- `Switchyard` can be constructed with an attestor; when present, apply attaches attestation fields (`sig_alg`, `signature`, `bundle_hash`, `public_key_id`).
- Conclusion: wired for optional use; no default attestor provided.

## Evidence and Proof

- Integration verified in `api/apply/mod.rs`; test `tests/attestation_apply_success.rs` exercises success path.

## Gaps and Risks

- No built-in key management; algorithm default is `ed25519`.
- Bundle schema minimal; may evolve alongside SPEC.

## Next Steps to Raise Maturity

- Provide a sample ed25519 attestor with test keys; add golden for attestation fields.

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- PLAN/90-implementation-tiers.md (Apply → Gold/Platinum); SPEC/features/determinism_attestation.feature.
