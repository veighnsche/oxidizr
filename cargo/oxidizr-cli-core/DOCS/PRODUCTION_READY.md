# oxidizr-cli-core — Production Readiness

This document explains why the `oxidizr-cli-core` crate is suitable for production use, and the guarantees and limits you should expect.

## Scope and responsibilities

- Provides shared CLI helpers used by `oxidizr-arch` and `oxidizr-deb`.
- Key surfaces:
  - `api::build_api(policy, lock_path) -> Switchyard<JsonlSink, JsonlSink>` in `src/lib.rs`
  - Cross-distro coverage helpers in `src/coverage2.rs`
  - Simple prompt gating in `prompts::should_proceed()`
- This crate does not directly mutate the filesystem; it wires and configures the Switchyard API and offers safe applet coverage helpers for higher-level CLIs.

## Quality gates (passing)

- Clippy: zero warnings for this crate
  - Command: `cargo clippy -p oxidizr-cli-core --all-targets --no-deps -- -D warnings`
  - Recent fixes:
    - Removed unit-struct `::default()` calls in `src/lib.rs` builder
    - Replaced `iter().cloned().collect()` with `.to_vec()` in `src/coverage2.rs`
- Tests: all unit tests pass
  - Command: `cargo test -p oxidizr-cli-core`
  - Coverage helpers tested in `src/coverage2.rs` (happy path, missing applets, intersection behavior, fallback behavior)
- Docs build: rustdoc + docs.rs cfg succeed
  - Command: `RUSTDOCFLAGS="--cfg docsrs" cargo doc -p oxidizr-cli-core --no-deps`
- Packaging sanity validated
  - Command: `cargo package -p oxidizr-cli-core --list --allow-dirty`

## Operational characteristics

- Deterministic and conservative helpers
  - `discover_applets_with_allow()` aggressively validates output; falls back to the static allow-list when probing fails or is implausible (e.g., <3 applets).
  - `resolve_applets_for_use()` intersects distro-provided commands with replacement-supported applets when available; otherwise returns the replacement set (or static fallback).
- Non-interactive safety by default
  - `prompts::should_proceed()` becomes non-blocking in non-TTY contexts (CI), honoring CLI `--assume-yes` when set.
- Switchyard wiring defaults favor safety and observability
  - Builder uses file-backed JSONL sinks (`JsonlSink`) for facts and audit streams.
  - File lock manager (`FileLockManager`) ensures single-writer semantics across processes.
  - `FsOwnershipOracle` and `DefaultSmokeRunner` are installed to back policy checks and smoke.

## Dependencies and compatibility

- Minimal runtime deps: `switchyard-fs`, `atty`.
- MSRV pinned: `rust-version = "1.89"` in `Cargo.toml`.
- No `unsafe` code in this crate.
- Unix-focused (Switchyard is Unix/FS centric). Thread-safety derives from Switchyard; this crate holds no global mutable state.

## Security notes

- No privilege escalation or direct syscalls; only composes Switchyard.
- Prompts are TTY-gated and can be disabled via `--assume-yes` by the consumer CLI.
- Audit/Facts default to JSONL file sinks to keep an append-only operational record.

## Known limitations (pre-1.0)

- SemVer pre-1.0: minor version bumps may include breaking changes.
- Coverage heuristics rely on replacement binary introspection; some vendors may vary their `--list`/`--help` formats.
- This crate assumes the consumer CLI enforces path safety and commit/dry-run semantics; refer to Switchyard SPEC for FS invariants.

## Conclusion

Within its scope, `oxidizr-cli-core` meets production-readiness expectations: strong linting, tests, docs, deterministic helpers with safe fallbacks, and a conservative Switchyard wiring that prioritizes locking and auditability.
