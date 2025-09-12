# Core Features for Operational Edge Cases

Status: Draft
Generated: 2025-09-12
Scope: Propose concrete core features to address edge cases identified across Switchyard. Each item includes edge-cases, proposal, scope (Library/Policy/CLI), affected modules, sketch, and acceptance criteria.

---

## 1) Sidecar v2: Integrity and Signing

- Edge cases
  - Tampered or corrupted sidecar JSON; stale or mismatched payload.
- Proposal
  - Extend sidecar schema to include `payload_hash` (sha256 of payload bytes) and optional `signature` over the sidecar payload (detached or embedded).
  - On restore, verify `payload_hash` when present; if mismatch, STOP unless `Policy.force_restore_best_effort` is true.
  - If `signature` present and an `Attestor` is configured, verify signature and emit verification results into audit facts.
- Scope
  - Library + Logging; backward-compatible (treat missing fields as legacy).
- Affected modules
  - `src/fs/backup.rs` (sidecar struct, write)
  - `src/fs/restore.rs` (read+verify)
  - `src/logging/audit.rs` (additional fields for verification results)
- Sketch
  - Add optional fields to `BackupSidecar`:
    - `payload_hash: Option<String>`
    - `signature: Option<String>` (base64)
  - Compute and write `payload_hash` on snapshot for file payloads.
  - During restore, when payload exists, recompute hash and compare.
  - If an `Attest` adapter is configured, sign on snapshot and verify on restore; record status in `apply.result`.
- Acceptance
  - Restore fails closed on hash mismatch unless best-effort flag is set.
  - Audit includes `payload_hash_ok` and `signature_ok` when applicable.

## 2) SafePath-first mutating APIs

- Edge cases
  - Path traversal or accidental writes outside allowed roots.
- Proposal
  - Adopt `types/safepath.rs::SafePath` in the signature of all mutating fs functions (swap, restore, backup). External APIs accept raw `Path`, but are immediately converted to `SafePath` against a configured root (policy’s allowed root) or fail.
- Scope
  - Library + API surface tightening (non-breaking by providing facade overloads).
- Affected modules
  - `src/fs/{swap.rs,restore.rs,backup.rs,atomic.rs,paths.rs}`
  - `src/types/safepath.rs`
  - `src/api/{apply,preflight}` (conversion points)
- Sketch
  - Introduce internal helpers `*_sp` that take `&SafePath`.
  - Facade converts `&Path`→`SafePath` using selected root (policy); reject on escape.
- Acceptance
  - Unit tests demonstrate rejection on `..` and absolute paths outside root.

## 3) Parent sticky-bit and ownership gate

- Edge cases
  - Sticky directories (`+t`, e.g., `/tmp`) produce `EACCES` on unlink/rename if caller doesn’t own.
- Proposal
  - Preflight/gating probe of parent dir: detect sticky bit and ownership/writability; STOP with actionable message.
- Scope
  - Library (preflight checks) + Policy gating.
- Affected modules
  - `src/preflight/checks.rs` (new `check_parent_sticky_and_ownership`)
  - `src/policy/gating.rs` (invoke check)
- Sketch
  - Read `st_mode` of parent (via `MetadataExt::mode()`); if sticky and not owned by effective uid, error.
- Acceptance
  - Clear STOP message referencing `chown`/`install` guidance.

## 4) Hardlink hazard preflight

- Edge cases
  - Restoring via `renameat` breaks hardlink relationships.
- Proposal
  - Detect `nlink > 1` for file targets and STOP unless `--force` (policy knob `allow_hardlink_breakage`). Emit explicit advisory.
- Scope
  - Preflight + Policy.
- Affected modules
  - `src/preflight/checks.rs`
  - `src/policy/config.rs` (optional knob)
  - `src/policy/gating.rs`
- Acceptance
  - Preflight row shows `nlink` and decision; apply stops without override.

## 5) Preservation Fidelity Extension (uid/gid/mtime/xattrs/acl)

- Edge cases
  - Mode-only restore loses ownership, timestamps, xattrs.
- Proposal
  - Optional preservation during restore based on policy and detected platform capabilities.
  - Extend sidecar with optional preservation dimensions when captured (uid,gid,mtime,xattrs keys); apply best-effort restore with capability checks.
- Scope
  - Library + Policy.
- Affected modules
  - `src/fs/backup.rs` (capture)
  - `src/fs/restore.rs` (apply)
  - `src/fs/meta.rs` (capability probe already present; may extend)
  - `src/policy/config.rs` (knobs per-dimension)
- Acceptance
  - When enabled and supported, restored file matches original mode+uid+gid+mtime (and selected xattrs).

## 6) Package Manager Activity Gate

- Edge cases
  - Concurrent PM runs racing with Switchyard operations.
- Proposal
  - Preflight adapter that detects active package manager locks (e.g., pacman `/var/lib/pacman/db.lck`, dpkg/apt locks). STOP unless `override_preflight`.
- Scope
  - Preflight + Policy.
- Affected modules
  - `src/preflight/checks.rs` (new `check_package_manager_activity`)
  - `src/policy/gating.rs` (invoke for actions touching common system paths)
- Acceptance
  - Gate emits specific PM lock evidence in notes.

## 7) Lock Manager v2: Path-scoped locks + stale lock handling

- Edge cases
  - Coarse process-wide lock harms concurrency; stale locks not differentiated from contention.
- Proposal
  - Introduce path-scoped lock names (e.g., hash of parent directory) to reduce contention.
  - Add optional lock lease/heartbeat file to detect and break stale locks after bounded wait with explicit operator consent.
- Scope
  - Adapters (locking) + Policy (timeout semantics/lease enabled).
- Affected modules
  - `src/adapters/lock/{mod.rs,file.rs}`
  - `src/policy/config.rs` (knobs: `lock_scope=process|parent`, `lock_lease_ms: Option<u64>`)
- Acceptance
  - Unit tests cover timeout vs. lease expiry; path-scoped locking allows parallel non-overlapping operations.

## 8) Plan-of-Truth Registry for RELINK

- Edge cases
  - RELINK needs an intended mapping beyond “prior state”.
- Proposal
  - Maintain a small registry file under a configurable state dir (e.g., `/var/lib/switchyard/registry.jsonl`) recording the last successful `EnsureSymlink` actions per target with `source`, `plan_id`, and timestamp.
  - RELINK prefers registry intent over sidecar’s prior topology.
- Scope
  - Logging/State + CLI.
- Affected modules
  - `src/logging/` (new small writer/reader) or `src/adapters/` state helper
  - CLI (doctor/relink) consumer
- Acceptance
  - Registry is append-only; doctor reads it and proposes relink plans deterministically.

## 9) Doctor: Artifact Hygiene (orphaned backups/sidecars)

- Edge cases
  - Left-over artifacts without live targets; growth and confusion.
- Proposal
  - Scanner that lists orphaned `.bak`/`.meta.json` pairs per directory; offers remediation (delete, archive) based on Policy/flags.
- Scope
  - CLI + optional Library helper.
- Affected modules
  - `src/fs/backup.rs` (enumeration functions already exist)
  - New helper `fs::backup::list_pairs(target_dir, tag)`
- Acceptance
  - Report includes counts per directory; optional `--fix` prunes per retention policy.

## 10) Degraded Mode Rules (Path-sensitive)

- Edge cases
  - Some paths can tolerate EXDEV degraded fallback; others must fail-closed.
- Proposal
  - Extend Policy to support path matchers for degraded allowance (e.g., allow in `/opt`, disallow in `/usr/bin`).
- Scope
  - Policy + Apply handlers.
- Affected modules
  - `src/policy/config.rs` (e.g., `degraded_allow_rules: Vec<PathBuf>` and deny rules)
  - `src/api/apply/handlers.rs` (set fields accordingly; enforce before swap)
- Acceptance
  - Apply fails on EXDEV under disallowed paths; emits `E_EXDEV`.

## 11) FSYNC Health Thresholds and Telemetry

- Edge cases
  - Slow fsync indicates storage or mount problems.
- Proposal
  - Policy-driven fsync thresholds with escalating decisions: warn → soft-fail (rollback) → hard STOP, depending on path criticality.
- Scope
  - Policy + Apply handlers + Audit.
- Affected modules
  - `src/constants.rs` (threshold defaults)
  - `src/api/apply/handlers.rs` (decision logic)
  - `src/logging/audit.rs` (emit `fsync_slow=true` details)
- Acceptance
  - Facts include threshold crossing; policy can trigger rollback on exceeding hard threshold.

## 12) Offline Rescue CLI

- Edge cases
  - System unbootable; need to restore from backups from a live USB.
- Proposal
  - A minimal `switchyard-rescue` tool that can enumerate and restore sidecar/payload pairs under a given root without full environment (no smoke/locks). Operates read-then-rename with conservative checks.
- Scope
  - Separate binary (CLI) building on the same library structs; or a static, tiny helper.
- Affected modules
  - Reuse `src/fs/{backup,restore}`; small new crate/binary.
- Acceptance
  - Can restore a broken coreutils swap from a live environment following documented steps.

## 13) PATH Overshadow Check

- Edge cases
  - A new symlink in one directory shadows another binary earlier/later in PATH, causing surprise.
- Proposal
  - Preflight check that resolves `which <target-name>` across PATH and warns/STOPs based on policy if the planned target would be overshadowed or overshadow others.
- Scope
  - Preflight + Policy.
- Affected modules
  - `src/preflight/checks.rs` (new helper)
  - `src/policy/config.rs` (knobs)
- Acceptance
  - Preflight notes include `path_shadowing=true` with details.

## 14) Deterministic IDs and Safer Defaults

- Edge cases
  - Non-deterministic plan/action IDs; accidental commits.
- Proposal
  - Library helper for `UUIDv5` plan and action IDs from stable inputs; default Apply mode to DryRun in the top-level CLI unless `--commit`.
- Scope
  - Types + CLI.
- Affected modules
  - `src/types/ids.rs` (already close; add v5 helpers)
  - CLI default flags
- Acceptance
  - Repeated planning yields stable ids; CLI prints “Dry run by default. Use --commit to mutate.”

## 15) SUID/SGID Risk Gate

- Edge cases
  - Mutating suid/sgid binaries is high risk.
- Proposal
  - Preflight probe of `st_mode & S_ISUID/S_ISGID`; STOP unless `--allow-suid` policy knob is set.
- Scope
  - Preflight + Policy.
- Affected modules
  - `src/preflight/checks.rs` (new probe)
  - `src/policy/config.rs`
  - `src/policy/gating.rs`
- Acceptance
  - Preflight row shows `suid=true` or `sgid=true` and stops by default.

---

## References

- Code: `src/fs/{backup.rs,restore.rs,swap.rs,atomic.rs,meta.rs,paths.rs,mount.rs}`
- Code: `src/policy/{config.rs,gating.rs}`
- Code: `src/api/apply/{mod.rs,handlers.rs}`
- Code: `src/logging/audit.rs`
- Types: `src/types/{safepath.rs,ids.rs}`
- Docs: `EDGE_CASES_AND_BEHAVIOR.md`, `RETENTION_STRATEGY.md`, `PRESERVATION_FIDELITY.md`, `LOCKING_STRATEGY.md`

## Round 1 Peer Review (AI 4, 2025-09-12 15:16 CET)

- **Claims Verified:**
  - Sidecar v2 proposal for integrity and signing is not yet implemented. Current `BackupSidecar` structure in `src/fs/backup.rs` does not include `payload_hash` or `signature` fields.
  - SafePath-first mutating APIs are partially implemented. `SafePath` structure and validation are present in `src/types/safepath.rs`, but not fully integrated into all mutating FS functions as proposed.
  - Parent sticky-bit and ownership gate is not implemented. No evidence of `check_parent_sticky_and_ownership` in `src/preflight/checks.rs`.
  - Hardlink hazard preflight is not implemented. While `nlink` is referenced in some FS operations, there is no specific preflight check for hardlink breakage in `src/preflight/checks.rs`.
- **Key Citations:**
  - `src/fs/backup.rs`: Current `BackupSidecar` structure without integrity fields.
  - `src/types/safepath.rs`: Implementation of `SafePath` for path validation.
  - `src/preflight/checks.rs`: Current preflight checks without sticky-bit or hardlink-specific gates.
- **Summary of Edits:**
  - Added citations to confirm the current state of proposed features in the codebase.
  - Clarified that several proposed features (Sidecar v2, sticky-bit checks, hardlink checks) are not yet implemented, aligning the document with the current code reality.
  - No major content changes were necessary as the document is a proposal draft, but updated to reflect current implementation status.

Reviewed and updated in Round 1 by AI 4 on 2025-09-12 15:16 CET

## Round 2 Gap Analysis (AI 3, 2025-09-12 15:33+02:00)

- Invariant: Filesystem mutations are restricted to intended, safe paths.
- Assumption (from doc): The document proposes that all mutating filesystem functions should use a `SafePath` type to prevent path traversal vulnerabilities (`CORE_FEATURES_FOR_EDGE_CASES.md:35-51`). The Round 1 review noted this was "partially implemented."
- Reality (evidence): A search confirms that while `SafePath` exists in `src/types/safepath.rs`, it is not used in the signatures of the core mutating functions within the `src/fs/` module (e.g., `swap.rs`, `restore.rs`, `backup.rs`). These functions still accept raw `&Path` arguments, bypassing the proposed safety layer.
- Gap: The core library functions that perform mutations are not protected against path traversal attacks, as the `SafePath` wrapper is not enforced at the point of execution. This contradicts a fundamental consumer expectation of operational safety.
- Mitigations: Refactor the internal `*_sp` helpers as proposed, and ensure all public-facing fs functions in the `api` module perform the `Path` -> `SafePath` conversion immediately, failing closed if the path is outside the allowed root. This aligns with `SPEC.md#safepath`.
- Impacted users: All users, as the lack of path safety at the library's core creates a potential security vulnerability.
- Follow-ups: This is a high-severity security gap. It should be prioritized for implementation in Round 4.

- Invariant: The tool will not perform high-risk operations without explicit user consent.
- Assumption (from doc): The document proposes a preflight gate to detect and stop operations on SUID/SGID binaries unless explicitly allowed by a policy knob (`CORE_FEATURES_FOR_EDGE_CASES.md:229-242`).
- Reality (evidence): A search for `S_ISUID` or `S_ISGID` within `src/preflight/checks.rs` confirms that no such check is implemented. The library will operate on SUID/SGID binaries without any warning or special gating.
- Gap: A critical security gate is missing. Consumers, especially system administrators, would expect a tool modifying system files to be aware of and cautious with privileged binaries. The current implementation violates the principle of least surprise.
- Mitigations: Implement the proposed preflight check (`check_suid_sgid_risk`) in `preflight/checks.rs` and integrate it into the policy gating logic in `policy/gating.rs`. Add a corresponding policy knob like `allow_suid_sgid_mutation`.
- Impacted users: Administrators and security-conscious users. Unwittingly modifying an SUID binary could have significant security implications.
- Follow-ups: This is a high-severity security gap. It should be prioritized for implementation in Round 4.

Gap analysis in Round 2 by AI 3 on 2025-09-12 15:33+02:00
