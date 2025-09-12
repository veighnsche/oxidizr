# Switchyard Code Smell and Clean Code Audit

Scope: cargo/switchyard/ (library code and embedded tests)
Sources: zrefactor/documantation/code_smell.md, docs/CLEAN_CODE.md

---

## Executive Summary

Overall, `cargo/switchyard/` adheres well to the project’s Clean Code and safety posture:

- Unsafe code is forbidden and not used.
- Mutations follow TOCTOU-safe `rustix` sequences.
- Public mutating APIs operate on `SafePath` types.
- Error handling is explicit with `thiserror` and stable error IDs.
- Observability is structured via the `FactsEmitter`/Audit envelope.

Key opportunities to improve:

- Duplicate and lengthy logic in `src/fs/restore.rs` and backup scanners in `src/fs/backup.rs`.
- Sidecar schema/kind fields are stringly-typed; consider typed enums with serde for safety.
- No `tracing` instrumentation; current audit pipeline is good, but adding `tracing` spans would ease developer diagnostics without changing emitted facts.

---

## Method

- Applied checklist from `zrefactor/documantation/code_smell.md`.
- Cross-checked principles from `docs/CLEAN_CODE.md`.
- Searched the codebase with ripgrep for common smells and policy markers (`unwrap`, `expect`, `unsafe`, `println!`, `dbg!`, `todo!`, `RefCell`, `Mutex`, `tracing::`).
- Manually reviewed hotspots: `src/fs/restore.rs`, `src/fs/backup.rs`, `src/api.rs`, `src/api/apply/handlers.rs`, `src/types/safepath.rs`, `src/logging/audit.rs`.

---

## Clean Code Compliance (highlights)

- Clarity over cleverness: good separation (`fs/`, `api/`, `logging/`, `types/`).
- Explicit dependencies and side-effect isolation: public FS mutators accept `SafePath`; heavy I/O is localized under `src/fs/` and adapters.
- Types encode invariants: `SafePath` enforces root-bound paths (`src/types/safepath.rs`).
- Honest error handling: `thiserror`-based `ApiError` and `types::errors::Error` with mapping (`src/api/errors.rs`).
- Determinism & idempotence: UUIDv5 namespace and zeroed timestamps on dry-run are present in policy and audit flow (`src/constants.rs`, `src/logging/audit.rs`).
- Atomicity & consistency: `open_dir_nofollow` + `openat` + `renameat` + `fsync` used consistently (e.g., `src/fs/backup.rs`, `src/fs/restore.rs`).
- Observability: minimal JSON facts with envelope (`src/logging/audit.rs`), provenance, error IDs and exit codes.
- Safe concurrency: no shared global state; file locks via `adapters/lock/file.rs` with bounded wait.
- Minimal public surface: internal structs are `pub(crate)` where appropriate; top-level re-export marked deprecated to guide callers (`src/lib.rs`).
- Lints and CI posture: `#![forbid(unsafe_code)]` and clippy denies unwrap/expect in non-test code (`src/lib.rs`).

Gaps and nits:

- No `tracing` usage detected; consider adding lightweight spans around API entry points to complement audit facts for dev-time debugging.
- Some long functions/duplication (see Findings) increase maintenance surface and cognitive load.

---

## Code Smell Checklist Results

- Unnecessary clones/to_owned: none obviously problematic in hot paths; conversions in `fs/backup.rs` for `OsString` are appropriate.
- Rc<RefCell>/Arc<Mutex>: not used in library code. Tests use `Mutex` to collect facts.
- Unsafe blocks: none. `#![forbid(unsafe_code)]` present in `src/lib.rs`.
- `.unwrap()`/`.expect()` in production: confined to test modules and test files. Non-test code maps errors explicitly.
- Logging/telemetry: structured via audit; no stray `println!`/`dbg!` found.
- Magic numbers: centralized under `src/constants.rs` (e.g., `FSYNC_WARN_MS`, `LOCK_POLL_MS`).
- Overly long functions / god modules: hotspots identified below.
- Duplicate logic: present in restore/backup scanning paths.

Search evidence (representative):

- `unsafe`: `rg -n "\bunsafe\b" cargo/switchyard/src -S` → 0 hits (policy enforced by `src/lib.rs`).
- `unwrap(`/`expect(` in production: confined to tests/`#[cfg(test)]` blocks; verify with `rg -n "\.(unwrap|expect)\(" cargo/switchyard/src -S | rg -v "#\[cfg\(test\)\]|tests"`.
- `println!`, `dbg!`, `todo!`: none in library code.
- `RefCell`, `Mutex<`: none in library code; `Mutex` used in tests to collect events.
- `tracing::`: none in `cargo/switchyard/src`.

---

## Notable Findings and Recommendations

1. Duplicate and lengthy restore logic

- Evidence: `src/fs/restore.rs` contains two large functions, `restore_file` (~290 lines) and `restore_file_prev` (~260 lines), with near-identical control flow and repeated branches for `file`/`symlink`/`none`/fallback.
- Risks: Increased chance of divergence/bugs; harder to audit and evolve sidecar logic; raises cognitive load.
- Recommendation:
  - Extract shared helpers for: reading sidecar, integrity verification, rename-via-fd, restoring file mode, removing payloads, and "ensure absent" logic.
  - Parameterize the backup selection strategy (latest vs previous) as a small enum or by passing a selector function.
  - Target a reduction to ~80–120 lines per public function by delegating to private helpers.

2. Backup scanner duplication and consistency

- Evidence: `src/fs/backup.rs` implements both `find_latest_backup_and_sidecar` and `find_previous_backup_and_sidecar` with similar directory scanning and timestamp extraction logic.
- Risks: Subtle bugs if one path is updated without the other; repeated string prefix/suffix math.
- Recommendation:
  - Introduce a single `scan_backups(target, tag) -> Vec<(timestamp, base_path)>` utility used by both callers.
  - Define the filename prefix/suffix pieces as `const` values to avoid drift.

3. Stringly-typed sidecar schema and kinds

- Evidence: `BackupSidecar` stores `schema`, `prior_kind` as `String` (`src/fs/backup.rs`). Callers compare against string literals (e.g., `"file"`, `"symlink"`, `"none"`).
- Risks: Typos and refactoring hazards are not caught at compile time.
- Recommendation:
  - Replace with enums like `enum PriorKind { File, Symlink, None, Other }` and `enum SidecarSchema { V1, V2 }` using `serde(rename = "...")` for on-disk compatibility.

4. Improve developer observability with `tracing`

- Evidence: No `tracing` spans found; all external observability goes through `FactsEmitter`/audit.
- Value: `tracing` spans with fields at API boundaries (`plan`, `preflight`, `apply`, `rollback`, `prune`) greatly help during local debugging, without changing emitted audit facts.
- Recommendation:
  - Add optional `tracing` spans in `src/api.rs` public methods (feature-gated if needed). Keep audit as the product-facing contract.

5. Minor retention policy clarity

- Evidence: In `prune_backups` (`src/fs/backup.rs`), comment says “keep first `limit-1` after newest? Interpret as total retain = limit”, while code retains at most `limit` total items (newest always retained; `idx >= limit` pruned).
- Recommendation:
  - Clarify doc comments and tests to lock semantics: e.g., "count_limit=N retains up to N newest backups (including the newest). N=0 still retains the newest by safety policy." If different behavior desired, adjust accordingly and codify.

6. Dead code allowance markers

- Evidence: `#[allow(dead_code)]` present for some audit helpers (e.g., `emit_preflight_fact` in `src/logging/audit.rs`).
- Recommendation:
  - Confirm callers or remove deprecated variants after migration to the extended variant `emit_preflight_fact_ext`.

---

## Positive Patterns Worth Keeping

- `SafePath` (`src/types/safepath.rs`) as the public surface for mutating operations.
- TOCTOU-safe directory handle pattern across FS mutations.
- Explicit error ID and exit-code mapping (`src/api/errors.rs`).
- Centralized constants (`src/constants.rs`) to eliminate magic numbers in code paths.
- Minimal facts envelope with provenance and dry-run redaction (`src/logging/audit.rs`).

---

## Suggested Next Steps (PR-sized)

- PR1: Factor restore helpers and unify `restore_file`/`restore_file_prev` shared logic.
- PR2: Introduce `scan_backups()` and deduplicate latest/previous scanners.
- PR3: Define `PriorKind` and `SidecarSchema` enums with serde; migrate callers.
- PR4: Add opt-in `tracing` spans at API boundaries; keep emitted facts unchanged.
- PR5: Tighten comments/tests around `prune_backups` count semantics; add table tests.
- PR6: Remove or route remaining `#[allow(dead_code)]` functions to the canonical variants.

Each PR should include:

- Unit tests for new helpers and invariants.
- No behavior change unless explicitly called out (e.g., retention semantics doc).
- Docs: module-level purpose/invariants where logic is split out.

---

## Appendix: File references

- `src/lib.rs`: forbids unsafe, clippy settings.
- `src/fs/restore.rs`: restore logic; duplication hotspot.
- `src/fs/backup.rs`: snapshot, prune, sidecar read/write; scanner duplication and retention semantics.
- `src/api.rs`: API facade and prune logic with audit emission.
- `src/api/apply/handlers.rs`: per-action apply/restore handlers with facts, error IDs.
- `src/types/safepath.rs`: root-bound, normalized paths.
- `src/logging/audit.rs`: minimal facts envelope, redaction, provenance.
