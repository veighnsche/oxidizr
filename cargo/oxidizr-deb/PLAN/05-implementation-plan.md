# Implementation Plan for oxidizr-deb

## Phase 0 — Scaffolding (Done)

- Minimal CLI wired to Switchyard with builder and adapters.
- SPEC and Debian UX addendum authored.

## Phase 1 — CLI Surface & Safety (package-level)

- Commands: `use <package>`, `restore <package|all>`, `status`, `completions`.
- Flags: `--assume-yes`, `--commit`, global `--root`.
- Debian safety: dpkg/apt lock detection (no mutations if busy), friendly diagnostics.
- Sudo guard: preflight check for `root:root` + `4755` on replacement in commit mode.
- No applet exposure in CLI; coreutils mapping is internal.

## Phase 2 — Optional Distro Integrations

- `--use-alternatives` (feature `debian-alternatives`): idempotent registration; `restore` reverts prior topology.
- `--use-divert` (feature `debian-divert`): diversion create/remove; transactional with rollback.

## Phase 3 — Fetch/Verify & Tests

- Implement fetcher: resolver (arch/distro/channel), verifier (SHA-256, signature when available), offline local path.
- Unit tests for helpers (locks, prompts, path rules, fetch verification).
- BDD features from plan; golden fixtures for deterministic outputs.
- Test matrix generator; parameterized runs.

## Phase 4 — Observability & Docs

- Optional file-logging sinks via engine feature; path configurable under `--root`.
- README + manpage/completions generation.
- SPEC trace matrix and CI check.

## Phase 5 — Exit Codes Alignment (Optional)

- Map common errors to a published error taxonomy and exit codes.

## Risk Register

- Privilege requirements for chmod/chown in tests → operate under `--root` fakeroot; skip privileged ops.
- Alternatives/divert require exec of system tools → provide mock layers in tests; gate under features.

## Deliverables & Done Criteria

- Package-level CLI (`use`, `restore`, `status`) functional with unit + BDD coverage.
- Fetch-and-verify implemented and covered by tests.
- Debian UX acceptance items satisfied.
- CI matrix green; fixtures stable; zero SKIP.
