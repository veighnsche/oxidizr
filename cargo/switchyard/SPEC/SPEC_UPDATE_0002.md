# SPEC_UPDATE_0002 (2025-09-11)

Title: Rescue Gating (require_rescue), Degraded Symlink Semantics, and Preflight Row Clarifications
Status: Proposed
Applies-to: SPEC/SPEC.md v1.1

## Summary

This update clarifies three normative areas:

1. Rescue policy and minimal verification
2. Degraded symlink semantics under cross-filesystem conditions (EXDEV)
3. Preflight row field set alignment (preservation fields)

---

## 1) Rescue Policy and Minimal Verification (SPEC §2.6 Rescue)

- Introduce `Policy.require_rescue` as a normative flag. When `true`, preflight MUST verify that at least one functional fallback toolset is available on PATH (GNU or BusyBox). Failing this check MUST cause preflight to STOP (fail-closed) unless explicitly overridden by policy.
- Minimal verification (interim): PASS if `busybox` is present on PATH. Otherwise, require a subset of GNU core tools to be present (cp, mv, rm, ln, stat, readlink, sha256sum, sort, date, ls) with threshold ≥6/10. This verification MUST be deterministic and MUST NOT execute external commands during preflight.
- Apply MUST refuse to proceed when rescue verification fails and `require_rescue=true`, unless `override_preflight=true` is set.

Rationale: Ensures a basic recovery path without introducing flakiness or side effects during checks.

---

## 2) Degraded Symlink Semantics (SPEC §2.10 Filesystems & Degraded Mode)

- Clarify that for symlink replacement across filesystems (EXDEV), the engine cannot achieve atomic copy+fsync+rename on the symlink itself. The engine SHALL use an unlink+symlink best-effort degraded fallback when `allow_degraded_fs=true`.
- Facts MUST record `degraded=true` when the degraded path is used. When `allow_degraded_fs=false`, the engine MUST fail with `exdev_fallback_failed` (E_EXDEV) and perform no visible change.

Rationale: Aligns spec language with filesystem semantics for symlink replacement while preserving safety and observability guarantees.

---

## 3) Preflight Row Fields (SPEC §4 Preflight Diff)

- The preflight row schema includes preservation capability fields and support flag:
  - `preservation` object with booleans: `owner`, `mode`, `timestamps`, `xattrs`, `acls`, `caps`.
  - `preservation_supported` boolean.
- Rows MUST be deterministically ordered by (`path`, `action_id`). Dry-run output MUST be byte-identical to commit output after redaction.

Rationale: Ensures spec-aligned visibility into preservation capabilities prior to apply.

---

## Migration & Compatibility

- Policy surfaces gain `require_rescue` (default false). Implementations MAY provide interim, deterministic verification as above.
- No schema-breaking changes are introduced; fields already present in v1.1 are clarified. Implementations SHALL validate and emit preservation fields.

## Security & Privacy

- Rescue checks MUST not execute external commands during preflight. Implementations SHOULD avoid leaking PATH contents in facts; if reported, they MUST be sanitized/redacted per policy.

## References

- SPEC/SPEC.md §§2.6, 2.10, 4
- PLAN/65-rescue.md implementation notes
