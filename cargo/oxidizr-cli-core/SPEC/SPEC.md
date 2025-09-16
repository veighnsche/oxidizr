# oxidizr-cli-core Specification (RFC-2119)

## 0. Domain & Purpose

`oxidizr-cli-core` provides the shared, cross-distro contract and helpers used by all Oxidizr CLIs (e.g., `oxidizr-deb`, `oxidizr-arch`, future `oxidizr-rpm`). It standardizes semantics, coverage guarantees, and interfaces so every Oxidizr behaves identically regardless of distro, while delegating package-manager specifics to adapter implementations.

This document uses RFC‑2119 keywords (MUST, SHOULD, MAY).

---

## 1. Scope and Shared Guarantees

The library’s scope is to ensure all Oxidizr CLIs:

- Replace and restore complete package toolchains (coreutils, findutils, sudo) without gaps.
- Never leave missing commands after a replacement.
- Provide the same command surface and semantics across distros.
- Use consistent coverage logic:
  - Discover what the replacement binary supports
  - Enumerate distro-provided command names under `/usr/bin` (and legacy `/bin`)
  - Intersect or compare those sets to prevent gaps
- Keep filesystem mutations safe, reversible, and bounded under `--root`.

The low-level filesystem safety (SafePath, TOCTOU-safe apply, backups, rollback) is enforced by the underlying Switchyard engine that the CLIs invoke. This spec defines how Oxidizr CLIs must use `oxidizr-cli-core` to meet their user-facing guarantees.

---

## 2. Command Surface (MUST be identical across CLIs)

- `use <package>`
  - Ensures the replacement package is present and preferred; links the complete dynamic applet set.
  - Does not remove the distro package.
- `replace <package|all>`
  - Ensures the replacement is present & active; then removes/purges the legacy distro package(s) under guardrails.
  - MUST pass coverage preflight (zero missing commands) before removal.
- `restore <package|all>`
  - Restores the complete distro command set and de-preferences/removes the replacement package(s) (policy-dependent).
- `status`
  - Reports current state.
- `doctor` and `completions`
  - Optional but SHOULD be consistent across CLIs.

No PM-only commands (install/remove/purge) are exposed directly; they happen inside `use/replace/restore`.

---

## 3. Global Flags

- `--root PATH` (default `/`): absolute root under which all paths are scoped.
- `--commit`: perform mutations; dry-run by default.
- `--assume-yes`: suppresses prompts in interactive runs.
- `--offline --use-local PATH`:
  - MAY be supported for development in `use`.
  - MUST NOT be used for `replace` (replace must verify real PM coverage).

---

## 4. Path & Filesystem Rules

- `DEST_DIR` is `/usr/bin` on merged-/usr systems. Legacy `/bin` is treated as a symlink or compatibility path.
- All mutations MUST be SafePath-bounded under `--root`.
- Cross-filesystem degraded mode for built-in packages is disallowed by default (policy-driven in the engine).

---

## 5. Coverage & No-Missing-Commands Guarantees

These are the core cross-distro guarantees `oxidizr-cli-core` enforces through shared algorithms and must be uniformly honored:

- REQ-COVER-1 (Replace): After `replace <package>` commits, every command provided by the distro package under `/usr/bin` (and legacy `/bin`) MUST still exist and resolve to a functional provider (the replacement). There MUST NOT be any missing commands or dangling symlinks.
- REQ-COVER-2 (Preflight): Before purging legacy packages, the CLI MUST:
  1) Enumerate the distro-provided commands for the package (via adapter).
  2) Discover applets supported by the replacement (via unified binary).
  3) Require 100% coverage; else STOP with an explicit missing list; no PM mutations must occur.
- REQ-COVER-3 (Use): `use <package>` MUST link all applets supported by the replacement on this system. On live roots, it MUST intersect the replacement-supported set with the distro-provided set to avoid stray targets.
- REQ-COVER-4 (Post-Verify): After `replace`, zero missing commands MUST be verified before reporting success. Failures MUST abort with engine rollback preserving availability.

---

## 6. Distro Adapter Contract (Trait)

Each CLI must provide a distro adapter that implements:

- `enumerate_package_commands(root: &Path, pkg: PackageKind) -> Vec<String>`
  - Returns the distro-provided commands under `/usr/bin` (and legacy `/bin`) for a package.
  - If enumeration is not possible (e.g., non-live root), return an empty vector.

Implementations:

- Debian/Ubuntu: `dpkg-query -L <pkg>`, filter `/usr/bin` and `/bin`.
- Arch: `pacman -Ql <pkg>`, parse `<pkg> <path>`, filter `/usr/bin` and `/bin`.
- RPM-based: `rpm -ql <pkg>`, filter similar paths.

The adapter MAY expose additional distro-specific helpers (locks, PM invocation, etc.) in the downstream CLIs, but this is the minimal interface `oxidizr-cli-core` requires.

---

## 7. Replacement Discovery & Static Fallback

`oxidizr-cli-core` provides a comprehensive static fallback list of applets per package (e.g., a complete GNU coreutils list that includes `[` alias). It also provides discovery helpers to interrogate the replacement unified binary.

Algorithm overview used by core:

- Discover with allowlist:
  - Try `--list` on the unified binary; else `--help`.
  - Parse tokens; intersect with the static allowlist for the package.
  - If discovery yields a tiny set, fallback to the static set.

- `use` (resolve applets):
  - Replacement-supported set (discovered or fallback)
  - If adapter enumeration is available (non-empty), intersect with distro-provided names.
  - Link the resulting set.

- `replace` (coverage preflight):
  - Replacement-supported set (discovered or fallback)
  - Distro-provided set (adapter)
  - Require coverage_check(distro ⊆ replacement) to pass → else explicit error with missing list.

---

## 8. Public API (Library Surface)

Types:

- `enum PackageKind { Coreutils, Findutils, Sudo }`
- `const DEST_DIR: &str = "/usr/bin"`

Adapter:

- `trait DistroAdapter { fn enumerate_package_commands(&self, root: &Path, pkg: PackageKind) -> Vec<String>; }`

Helpers:

- `static_fallback_applets(pkg: PackageKind) -> Vec<String>`
- `discover_applets_with_allow(source_bin: &Path, allow: &[String]) -> Vec<String>`
- `intersect_distro_with_replacement(distro: &[String], repl: &[String]) -> Vec<String>`
- `coverage_check(distro: &[String], repl: &[String]) -> Result<(), Vec<String>>`
- `resolve_applets_for_use(adapter: &impl DistroAdapter, root: &Path, pkg: PackageKind, source_bin: &Path) -> Vec<String>`
- `coverage_preflight(adapter: &impl DistroAdapter, root: &Path, pkg: PackageKind, source_bin: &Path) -> Result<(), Vec<String>>`

Prompts/API builder:

- Provided as convenience modules for downstream CLIs; not normative here.

---

## 9. CLI Behavior That MUST Use the Shared Core

Downstream CLIs (e.g., `oxidizr-deb`, `oxidizr-arch`) MUST use the shared algorithms:

- `use`:
  - Compute applets via `resolve_applets_for_use` (dynamic discovery ∩ distro set if available).
  - Build link plan for that set under `DEST_DIR`, SafePath-bounded under `--root`.
- `replace`:
  - Require `coverage_preflight` to succeed before any purge/removal step.
  - Post-apply verification MUST check zero missing commands and abort on violation.
- `restore`:
  - Enumerate distro-provided command sets per package via adapter; if unavailable (non-live root), fallback to `static_fallback_applets`.
  - Plan complete restoration of those targets.

---

## 10. Observability & Diagnostics (Cross-CLI Conventions)

- CLI-level logging for PM actions MUST use a stable shape:
  - Fields: `event` (e.g., `pm.install`, `pm.purge`), `pm.tool`, `pm.args`, `exit_code`, `stderr_tail`, `package`.
- Error messages SHOULD include stage context and a concise cause.
- No secrets in logs.
- Post-apply smoke checks SHOULD be executed per package policy (engine-driven).

---

## 11. Locking & Package Manager Operations (Guidance)

- PM mutations MUST occur only on live roots (`--root=/`).
- Each CLI MUST check for distro PM locks and fail closed with a friendly diagnostic.
- PM steps (ensure install/upgrade, purge/remove, re-install during restore) are distro-specific and implemented by downstream CLIs. The coverage logic and verification MUST still be used from `oxidizr-cli-core`.

---

## 12. Acceptance Tests (Cross-Distro BDD Hints)

Core scenarios that every Oxidizr CLI MUST satisfy (running on a live root in CI containers):

- Use coreutils links a complete set:
  - Given a live root, when running `--commit use coreutils`, then a non-empty set of applets resolves to the replacement, and the set aligns with the distro-provided commands ∩ replacement-supported set.

- Replace coreutils preserves availability:
  - Given a live root and use is active, when running `--commit replace coreutils`, then coverage preflight passes (zero missing distro-provided commands), PM purge succeeds, and post-apply verification confirms zero missing commands.

- Restore coreutils restores all commands:
  - Given backups and PM availability, when running `--commit restore coreutils`, then the complete distro-provided set is restored and becomes preferred.

Analogous scenarios apply to `findutils` and `sudo` (sudo includes a setuid guard handled by the CLI).

---

## 13. Security Summary

- No missing commands after `replace` (explicitly verified).
- No out-of-bound path mutations; all under `--root` via SafePath.
- Dry-run by default; explicit commit required.
- PM locks prevent concurrent/dangerous operations.
- Replacement discovery and coverage checks avoid partial-brick scenarios.

---

## 14. Versioning & Future Work

- v0: This spec mandates the unified cross-distro coverage behavior and adapter interface.
- Future: the adapter trait MAY be extended with lock checks or PM helpers if we choose to unify more behaviors inside the core library. The cross-distro coverage and restore rules remain stable.

---

## 15. Appendix: Static Fallback Lists

The core provides an authoritative static fallback list for:

- Coreutils: the complete applet list synchronized to GNU upstream and including `[` alias used by distros.
- Findutils: `find`, `xargs`.
- Sudo: `sudo`.

Downstream CLIs MUST keep their mapping logic anchored to this unified source, and rely on dynamic discovery on live systems.
