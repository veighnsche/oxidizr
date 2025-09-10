# ADR Template

- Title: API stability and semver commitments
- Status: Accepted
- Date: 2025-09-11

## Context

The crate exposes a public API (library, not CLI). Consumers rely on stability. SPEC ties behavior to audit/log schemas and policies; API surface must evolve conservatively and predictably.

## Decision

- Adopt semantic versioning (semver). Public API covered by semver is the `api` module exports and re-exports in `lib.rs`.
- Introduce stability policy:
  - 0.x: minor versions may include breaking changes with clear CHANGELOG entries.
  - ≥1.0: breaking changes require MAJOR bump and migration notes.
- Deprecation policy:
  - Mark functions/types `#[deprecated]` for at least one MINOR before removal.
  - Provide replacement guidance and migration snippets in docs.
- Schema versioning is independent (`schema_version` field in facts), managed per SPEC.

## Consequences

+ Predictable upgrades for consumers.
+ Clear separation between code API semver and audit schema versioning.
- Requires ongoing discipline and documentation.

## Links

- `cargo/switchyard/PLAN/10-architecture-outline.md`
- `cargo/switchyard/SPEC/SPEC.md` §§ 5, 13
