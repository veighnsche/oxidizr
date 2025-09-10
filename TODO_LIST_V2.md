# TODO_LIST_V2

This list prioritizes remediation work from the fresh audit of `src/` (entrypoint `src/main.rs`). Each task references relevant modules and suggests concrete changes.

## P0 — Immediate Safety and Transaction Guarantees

- __Transaction rollback across multi-target operations__
  - Implement a transactional guard in `experiments/util.rs::create_symlinks()` that records per-target actions (backup created, target replaced) and automatically restores on the first failure.
  - Strategy: build an in-memory journal and use RAII to roll back unless an explicit `commit()` is called at the end.
  - Touch points: `symlink/ops.rs::{replace_file_with_symlink, restore_file}`, `experiments/*::enable()` callers.

- __Preflight plan and diff__
  - Compute and print a preflight plan of intended changes before mutating the system.
  - Add `--preflight` (on by default unless `--assume-yes`) to show a diff-like summary: target path, current state (file/symlink->dest), planned state (symlink->source).
  - Touch points: `experiments/*::enable()`, `experiments/util.rs::create_symlinks()` and `list_targets()`.

- __Strict metadata backup & restore__
  - Back up and restore timestamps (atime/mtime), owner (uid/gid), and permissions for regular files; for symlinks, preserve link target and lstat metadata when possible.
  - Extend `symlink/ops.rs` copy/backup logic to set timestamps (via `libc::utimensat`), and optionally record xattrs/ACLs (see P1 items) to sidecar `.meta` file.

## P1 — Auditability and Supply Chain

- __Before/after cryptographic hashes in audit logs__
  - For each target, compute `sha256` (or `blake3`) of the real file contents before change and of the resulting symlink destination binary after change, and include in `AuditFields`.
  - Add fields: `before_hash`, `after_hash` (extend `logging/audit.rs::AuditFields`).
  - Touch points: `experiments/util.rs::create_symlinks()`, `symlink/ops.rs`.

- __Record actor, provenance, versions, and exit codes__
  - Actor: add `uid`, `euid`, `user` into audit envelope using `nix::unistd` and (optionally) `users` crate.
  - Provenance: record package owner (`pacman -Qo`) and binary version (`<bin> --version | head -n1`) for linked sources.
  - Exit codes are already captured for package operations; propagate to all external command calls.
  - Touch points: `logging/audit.rs`, `system/worker/packages.rs`, `experiments/*::discover_*()`.

- __Append-only, tamper-evident audit log__
  - Ensure open with O_APPEND semantics and 0640 perms; document log rotation.
  - Add chained hashing: include previous line hash in each record (`prev_hash`) to create a verifiable chain.
  - Touch points: `logging/init.rs::AuditMakeWriter`, `logging/audit.rs`.

- __Supply chain: signature verification and SBOM fragments__
  - For official packages, rely on pacman’s signature verification; record verification result in audit (`pacman -Qi` fields like `Validated By`).
  - For AUR builds, record makepkg checksum verification results in audit; optionally verify downloaded artifacts against upstream signatures.
  - Generate minimal SBOM fragment: package name, version, source repo, installed files subset relevant to linked applets.
  - Touch points: `system/worker/packages.rs`, `experiments/*` discovery.

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

- __Rescue toolset and initramfs access__
  - Add an optional `oxidizr-arch install-rescue` subcommand to install a static `busybox` (or ensure presence) and record its path in state.
  - Document and/or implement hooks to ensure essential commands available in initramfs (documentation plus optional mkinitcpio hook).

- __Post-restore verifiers for non-sudo experiments__
  - After `disable`, verify restored targets are regular files (not symlinks) for coreutils/findutils/checksums, mirroring `sudors.rs` checks.
  - Touch points: `experiments/coreutils.rs`, `experiments/findutils.rs`, `experiments/checksums.rs`.

## P2 — Determinism and Minimal Trusted Surface

- __Determinism guardrails__
  - Reduce reliance on ambient env: standardize locale (`LC_ALL=C`), clear nonessential env for external calls, and make progress/UI non-randomized.
  - Consider a `--deterministic` flag to enforce stricter behavior (timestamps normalization in logs, stable ordering of operations).

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
  - Add optional jitter/backoff and cancellation hooks during pacman DB lock waits to improve UX; surface clearer progress messages while waiting.
  - Touch points: `system/worker/packages.rs::wait_for_pacman_lock_clear()`.

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
