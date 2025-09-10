# ADR Template

- Title: SafePath type and TOCTOU-safe syscall sequence
- Status: Accepted
- Date: 2025-09-11

## Context

Mutating public APIs must not accept raw `PathBuf` and must enforce SafePath semantics. All mutations must follow the TOCTOU-safe sequence: open parent with `O_DIRECTORY|O_NOFOLLOW` → `openat` final component → `renameat` into place → `fsync(parent)`.

## Decision

- Introduce `SafePath` constructed via `SafePath::from_rooted(root, candidate)`; reject `..` after normalization and prevent root escape.
- Require `SafePath` on all mutating APIs. Non-mutating APIs that accept raw paths immediately normalize into `SafePath`.
- For filesystem mutations, implement the normative syscall sequence; include fsync ≤50ms after rename per bounds.

## Consequences

+ Strong defense-in-depth against traversal and race attacks.
+ Consistency across all mutating operations.
- Slight complexity in path handling and testing.

## Links

- `cargo/switchyard/SPEC/SPEC.md` §§ 2.3, 3.3, 9
- `cargo/switchyard/PLAN/10-architecture-outline.md`
