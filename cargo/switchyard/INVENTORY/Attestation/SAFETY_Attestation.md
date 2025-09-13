# Attestation

- Category: Safety
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Optional emission of an attestation bundle after successful apply (non-dry-run), signed by an injected `Attestor` implementation.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Provides post-apply cryptographic evidence | `cargo/switchyard/src/api/apply/mod.rs` builds bundle; `adapters/attest.rs` signs |
| Opt-in, no operational coupling if not configured | `Switchyard::with_attestor(...)` optional builder; omission allowed |
| Non-fatal: does not block apply | Apply attaches attestation on success; errors do not STOP stage |

| Cons | Notes |
| --- | --- |
| No default key management | Integrators must supply keys and `Attestor` impl |
| Minimal bundle schema | Fields limited to `plan_id`, executed count, `rolled_back` |
| No CI enforcement | Not validated by schema or gates yet |

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

## Feature Analytics

- Complexity: Low. Adapter trait + summary emission.
- Risk & Blast Radius: Minimal; non-fatal to apply. Risk is around key handling in integrator code.
- Performance Budget: Negligible; single signing op per apply.
- Observability: Fields emitted on `apply.result` under `attestation`.
- Test Coverage: Gap — add integration test exercising `with_attestor` success/failure cases.
- Determinism & Redaction: Attestation only in Commit; timestamps unaffected; bundle hash deterministic from bundle content.
- Policy Knobs: None in `Policy`; feature is opt-in via builder.
- Exit Codes & Error Mapping: None; attestation failure does not map to an exit code.
- Concurrency/Locking: Independent.
- Cross-FS/Degraded: N/A.
- Platform Notes: Crypto algorithm selection via `sig_alg` (default ed25519 implied in docs).
- DX Ergonomics: Simple builder method; integrators supply attestor.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| N/A | — | Attestation controlled via `with_attestor(...)` builder, not a policy flag |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| N/A | — | Attestation errors do not fail the stage |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `apply.result` | `attestation.sig_alg`, `attestation.signature`, `attestation.bundle_hash`, `attestation.public_key_id` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `tests/attestation_apply_success.rs` | success path (planned/verify) | attestation attached on success |
| `src/api/apply/mod.rs` | unit/integration harness (planned) | no-fail behavior on attestation errors |

## Gaps and Risks

- No built-in key management; algorithm default is `ed25519`.
- Bundle schema minimal; may evolve alongside SPEC.

## Next Steps to Raise Maturity

- Provide a sample ed25519 attestor with test keys; add golden for attestation fields.

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Optional attestation; minimal bundle | Non-fatal to apply; deterministic bundle hash | Basic integration test | None | Additive |
| Silver | Configurable algorithms; schema documented | Accurate fields, stable schema | Integration + docs | Inventory entry | Additive |
| Gold | Schema validation + CI gates; key rotation docs | Validated emission; operational guidance | Goldens + CI checks | CI gates | Additive |
| Platinum | Strong attestation guarantees and verification tooling | Verified signatures, rotation, monitoring | Property tests; compliance | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [ ] Policy knobs documented reflect current `policy::Policy` (N/A)
- [x] Error mapping and `exit_code` coverage verified (N/A)
- [x] Emitted facts fields listed and schema version up to date
- [ ] Determinism parity (DryRun vs Commit) verified in tests
- [ ] Goldens added/updated and CI gates green
- [ ] Preflight YAML or JSON Schema validated (where applicable)
- [ ] Cross-filesystem or degraded-mode notes reviewed (if applicable)
- [ ] Security considerations reviewed; redaction masks adequate
- [ ] Licensing impact considered (deps changed? update licensing inventory)
- [x] Maturity rating reassessed and justified if changed
- [ ] Observations log updated with date/author if noteworthy

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- PLAN/90-implementation-tiers.md (Apply → Gold/Platinum); SPEC/features/determinism_attestation.feature.
