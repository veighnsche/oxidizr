# TODO_LIST_V2

This list prioritizes remediation work from the fresh audit of `src/` (entrypoint `src/main.rs`). Each task references relevant modules and suggests concrete changes.

## P0 — Immediate Safety and Transaction Guarantees

- __Profiles layout + single active pointer flip__ (replaces multi-target journaling)
  - Why: Touching N symlinks one-by-one increases blast radius and rollback complexity; a single `renameat` of an active profile pointer is safer and smaller.
  - Actionable Steps:
    - Create profiles: `/usr/lib/oxidizr-arch/profiles/{gnu,uutils}/bin` and `active -> <profile>`.
    - Link `/usr/bin/*` to `.../active/bin/<applet>`; during flips, update `active` via atomic `renameat`.
    - Migrate current state to profiles; keep `restore_file()` for emergency repair paths.
  - References: `symlink/ops.rs::{atomic_symlink_swap, restore_file}`, `experiments/coreutils.rs::{enable,disable}`, `state/mod.rs`.

- __Preflight plan and diff__
  - Compute and print a preflight plan of intended changes before mutating the system.
  - Add `--preflight` (on by default unless `--assume-yes`) to show a diff-like summary: target path, current state (file/symlink->dest), planned state (symlink->source).
  - Touch points: `experiments/*::enable()`, `experiments/util.rs::create_symlinks()` and `list_targets()`.

- __Metadata backup & restore (file vs symlink split)__
  - Why: Full xattrs/ACL/timestamps on symlinks are not meaningful here; preserving linkness + target is sufficient for symlinks.
  - Actionable Steps:
    - Regular files: after `fs::copy`, restore owner/mode and set mtime/atime via `utimensat`.
    - Symlinks: keep current link-aware backup/restore; skip deep metadata unless a policy requires it.
  - References: `symlink/ops.rs::{replace_file_with_symlink, restore_file, atomic_symlink_swap}`.

## P1 — Auditability and Supply Chain

- __Selective hashing for audit (changed or untrusted sources)__
  - Why: Hashing every applet each run is expensive and unnecessary when official repo provenance is recorded.
  - Actionable Steps:
    - Hash only targets mutated during the current run; always hash when `query_file_owner()` returns `None` or the source is under `$HOME`/non-repo.
    - Cache by `(inode, mtime, size)` to avoid re-hashing unchanged paths; add `--hash` to force full hashing.
    - Extend `AuditFields` with optional `before_hash`, `after_hash`.
  - References: `experiments/util.rs::create_symlinks()`, `symlink/ops.rs`, `system/worker/packages.rs::query_file_owner()`.

- __Record actor, provenance, versions, and exit codes__
  - Actor: add `uid`, `euid`, `user` into audit envelope using `nix::unistd` and (optionally) `users` crate.
  - Provenance: record package owner (`pacman -Qo`) and binary version (`<bin> --version | head -n1`) for linked sources.
  - Exit codes are already captured for package operations; propagate to all external command calls.
  - Touch points: `logging/audit.rs`, `system/worker/packages.rs`, `experiments/*::discover_*()`.

- __Append-only, tamper-evident audit log__
  - Ensure open with O_APPEND semantics and 0640 perms; document log rotation.
  - Why: Per-line chained hashes complicate rotation and are brittle on partial loss.
  - Lean replacement: per-operation detached signature (`audit-<op_id>.jsonl` + `.sig`).
  - Actionable Steps:
    - Buffer audit events per `op_id` and flush to `audit-<op_id>.jsonl` at end of run.
    - Sign with Ed25519 to `audit-<op_id>.jsonl.sig`; add `oxidizr-arch audit verify --op <op_id>`.
  - References: `logging/init.rs::AuditMakeWriter`, `logging/audit.rs::audit_event_fields`.

- __Supply chain: signature verification and SBOM fragments__
  - For official packages, rely on pacman’s signature verification; record verification result in audit (`pacman -Qi` fields like `Validated By`).
  - For AUR builds, record makepkg checksum verification results in audit; optionally verify downloaded artifacts against upstream signatures.
  - Generate minimal SBOM fragment: package name, version, source repo, installed files subset relevant to linked applets.
  - Touch points: `system/worker/packages.rs`, `experiments/*` discovery.

- __AUR opt-in policy (repo-first)__
  - Why: Implicit AUR invocation increases trust surface; require explicit operator consent.
  - Actionable Steps:
    - Gate AUR paths behind `--allow-aur` and `--aur-user`; otherwise emit the exact helper command for the operator.
  - References: `system/worker/packages.rs::{install_package, ensure_aur_preflight}`.

- __Mask sensitive fields in structured audit events__
  - Extend `logging/audit.rs::AuditFields` pipeline (or pre-call sanitization) to mask secrets in `audit_event_fields()` similar to `audit_event()`.
  - Review all field call sites to ensure tokens/credentials are never logged.
  - Touch points: `logging/audit.rs`, all `audit_event_fields(...)` call sites.

- __Sanitize environment for external commands (AUR helpers, pacman)__
  - Provide a minimal, controlled environment (e.g., `LC_ALL=C`, scrub PATH additions) for `Command` invocations and `su -c` strings, to reduce nondeterminism and risk.
  - Touch points: `system/worker/packages.rs`, `system/worker/distro.rs`.

## P1 — Recovery and Health

- __Automatic smoke tests and rollback triggers__
  - After link phase, run a small smoke suite (e.g., `ls --version`, `cp --version`, `mv --version`, `rm --version`, `find --version`, `xargs --version`).
  - If any test fails, trigger transaction rollback via the journal (P0) and surface a clear error.
  - Touch points: `experiments/*::enable()`, new `experiments/util.rs::run_smoke_tests()`.

- __GNU escape path + canary shell (replace BusyBox bundle)__
  - Why: GNU binaries remain installed; adding a static BusyBox increases attack surface without proportional benefit.
  - Actionable Steps:
    - Provide a stable GNU escape profile and document `export PATH=/usr/lib/oxidizr-arch/profiles/gnu/bin:$PATH`.
    - Add a non-mutating `oxidizr-arch canary --shell` that spawns a shell with GNU PATH for diagnostics (design only here).
  - References: `experiments/coreutils.rs::{enable,disable}`, `symlink/ops.rs`, `state/mod.rs`.

- __Post-restore verifiers for non-sudo experiments__
  - After `disable`, verify restored targets are regular files (not symlinks) for coreutils/findutils/checksums, mirroring `sudors.rs` checks.
  - Touch points: `experiments/coreutils.rs`, `experiments/findutils.rs`, `experiments/checksums.rs`.

## P2 — Determinism and Minimal Trusted Surface

- __Determinism guardrails__
  - Why: A global "deterministic mode" is over-broad for this tool.
  - Actionable Steps:
    - Standardize locale (`LC_ALL=C`), sanitize env for external commands, and ensure stable ordering of operations and log fields.
  - References: `system/worker/packages.rs` (Command env), `logging/audit.rs`.

- __Dependencies review__
  - Evaluate replacing `which` crate with small PATH search; consider removing heavy deps where possible.
  - Consider using `time` instead of `chrono` if footprint matters; evaluate `indicatif` usage behind a feature flag.

- __Preflight for required system tools__
  - Add preflight checks (with remediation guidance) for required tools like `pacman`, `pacman-conf`, and `lsattr`; fail-fast with clear errors.
  - Touch points: `system/worker/packages.rs`, `system/worker/distro.rs`, `system/fs_checks.rs`.

## P2 — UX and CLI Semantics

- __Dry-run-first posture__
  - Consider making `--dry-run` the default unless `--assume-yes` is provided.
  - At minimum, enhance dry-run to show the full preflight plan with hashes and ownership checks, and to simulate smoke tests.

- __Owner and trust policies__
  - Enforce explicit policy toggles in help text; expand error messages with remediation guidance and explicit commands.

- __Explicit experiment ordering gating__
  - When both findutils and coreutils are selected, enforce user-visible gating (or automatic reordering) with explicit messages; avoid relying on implicit order.
  - Touch points: `src/cli/handler.rs`, `experiments/mod.rs::all_experiments()`.

- __Log exact per-applet path when falling back to PATH__
  - Improve visibility by logging the resolved path for each applet when not using a unified binary.
  - Touch points: `experiments/coreutils.rs::discover_applets()`, `experiments/findutils.rs::discover_applets()`.

- __Broader IO error mapping with actionable remediation__
  - Extend error mapping for common IO failures across symlink operations and worker edges (e.g., permission denied, missing parent dirs) with suggested fixes.
  - Touch points: `symlink/ops.rs`, `system/worker/fs_ops.rs`.

- __Pacman lock handling ergonomics__
  - Why: Elaborate backoff/cancel logic is overkill; keep behavior simple and predictable.
  - Actionable Steps:
    - Maintain a simple bounded wait with small jitter and clear progress messages while waiting for the lock.
  - References: `system/worker/packages.rs::wait_for_pacman_lock_clear()`.

## P2 — Tests and CI

- __Unit and e2e tests for rollback and smoke tests__
  - Add tests for the transaction journal and rollback correctness.
  - Add e2e smoke tests that intentionally break one applet to verify rollback triggers.
  - Integrate with existing container-based runner under `test-orch/`.

- __Idempotency tests for enable/disable/relink__
  - Add integration tests using temporary roots to repeat enable/disable and verify no drift; include relink after package upgrades.
  - Touch points: `experiments/*`, `state/mod.rs`, test harness.

- __Unit test for pacman hook body generation__
  - Verify flags and formatting of the generated hook body to prevent regressions.
  - Touch points: `system/hook.rs`.

- __Aggregate restore errors in sudo-rs flows__
  - In `sudors.rs` rollback, collect multiple restore/removal errors and report a consolidated error for better diagnostics.
  - Touch points: `experiments/sudors.rs`.

## P3 — Documentation

- __Recovery playbook and side-effects__
  - Document rollback procedure, what files are touched, and how to recover in degraded environments.
  - Document pacman hook behavior and state file locations.
  
- __Exit code mapping__
  - Document the exit code table from `src/error.rs::Error::exit_code()` in the README or a dedicated doc.

## Detected Overkill (from code scan)

- [experiments/coreutils.rs::discover_applets]
  - Why overkill: Over-broad fallback search paths (e.g., `/usr/lib/cargo/bin/coreutils/`, `/usr/lib/cargo/bin/`) add surface and latency without clear benefit.
  - Safer, smaller replacement: Prefer unified binary or configured bin directory; otherwise fall back to PATH only.
  - Actionable Steps:
    - Trim candidates to `{bin_directory, unified binary, PATH}` and log final resolution.
  - References: `src/experiments/coreutils.rs::discover_applets()`.
  - Priority: P2 (simplifies discovery, reduces surprises).

- [system/worker/packages.rs::install_package]
  - Why overkill: Implicit AUR fallback for `uutils-findutils-bin` runs helpers automatically.
  - Safer, smaller replacement: Repo-first; require `--allow-aur` + `--aur-user` for AUR, else print exact helper command.
  - Actionable Steps:
    - Add CLI gating and condition the AUR path on opt-in; improve audit logs to record decision.
  - References: `src/system/worker/packages.rs::{install_package, ensure_aur_preflight}`.
  - Priority: P1 (reduces trusted surface for supply chain).

- [system/worker/packages.rs]
  - Why overkill: Dependency on `which` crate for a small number of PATH lookups.
  - Safer, smaller replacement: Implement a tiny PATH search helper and remove the external crate usage.
  - Actionable Steps:
    - Add internal `path_search::which()` and replace call sites in worker/experiments.
  - References: `src/system/worker/packages.rs` (use sites), `experiments/*::discover_applets()`.
  - Priority: P2 (reduce dependencies).

- [system/worker/packages.rs]
  - Why overkill: Chatty info logs like “Expected: … Received: …” during normal success paths.
  - Safer, smaller replacement: Demote to DEBUG; keep INFO for decisions and user-facing prompts.
  - Actionable Steps:
    - Review `tracing::info!` calls and adjust levels accordingly.
  - References: `src/system/worker/packages.rs` success branches.
  - Priority: P2 (reduce noise, simpler UX).
