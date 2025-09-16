# oxidizr-deb Safety Manifesto

Updated: 2025-09-16

## Mission

Never break basic shell behavior. oxidizr-deb exists to safely switch between distro implementations and Rust replacements of core utilities while preserving a working system and a one-step restore path at all times.

## What “basic shell behavior” means here

- The user can run core commands (representative examples): `ls`, `cat`, `cp`, `mv`, `find`, `xargs`, and `sudo`.
- PATH-resolved commands remain available and executable; ownership/mode constraints for security-sensitive tools (e.g., `sudo` setuid 4755) are respected when active.
- If replacement activation fails or a post-apply smoke check fails, the system rolls back to the prior working state.

## Guarantees & invariants

- Provider availability invariant:
  - There must always be at least one functional provider for `coreutils` and one for `sudo` installed. oxidizr-deb refuses operations that would leave zero providers.
- Atomic, reversible filesystem swaps:
  - Uses the Switchyard engine to perform `plan → preflight → apply` with SafePath boundaries, atomic operations, and backups.
- One-step restore:
  - Backups are created prior to mutation; `restore` can switch back to GNU/stock binaries deterministically.
- Sudo hardening:
  - On commit, the replacement must be `root:root` with mode `4755` (setuid). Otherwise commit fails closed.

## Operational guardrails

- No standalone package-manager commands are exposed.
  - All APT/DPKG operations happen inside `use`, `replace`, or `restore`.
- Locks are respected:
  - If dpkg/apt locks are detected, commit refuses with a friendly diagnostic.
- Live-root gating for APT/DPKG mutations:
  - Commit-time package manager changes require `--root=/`.
  - Tests and experimentation can run under `--root /tmp/fakeroot` (or chroot) with dry-run previews; no live mutations occur.
- Dry-run by default:
  - All commands preview intended changes unless `--commit` is provided.
- `replace` safety:
  - Ensures the Rust replacement is installed and active first, then removes GNU packages under guardrails and invariants.
- `restore` safety:
  - Ensures GNU packages are installed and preferred; by default removes RS packages, or keeps them installed but de-preferred with `--keep-replacements`.

## Why test jobs avoid purging on a live root

Although oxidizr-deb is designed to protect basic shell behavior, test harnesses deliberately avoid performing `apt-get purge` on a live `/` to:

- Keep CI environments deterministic and disposable (no post-test cleanup required).
- Avoid non-deterministic network/package availability flakiness during destructive PM operations.
- Honor least-privilege: most CI containers lack persistent admin state for realistic `apt` mutation tests.

Instead, tests:

- Use a fakeroot (`--root /tmp/fakeroot`) to exercise file swaps safely.
- Use dry-run for `replace` to validate APT commands and guardrails (locks, live-root checks) without destructive effects.

## Risk register (and how we avoid it)

- Manual misuse outside the CLI: running raw `apt purge coreutils` on a live root without ensuring a replacement is active can break shell behavior.
  - Mitigation: Do not do this. Use `oxidizr-deb replace <package>` which ensures the replacement is installed and active before removal and enforces provider invariants and lock checks.
- Environmental drift (locks, partial upgrades): committing while the package manager is busy can cause undefined outcomes.
  - Mitigation: Lock detection fails closed; the user must retry after the ongoing operation finishes.

## Operator guidance

- Validate with dry-run; then `--commit` under a controlled maintenance window if operating on a real system.
- Prefer testing inside VMs/containers or a chroot for early validation (`--root /target/chroot`).
- Use `status` to inspect active state and next-step tips for `restore` and `replace`.

---

When in doubt, remember: The product goal is to safely swap core utilities without breaking the user’s ability to use their shell, and to always preserve a one-step path back to the prior working state.

---

## Switchyard engine responsibilities and boundaries

oxidizr-deb is a thin CLI over the Switchyard safety engine. The engine is responsible for safe, atomic, reversible filesystem changes; the CLI is responsible for package manager orchestration and Debian/Ubuntu UX.

- Engine pipeline
  - Plan → Preflight → Apply.
  - Plans consist of link requests (to switch to replacements) and restore requests (to switch back to GNU/stock).
  - Preflight enforces policy gates and SafePath boundaries; STOPs prevent any mutation.
  - Apply performs TOCTOU-safe, atomic operations with backups and rollback on error.

- Safety primitives (see `cargo/switchyard/src/fs/`)
  - SafePath: all mutating paths must resolve under `--root`.
  - Atomic symlink/file swap (see `fs/atomic.rs`): uses dirfd + unlinkat/openat patterns to avoid TOCTOU; replaces a regular file with a symlink to the replacement or adjusts existing symlinks atomically.
  - Backups (see `fs/backup/`): sidecar snapshots are created before mutation to enable one-step restore; indexed and prunable.
  - Rollback (see `api/apply/rollback.rs`): failed applies restore prior state.

- Adapters wired by oxidizr-deb
  - Lock manager: `FileLockManager` with default path `<root>/var/lock/oxidizr-deb.lock`.
  - Smoke runner: `DefaultSmokeRunner` executes minimal deterministic checks in commit mode; failures trigger rollback.
  - Ownership oracle: `FsOwnershipOracle` validates ownership/mode where required.

- Policy gates (package-level defaults)
  - Coreutils & Findutils: strict link topology; degraded EXDEV fallback disallowed by default.
  - Sudo: production hardening; commit guarded by ownership/mode requirements (setuid 4755, `root:root`).
  - Policy violations detected during Preflight STOP the apply.

- Observability
  - The engine exposes JSON audit fields and apply summaries; oxidizr-deb uses JSON sinks by default and may enable file-backed JSONL via feature flag.

- Explicit boundaries
  - The engine never invokes APT/DPKG.
  - Package manager mutations are orchestrated by the CLI before and/or after engine phases in the high-level flows (`use`, `replace`, `restore`).

## Integration plan alignment

This manifesto aligns with the integration plan in `cargo/oxidizr-deb/PLAN/01-switchyard-integration.md`:

- Dry-run by default; `--commit` required.
- Locking errors surface with friendly diagnostics; bounded wait under the engine’s lock manager.
- Minimal deterministic smoke checks; rollback on smoke failure.
- SafePath enforced for all mutating paths; out-of-root writes rejected.
- Preflight STOPs on policy violations; no mutation occurs when STOPped.
- Fetch/verify responsibility
  - In this Debian variant, oxidizr-deb ensures replacements via APT/DPKG with repository trust; the engine receives concrete paths only.
  - Offline `--use-local` is supported for testing.

## Roles recap (who does what)

- Engine (Switchyard): safe planning and atomic filesystem mutation; backups; rollback; policy enforcement; no package manager.
- CLI (oxidizr-deb): Debian/Ubuntu UX; lock checks for apt/dpkg; ensure replacements installed/active; remove/purge GNU under guardrails; present dry-run previews; provide `restore` and `status` UX.
