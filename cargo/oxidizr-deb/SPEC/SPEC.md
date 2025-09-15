# oxidizr-deb Specification (RFC-2119)

## 0. Domain & Purpose

oxidizr-deb is a Debian-family CLI that orchestrates safe, atomic, reversible filesystem swaps
(e.g., GNU coreutils → uutils-coreutils; sudo → sudo-rs) with a simple package-level UX.

- Users do not choose applets, sources, or targets. The CLI provides a high-level `rustify` command per package
  and executes a safe plan under the hood.
- The CLI includes an integrated fetch-and-verify step to obtain the correct replacement artifacts.
- Package manager operations (install/remove) are out of scope; after a successful rustify, users may remove legacy
  packages themselves if they choose to make the change permanent.

---

## 1. Main Guarantees

The CLI inherits a safety engine’s guarantees and adds CLI-specific guardrails. Unless explicitly stated otherwise
below, the engine’s invariants (SafePath boundaries, TOCTOU-safe syscalls, rollback on failure, deterministic plans)
apply transitively to oxidizr-deb.

- Atomic, crash-safe swaps with backups and no user-visible broken/missing path.
- Complete, idempotent rollback on mid-plan failure (engine-managed).
- `SafePath` is enforced at CLI boundaries; all mutating inputs are validated.
- Deterministic plans and outputs via the engine (stable IDs; dry-run redactions stabilized).
- Production locking enabled by default via a filesystem lock under `<root>/var/lock/oxidizr-deb.lock`.
- Minimal smoke tests are run post-apply under production presets; failure triggers auto‑rollback unless disabled by policy.
- Dry‑run is the default mode; side effects require `--commit`.
- Cross-filesystem safety follows the package policy (built-in packages disallow degraded mode by default).

---

## 2. Normative Requirements

### 2.1 CLI Construction & SafePath Boundaries

- REQ-C1: The CLI **MUST** accept a `--root` argument (default `/`) and construct all mutating paths using a SafePath
  boundary rooted at `--root`. Any failure to validate **MUST** abort the command with an error message.
- REQ-C2: The CLI **MUST NOT** pass unvalidated filesystem paths to mutating engine APIs.
- REQ-C3: The `root` argument **MUST** be absolute.

### 2.2 Modes & Conservatism

- REQ-M1: The CLI **MUST** default to dry‑run. A user **MUST** supply `--commit` to perform mutations.
- REQ-M2: On dry‑run, the CLI **SHOULD** emit a clear summary (e.g., planned action counts).
- REQ-M3: On failure, the CLI **MUST** not leave the system in a partially applied state; automatic reverse‑order
  rollback semantics apply.

### 2.3 Locking (Process Serialization)

- REQ-L1: The CLI **MUST** configure process serialization with bounded wait. Default lock path:
  `<root>/var/lock/oxidizr-deb.lock`.
- REQ-L2: When the active policy requires locking, absence of a lock **MUST** cause apply to fail rather than proceed concurrently.

### 2.4 Packages & Implicit Policies

- REQ-PKG-1: `rustify coreutils` and `rustify findutils` **MUST** apply implicit policies tuned for their link topologies,
  including disallowing degraded cross‑filesystem fallback (EXDEV → fail), strict ownership and preservation where
  applicable, and forbidding untrusted sources.
- REQ-PKG-2: `rustify sudo` **MUST** apply a production‑grade policy tuned for replacing `/usr/bin/sudo` safely.
- REQ-PKG-3: Cross‑filesystem degraded fallback **MUST** be disallowed by default for all built‑in packages.

### 2.5 Health Verification

- REQ-H1: Under commit, the CLI **MUST** run minimal post‑apply smoke checks appropriate to the package and obey the
  policy’s `require_smoke_in_commit` behavior: smoke failure triggers auto‑rollback and an error.

### 2.6 Observability & Audit

- REQ-O1: The CLI **MUST** initialize audit/facts sinks for the engine.
- REQ-O2: When compiled with file‑logging support, deployments **MAY** configure file‑backed JSONL sinks; otherwise a
  no‑op sink satisfies development usage.
- REQ-O3: The CLI **MUST NOT** emit secrets in its own logs; redaction is enforced by the engine’s sinks.

### 2.7 Error Reporting & Exit Codes

- REQ-E1 (v0): The CLI **MUST** exit `0` on success and `1` on error. A future revision **SHOULD** align exit codes with
  a published error taxonomy.
- REQ-E2: User‑facing error messages **SHOULD** include the failing stage (preflight/apply) and a brief cause.

### 2.8 Filesystems & Degraded Mode

- REQ-F1: Cross‑filesystem behavior is governed by the package’s implicit policy. Degraded fallback **MUST** be disallowed
  by default (apply fails with a stable reason; no visible change).

### 2.9 Fetching & Verification

- REQ-FETCH-1: `rustify <package>` **MUST** fetch the appropriate replacement artifact for the current architecture and
  verify its integrity (SHA‑256, and signature when available) before applying any changes.
- REQ-FETCH-2: The default selection **MUST** be the latest stable release, with an option to choose a channel
  (e.g., stable vs. latest) in future versions.
- REQ-FETCH-3: An offline mode **MUST** allow providing a local artifact path, still subject to integrity checks.

---

## 3. Public Interfaces (CLI)

### 3.1 Synopsis

```
oxidizr-deb [--root PATH] [--commit] <COMMAND> [ARGS]
```

Global options:

- `--root PATH` — absolute root of the filesystem tree (default `/`).
- `--commit` — commit changes; without it, commands run in dry‑run.

### 3.2 Commands

- rustify
  - Arguments: `<package>` where `<package>` ∈ {`coreutils`, `findutils`, `sudo`} (extensible).
  - Semantics: fetches and verifies the replacement for the given package, then plans and applies a safe link topology
    with backups. No applet selection is exposed; coreutils/findutils mappings are internal.
- restore
  - Arguments: `<package|all>`.
  - Semantics: restores GNU/stock tools for the package (or all packages) from backups.
- status
  - Arguments: none.
  - Semantics: reports current rustified state and what can be restored.

All commands execute `plan → preflight → apply` through the engine and honor policy gates.

---

## 4. Preflight Diff & Audit Facts

oxidizr-deb reuses the engine’s schemas without modification.

- Preflight Diff schema: see engine SPEC.
- Audit Facts schema: see engine SPEC.

Dry‑run outputs are byte‑identical to real‑run (after redactions) and follow deterministic ordering.

---

## 5. Filesystems & Degraded Mode (Operational Guidance)

- Coreutils and Findutils: degraded cross‑filesystem fallback is disallowed by default; EXDEV causes apply to fail with a
  stable reason marker; no visible changes occur.
- Sudo: degraded fallback is disallowed by default.

---

## 6. Acceptance Tests (CLI-flavored BDD)

```gherkin
Feature: Safe swaps via CLI
  Scenario: Dry-run rustify of coreutils
    Given a staging root at /tmp/fakeroot
    When I run `oxidizr-deb --root /tmp/fakeroot rustify coreutils`
    Then the command exits 0
    And it reports a dry-run with a non-zero planned action count

  Scenario: Commit sudo rustify
    Given a verified sudo-rs artifact is available
    When I run `oxidizr-deb --commit rustify sudo`
    Then the command exits 0
    And subsequent reads of /usr/bin/sudo resolve to the rust replacement

  Scenario: Rustify and restore findutils
    Given a staging root at /tmp/fakeroot
    And a verified findutils artifact is available
    When I run `oxidizr-deb --commit rustify findutils`
    Then the command exits 0
    And representative findutils commands resolve to the rust replacement
    When I run `oxidizr-deb restore findutils`
    Then the command exits 0
    And the original binaries are restored

  Scenario: Restore package
    Given backups exist for coreutils
    When I run `oxidizr-deb restore coreutils`
    Then the command exits 0
    And the original binaries are restored
```

---

## 7. Operational Bounds

- Default lock file: `<root>/var/lock/oxidizr-deb.lock`.
- All operations are scoped to `--root`.
- Plan sizes and performance bounds are inherited from the engine.

---

## 8. Security Requirements Summary (CLI)

- Enforce SafePath at boundaries and reject unsafe paths.
- Dry‑run by default; explicit `--commit` required.
- Locking configured by default under a predictable path; bounded wait.
- Minimal smoke checks post‑apply; failure triggers auto‑rollback unless disabled by policy.
- Cross‑filesystem degraded mode disallowed by default for built‑in packages.
- Fetch and verify replacement artifacts before any mutation.

---

## 9. Versioning & Future Work

- v0 CLI exits 0 on success, 1 on error. Future versions **SHOULD** align exit codes with a published taxonomy
  and surface specific error identifiers.
- Future flags **MAY** expose policy toggles (e.g., degraded fallback, rescue thresholds, retention pruning), provided
  they continue to enforce SafePath and engine invariants.

---

## 10. Debian/Ubuntu UX Addendum

For Debian/Ubuntu-focused ergonomics (apt/dpkg lock detection, optional `update-alternatives`/`dpkg-divert` modes,
sudo setuid checks, prompts, completions, and diagnostics), see `cargo/oxidizr-deb/SPEC/DEBIAN_UX.md`. These
requirements complement this SPEC and are normative where marked with RFC‑2119 keywords.
