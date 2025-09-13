# SafePath (capability-scoped paths)

- Category: Safety
- Maturity: Silver
- Owner(s): <owner>
- Last reviewed: 2025-09-13
- Next review due: 2025-10-13
- Related PR(s): <#NNNN>

## Summary

`SafePath` ensures mutating APIs operate within a caller-provided root, preventing path traversal and root escape. Aligns with SPEC Reproducible v1.1: “SafePath for all mutating APIs”.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Prevents root escape and `..` traversal | `cargo/switchyard/src/types/safepath.rs::from_rooted()` returns `ErrorKind::Policy` on escape; tests: `rejects_dotdot`, `rejects_absolute_outside_root` |
| Normalizes `.` segments deterministically | `from_rooted()` ignores `Component::CurDir`; test `normalizes_curdir_components` |
| Unified across stages and fs atoms | Usage in `api/plan.rs`, `api/apply/handlers.rs`, `fs/swap.rs`, `fs/restore.rs` ensures targets are `SafePath` |
| Clear observability of relative path | `SafePath::rel()` preserved for facts/logging context |

| Cons | Notes |
| --- | --- |
| Requires callers to supply an absolute `root` | Enforced by `assert!(root.is_absolute())` in `from_rooted()` |
| No serde/FFI serialization yet | Internal-only type; would require schema if exposed |
| Plan-level single-root assumption | Heterogeneous per-action roots not enforced; see “Gaps and Risks” |

## Behaviors

- Rejects `..` (`ParentDir`) segments and normalizes `.` (`CurDir`).
- Constrains all mutating operations to a caller-provided root via `SafePath::as_path()`.
- Preserves a stable relative component with `SafePath::rel()` for observability.
- Ensures API stages (`plan`, `preflight`, `apply`) and fs atoms accept `SafePath` targets, not raw `Path`.

## Implementation

- Core type: `cargo/switchyard/src/types/safepath.rs::SafePath`
  - Validates components (rejects `ParentDir`, normalizes `CurDir`), preserves `rel()` under a fixed `root`.
- Usage:
  - API types and handlers use `SafePath` for sources/targets:
    - `cargo/switchyard/src/api/plan.rs` builds actions with `SafePath`.
    - `cargo/switchyard/src/api/apply/handlers.rs` receives `Action::{EnsureSymlink, RestoreFromBackup}` with `SafePath`.
  - Filesystem atoms accept `SafePath`:
    - `cargo/switchyard/src/fs/swap.rs::replace_file_with_symlink(source: &SafePath, target: &SafePath, ...)`.
    - `cargo/switchyard/src/fs/restore.rs::restore_file(target: &SafePath, ...)`.

## Wiring Assessment

- Entry points: `Switchyard::plan` produces `Plan` with `SafePath` members; `preflight` and `apply` operate on those.
- Adapters/Policy: OwnershipOracle and Policy operate on `SafePath`-wrapped targets.
- Stages: Preflight checks and Apply mutations always go through `SafePath`→absolute via `as_path()`.
- Conclusion: wired correctly; no mutating path API accepts raw `Path` for targets.

## Evidence and Proof

- Unit tests: `cargo/switchyard/src/types/safepath.rs` (`rejects_dotdot`, `accepts_absolute_inside_root`, `rejects_absolute_outside_root`, `normalizes_curdir_components`).
- Apply/Preflight tests indirectly prove integration using `SafePath` construction.

## Feature Analytics

- Complexity: Low. ~100 LOC in `types/safepath.rs`; touched modules: `api/plan.rs`, `api/apply/handlers.rs`, `fs/swap.rs`, `fs/restore.rs`.
- Risk & Blast Radius: High leverage, low risk. Misconfiguration of `root` could scope too broadly or narrowly; guardrails: `strip_prefix` + `starts_with(root)` checks prevent escape.
- Performance Budget: Negligible. Pure path manipulation and component iteration; not a hot path bottleneck.
- Observability: Relative path via `SafePath::rel()` can be surfaced in facts; envelope `path` field is constrained under the provided root (see `logging/audit.rs`).
- Test Coverage: Unit tests in `types/safepath.rs`; integration coverage through apply/preflight flows. Gap: property tests for normalization/idempotence.
- Determinism & Redaction: Deterministic normalization; relies on stage emitters for redaction/timestamps (`logging/redact.rs`).
- Policy Knobs: Path gating primarily via `policy::Policy::{allow_roots, forbid_paths}`; SafePath guards root-relative resolution.
- Exit Codes & Error Mapping: `ErrorKind::Policy` → `ApiError::PolicyViolation` → `E_POLICY` (10) via `api/errors.rs`.
- Concurrency/Locking Touchpoints: None directly; apply lock acquired separately per `adapters/lock` and `api/apply/mod.rs`.
- Cross-FS/Degraded Behavior: N/A. Managed by atomic swap module/policy (`allow_degraded_fs`).
- Platform Notes: Uses `std::path` semantics; designed for Unix-like paths in this project. Non-Unix components (e.g., Windows prefixes) are rejected as “unsupported component.”
- DX Ergonomics: Safer API surface by construction; call sites avoid raw `Path` for mutations.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `allow_roots: Vec<PathBuf>` | `[]` | Limits mutations to specific roots; SafePath resolution must be under a permitted root. |
| `forbid_paths: Vec<PathBuf>` | `[]` | Blocks mutations when target starts with any forbidden prefix; enforced in preflight/apply. |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_POLICY` | `10` | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}`; `ErrorKind::Policy` → `ApiError::PolicyViolation` |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `apply.attempt`/`apply.result` | `path`, `plan_id`, `dry_run`, `error_id?` | Minimal Facts v1 (`SPEC/audit_event.schema.json`) |
| `preflight.row` | `path`, `policy_ok` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/types/safepath.rs` | `rejects_dotdot` | `..` traversal rejected |
| `src/types/safepath.rs` | `accepts_absolute_inside_root` | absolute inside-root accepted, rel preserved |
| `src/types/safepath.rs` | `rejects_absolute_outside_root` | absolute outside-root rejected |
| `src/types/safepath.rs` | `normalizes_curdir_components` | `.` segments normalized deterministically |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Basic `SafePath::from_rooted` with `..` rejection and inside-root check | No root escape; deterministic normalization of `.` | Unit tests for dotdot/curdir; basic integration | None required | Additive |
| Silver (current) | Enforced across all mutating APIs and fs atoms; indexable `rel()` for observability | Fail-closed on escape; deterministic across stages; mapped to `E_POLICY` on violations | Unit + integration coverage; redaction/determinism parity via stage emitters | Inventory entry; docs | Additive |
| Gold | Property tests; schema validation for facts referencing path; broader invariants documented | Stronger guarantees of normalization/idempotence; schema-validated facts | Property tests; golden fixtures involving path fields | CI gates for schema/goldens | Additive |
| Platinum | Formal model/property proofs; multi-platform validation; continuous compliance | Formally specified invariants; cross-platform behavior ensured | Model/prop tests; platform matrix CI | Continuous compliance and reporting | Additive |

## Gaps and Risks

- No serialization/deserialization for `SafePath` yet (not required internally).
- Does not yet enforce per-target SafePath roots heterogeneously in a plan (plan-level root assumed).

## Next Steps to Raise Maturity

- Add property tests for normalization/idempotence.
- Integrate schema validation if/when `SafePath` crosses FFI/CLI boundaries.

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Policy knobs documented reflect current `policy::Policy`
- [x] Error mapping and `exit_code` coverage verified
- [ ] Emitted facts fields listed and schema version up to date
- [ ] Determinism parity (DryRun vs Commit) verified in tests
- [ ] Goldens added/updated and CI gates green
- [ ] Preflight YAML or JSON Schema validated (where applicable)
- [ ] Cross-filesystem or degraded-mode notes reviewed (if applicable)
- [ ] Security considerations reviewed; redaction masks adequate
- [ ] Licensing impact considered (deps changed? update licensing inventory)
- [x] Maturity rating reassessed and justified if changed
- [ ] Observations log updated with date/author if noteworthy

## Observations log

- 2025-09-13 — <author> — Added Feature Analytics, Pros & Cons, and Maturity model; verified error mapping to `E_POLICY`.

## Change history

- 2025-09-13 — <author> — Augmented entry per Inventory template; added Behaviors and analytics; PR: <#NNNN>

## Related

- SPEC v1.1 (SafePath requirement).
- `cargo/switchyard/src/fs/paths.rs::is_safe_path()` (auxiliary guard).
