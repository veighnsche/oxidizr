# ADR Template

- Title: Dependency and supply chain policy
- Status: Accepted
- Date: 2025-09-11

## Context

The crate must minimize external dependencies, ensure license compatibility (Apache-2.0 OR MIT; dual-licensed), respect MSRV, and provide provenance/SBOM signals in CI. SPEC calls for provenance and signed attestations.

## Decision

- Keep dependency set minimal; prefer stdlib and well-vetted crates.
- Maintain MSRV pinned to current `stable` during implementation; document version at first release.
- Require license check for all new dependencies; reject incompatible licenses.
- Generate SBOM-lite and include provenance fields in audit facts.
- Review third-party crates for maintenance status and supply-chain posture.

## Consequences

+ Smaller trusted surface and attack surface.
+ Easier updates and audits.
- Possible reimplementation of minor utilities to avoid heavy crates.

## Links

- `cargo/switchyard/PLAN/00-charter.md`
- `cargo/switchyard/SPEC/SPEC.md` §§ 2.4, 13
