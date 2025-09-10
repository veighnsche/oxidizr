# TASK: Rewrite and refactor the Switchyard specification to v1.3

## Goal

Refactor the current specification into a **perfectly coherent, OS-agnostic, reproducible** document:

- Preserve all approved guarantees and requirements
- Remove duplication/contradictions
- Make it AI-readable and human-auditable
- Keep Switchyard strictly a **library crate** (no CLI/distro coupling)

## Inputs (read-only)

- SPEC.md (current v1.2 spec)
- SWITCHYARD_SPECIFICATIONS.md (previous details)
- AUDIT_AND_LOGGING.md, AUDIT_CHECKLIST.md
- SAFETY_MEASURES.md, CLEAN_CODE.md, VERBOSITY.md
- AUR_COMPLIANCE.md (only mine for generic supply-chain requirements; do NOT hardcode Arch/AUR specifics into the core spec)

## Output

- Overwrite `SPEC.md` as **“Switchyard Specification (Reproducible v1.3)”**
- Single file, self-contained, with a short Change Log header (v1.2 → v1.3)

## Non-Goals / Do NOT

- Do NOT add distro-specific or package-manager specifics into the core spec
- Do NOT add CLI flags/docs (Switchyard is a library)
- Do NOT invent new features; consolidate and clarify existing approved ones
- Do NOT weaken security posture or determinism guarantees

## Structure (exact section order)

0. Domain & Purpose (library crate, single responsibility, OS-agnostic statement)
1. Main Guarantees (plain-language bullets; 10 items max)
2. Normative Requirements (RFC-2119, numbered IDs; grouped under guarantees)
   - Atomicity (A*)
   - Rollback (R*)
   - Safety Preconditions (S*)
   - Determinism (D*)
   - Locking (L*)
   - Rescue (RC*)
   - Observability/Audit (O*)
   - Conservatism & Modes (C*)
   - Health Verification (H*)
   - Filesystems & Degraded Mode (F*)
3. Public Interfaces
   - API surface (plan, preflight, apply, plan_rollback_of)
   - Adapters (OwnershipOracle, LockManager, PathResolver, Attestor, SmokeTestRunner)
   - SafePath (construction, invariants, TOCTOU-safe syscall sequence)
4. Preflight Diff (normative schema + deterministic rules)
5. Audit Facts (JSON Schema v1; fields, masking, versioning, attestation bundle layout)
6. Error Taxonomy & Exit Codes (stable identifiers + TOML mapping; no CLI behavior)
7. Formal Safety Model (invariants, properties; pointers to TLA+/property tests)
8. Acceptance Tests (BDD scenarios; include EXDEV fallback and auto-rollback-on-failure)
9. Operational Bounds (fsync ≤ 50ms, plan size default, resource notes)
10. Filesystems & Degraded Mode (supported semantics; `allow_degraded_fs` policy + telemetry)
11. Smoke Tests (minimal suite, exact args, auto-rollback semantics)
12. Golden Fixtures & CI Gate (byte-identical, zero-SKIP)
13. Schema Versioning & Migration (dual-emit policy on bump)
14. Thread-Safety (Send+Sync stance; concurrency expectations under LockManager)
15. Security Requirements Summary (consolidated from checklist/logging docs)
16. Change Log (v1.2 → v1.3 delta)

## Mandatory Content to Preserve/Clarify

- **Atomic swap protocol:** temp symlink → `renameat` → `fsync(parent)`; EXDEV fallback is copy+fsync+rename
- **Rollback:** reverse-order automatic rollback on any apply failure; idempotent; partial-rollback facts if any step fails
- **SafePath everywhere:** no mutating API accepts raw PathBuf; TOCTOU-safe parent handle with `O_DIRECTORY|O_NOFOLLOW`
- **Determinism:** UUIDv5 plan/action IDs over normalized inputs; dry-run facts byte-identical to real after redactions (timestamps zeroed or monotonic deltas)
- **Locking:** production requires LockManager; bounded wait + timeout → `E_LOCKING`; record `lock_wait_ms`
- **Rescue profile:** at least one fallback toolset reachable on PATH; preflight verifies fallback availability
- **Audit facts (v1):** required fields (ts, schema_version=1, plan_id, action_id, stage, decision, severity, path, current_kind, planned_kind, hash_alg=sha256, before_hash, after_hash, attestation{sig_alg=ed25519, signature, bundle_hash, public_key_id}, provenance{origin: repo|aur|manual, helper, uid, gid, pkg, env_sanitized, allow_aur}, preservation{owner,mode,timestamps,xattrs,acls,caps}, preservation_supported, exit_code, duration_ms, lock_wait_ms)
- **Secret masking:** explicit rule that no secrets may appear in free-form fields; tests enforce masking
- **Preflight Diff:** stable ordering and keys; schema includes action_id, path, current_kind, planned_kind, policy_ok, provenance{uid,gid,pkg}, notes[]
- **Health verification:** minimal smoke suite (ls, cp, mv, rm, ln, stat, readlink, sha256sum, sort, date) with exact args; any failure → auto-rollback unless explicitly disabled by policy
- **FS matrix & degraded mode:** ext4/xfs/btrfs/tmpfs; telemetry `degraded=true` when fallback used and policy allows; otherwise fail
- **Golden fixtures & zero-SKIP CI:** fixtures for plan/preflight/apply/rollback; CI fails on non-identical or SKIP
- **Error taxonomy:** stable identifiers (E_POLICY, E_PREFLIGHT_ENV, E_PREFLIGHT_OWNERSHIP, E_LOCKING, E_ATOMIC_SWAP, E_EXDEV_FALLBACK, E_BACKUP, E_RESTORE, E_OBSERVABILITY, E_SMOKE_FAIL) with TOML exit mapping
- **OS-agnostic stance:** no distro/vendor tooling in spec; supply-chain content framed through adapters and generic provenance; keep AUR/repo references only as *origin labels* in provenance (no policy coupling)

## Style & Formatting Rules

- Use RFC-2119 keywords (MUST/SHOULD/MAY) consistently
- Number requirements (e.g., REQ-A1, REQ-R2, REQ-S5…) and keep them stable
- Keep prose concise; avoid duplication; move rationale to short notes
- Provide all machine-readable blocks (YAML/TOML/JSON) as minimal, valid examples
- No external links; spec must be self-contained

## Coherence & Consistency Pass (required)

- Collapse duplicates across SPEC.md and SWITCHYARD_SPECIFICATIONS.md
- Resolve any contradictions in Safety, Determinism, and Audit sections
- Ensure each “Main Guarantee” maps to at least one numbered normative requirement and at least one acceptance test
- Ensure every adapter is referenced only via interfaces, not implementation details

## Acceptance Criteria (the spec rewrite is DONE when…)

- [ ] SPEC.md header shows **v1.3** with a v1.2→v1.3 change log
- [ ] “Main Guarantees” ≤ 10 bullets, each mapped to ≥1 REQ-*
- [ ] All REQ-* are unique, numbered, and referenced at least once by tests or schemas
- [ ] JSON Schema (facts v1), YAML (preflight), and TOML (error codes) are present, valid, and minimal
- [ ] SafePath invariants and TOCTOU syscall sequence are explicit and non-ambiguous
- [ ] Determinism rules (UUIDv5 inputs, timestamp redaction) are explicit
- [ ] Locking production requirement stated; bounded wait/timeout behavior specified
- [ ] Smoke suite listed with exact args and rollback semantics
- [ ] FS degraded-mode policy and telemetry clearly specified
- [ ] No distro/CLI specifics anywhere in the core spec

## Sanity Checks Before Save

- Run a quick schema lint (structural validity) on JSON/YAML/TOML blocks
- Re-scan the spec to ensure OS-agnostic language (no pacman/AUR instructions)
- Confirm glossary terms are either defined or removed
