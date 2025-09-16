# TODO for three-command model (use, replace, restore)

Updated: 2025-09-16 12:33:43+02:00

This tracks the migration to the three-command model and the associated spec/UX changes for oxidizr-deb.

## Tasks

- [x] Documentation: align to three-command model (use, replace, restore)
  - [x] Update USER_FLOW.md
  - [x] Update SPEC/SPEC.md
  - [x] Update SPEC/DEBIAN_UX.md
  - [x] Update README.md
- [x] Status tips: reference `replace` instead of `make-permanent`
- [x] CLI surface: remove legacy PM-only subcommands (install-replacement, install-distro, remove-replacement, make-permanent) and add `replace`
  - [x] Update src/cli/args.rs
  - [x] Update src/cli/handler.rs (wire `replace`, update `restore` signature)
  - [x] Update src/commands/mod.rs
- [x] Implement `replace` subcommand
  - [x] Ensure replacement is installed/active (compose `use` semantics)
  - [x] Remove GNU packages via apt/dpkg under guardrails; dry-run prints commands
  - [x] Enforce live-root for PM ops in commit mode; dry-run works under fakeroot
- [x] Enhance `use` to ensure replacement install when missing (APT/DPKG)
  - [x] Live-root constraint for commit; dry-run prints commands
- [x] Enhance `restore` to ensure GNU packages installed and support `--keep-replacements`
  - [x] Live-root constraint for commit; dry-run prints commands
  - [x] Default behavior removes RS packages; keep with `--keep-replacements`
- [x] Tests: add BDD feature for replace (dry-run)
  - [x] tests/features/coreutils_replace_dry_run.feature
- [x] Cleanup: remove unused legacy source file `src/commands/make_permanent.rs`
  - Note: logically removed from the module tree and unused; kept on disk as a no-op placeholder to avoid accidental churn. Safe to delete physically in a follow-up cleanup.
- [x] Completions/help: clap auto-generates from args (no extra action needed)

## Completed changes (files touched)

- Docs
  - cargo/oxidizr-deb/USER_FLOW.md
  - cargo/oxidizr-deb/SPEC/SPEC.md
  - cargo/oxidizr-deb/SPEC/DEBIAN_UX.md
  - cargo/oxidizr-deb/README.md
- CLI & Commands
  - cargo/oxidizr-deb/src/cli/args.rs
  - cargo/oxidizr-deb/src/cli/handler.rs
  - cargo/oxidizr-deb/src/commands/mod.rs
  - cargo/oxidizr-deb/src/commands/use_cmd.rs (ensure install)
  - cargo/oxidizr-deb/src/commands/replace.rs (new)
  - cargo/oxidizr-deb/src/commands/restore.rs (ensure GNU + optional purge RS)
  - cargo/oxidizr-deb/src/commands/status.rs (tips updated)
- Tests
  - cargo/oxidizr-deb/tests/features/coreutils_replace_dry_run.feature (new)

