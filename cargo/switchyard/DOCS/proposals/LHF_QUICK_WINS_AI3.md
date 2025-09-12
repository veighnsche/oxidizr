# Low-Hanging Fruit (Quick Wins) — AI 3
Generated: 2025-09-12 16:33:01+02:00
Author: AI 3

## Quick Win 1: Correct CLI Integration Guide

- **Type:** Docs
- **Change:** Update `CLI_INTEGRATION_GUIDE.md` to remove references to the non-existent `prune_backups` function and clarify that core `fs` functions do not yet enforce `SafePath` usage, despite recommendations. Add a note explaining these are planned features.
- **Scope (files):** `CLI_INTEGRATION_GUIDE.md`
- **Why now:** The guide is currently misleading for developers. This change prevents confusion and aligns the documentation with the current reality of the codebase. It directly addresses a gap identified in my Round 2 meta-review.
- **Time estimate:** < 1 hour
- **Risk:** Low
- **Acceptance criteria:** The `CLI_INTEGRATION_GUIDE.md` no longer contains incorrect information about `prune_backups` and `SafePath` enforcement.
- **Evidence:** My Round 2 meta-review notes on `CLI_INTEGRATION_GUIDE.md`.

## Quick Win 2: Add CI Check for Tests Touching System Paths

- **Type:** CI / Test
- **Change:** Add a script to CI that greps test files for string literals that look like absolute system paths (e.g., `"/usr/bin"`, `"/etc/"`). The script should fail the build if such paths are found outside of designated mock or integration setup modules. This encourages writing hermetic tests.
- **Scope (files):** A new CI workflow script (e.g., `.github/workflows/hermetic_tests.yml`).
- **Why now:** Enforces a key principle from `CODING_STANDARDS.md` (tests should be self-contained) and prevents test flakiness caused by dependencies on system state.
- **Time estimate:** a few hours
- **Risk:** Low (might have some initial false positives to tune).
- **Acceptance criteria:** The CI pipeline includes a new check that fails when a test file directly references a system path.
- **Evidence:** `CODING_STANDARDS.md` (states tests should prefer temp directories), `TEST_COVERAGE_MAP.md` (shows where tests live).

## Quick Win 3: Deprecate Top-Level `policy::rescue` Re-export

- **Type:** Refactor / API Design
- **Change:** Add a `#[deprecated]` attribute to the `pub use policy::rescue;` re-export in `src/lib.rs`. The deprecation note should guide users to the canonical path `switchyard::policy::rescue`.
- **Scope (files):** `src/lib.rs`
- **Why now:** This is a simple cleanup identified in `BACKWARDS_COMPAT_SHIMS.md` and `REEXPORTS_AND_FACADES.md`. It tightens the public API facade with minimal effort and encourages idiomatic usage.
- **Time estimate:** < 1 hour
- **Risk:** Low
- **Acceptance criteria:** Using `switchyard::rescue` produces a compile-time deprecation warning.
- **Evidence:** `BACKWARDS_COMPAT_SHIMS.md` (Round 1 peer review), `src/lib.rs:L21`.

## Quick Win 4: Add Module-Level Docs for `fs` and `policy`

- **Type:** Docs
- **Change:** Add a one-paragraph summary at the top of `src/fs/mod.rs` and `src/policy/mod.rs` explaining their core responsibilities and how they relate to other parts of the crate (e.g., `fs` provides atomic primitives, `policy` provides gating logic).
- **Scope (files):** `src/fs/mod.rs`, `src/policy/mod.rs`
- **Why now:** Improves code navigation and discoverability for new contributors, a goal mentioned in `CODING_STANDARDS.md`.
- **Time estimate:** < 1 hour
- **Risk:** Low
- **Acceptance criteria:** The specified files have clear, concise module-level documentation.
- **Evidence:** `CODING_STANDARDS.md` (recommends clear module structure).

---

Proposals authored by AI 3 on 2025-09-12 16:33:01+02:00
