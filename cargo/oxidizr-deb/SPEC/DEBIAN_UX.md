# Debian/Ubuntu UX Addendum (RFC-2119)

This addendum specifies CLI ergonomics and guardrails tailored for Debian/Ubuntu and derivatives. It complements
`cargo/oxidizr-deb/SPEC/SPEC.md` and inherits the engine invariants.

---

## 1. Distro Detection & Layout

- REQ-DIST-1: The CLI **MUST** detect the distro via `/etc/os-release` and record a `distro_id` fact (e.g., `debian`, `ubuntu`).
- REQ-DIST-2: The CLI **MUST** target `/usr/bin` for merged-/usr systems; on non-merged systems (rare), `/bin` is treated
  as a compatibility symlink and messaging **SHOULD** surface `/usr/bin` as the effective target.
- REQ-DIST-3: The CLI **SHOULD** detect `usrmerge` (e.g., `/bin -> /usr/bin`) and surface a single effective target directory in messages.

Acceptance (pseudo):

- Given `/bin` is a symlink to `/usr/bin`, when `oxidizr-deb use coreutils` runs, then targets under `/usr/bin` are shown in the plan output.

---

## 2. Package-Manager Safety (APT/DPKG)

- REQ-PKG-1: Before `apply`, the CLI **MUST** check for dpkg/apt locks and **MUST** stop with a friendly diagnostic when detected.
  Locks to check (non-exhaustive): `/var/lib/dpkg/lock-frontend`, `/var/lib/dpkg/lock`, `/var/lib/apt/lists/lock`.
- REQ-PKG-2: The CLI **SHOULD** recommend re-running after any ongoing `apt`/`dpkg` operation completes.
- REQ-PKG-3: The CLI **MUST NOT** invoke `apt`, `apt-get`, or `dpkg`; it is not a package manager.

Acceptance:

- Given `apt` is running and holds `/var/lib/dpkg/lock-frontend`, when I run `oxidizr-deb --commit ...`, then it fails with a message: "Package manager busy (dpkg/apt lock detected); retry after current operation finishes." and exits non-zero without mutating.

---

## 3. Alternatives Integration (Implementation Detail)

- REQ-ALT-1: The CLI **MAY** use `update-alternatives` under the hood to register linked binaries instead of creating direct symlinks.
- REQ-ALT-2: When used, registration **MUST** be idempotent and compatible with `restore`.
- REQ-ALT-3: The CLI **SHOULD** form appropriate groups when applicable (e.g., `[` vs `test`), without exposing applet selection.
- REQ-ALT-4: The CLI **MUST** revert the prior topology on `restore` when alternatives are in use.

Acceptance:

- After `use coreutils`, `update-alternatives --display ls` shows the configured provider pointing to the chosen source; `oxidizr-deb restore coreutils` reverts the prior topology.

---

## 4. dpkg-divert Strategy (Implementation Detail)

- REQ-DIVERT-1: The CLI **MAY** use `dpkg-divert` under the hood to move the original binary aside and place a replacement symlink.
- REQ-DIVERT-2: When diversion fails, the CLI **MUST** stop and leave the filesystem unchanged.
- REQ-DIVERT-3: `restore` **MUST** undo diversions it created.

Acceptance:

- After `use sudo`, `dpkg-divert --list | grep /usr/bin/sudo` lists the diversion; `oxidizr-deb restore sudo` removes it.

---

## 5. sudo Package Hardening

- REQ-SUDO-1: Before commit, the replacement binary for `sudo` **MUST** be `root:root` and `mode=4755` (setuid root).
- REQ-SUDO-2: If not satisfied, preflight **MUST** STOP with an error and a human-readable remediation.
- REQ-SUDO-3: The CLI **SHOULD** support `dpkg-statoverride` or capabilities checks in future versions (non-normative now).

Acceptance:

- Given the replacement `sudo-rs` is not setuid root, when I run `oxidizr-deb --commit use sudo`, then preflight fails closed with an explanation about setuid ownership/mode.

---

## 6. Coreutils and Findutils Package Ergonomics

- REQ-CU-1: Applet selection **MUST NOT** be exposed in the CLI; mappings to unified binaries are internal and conservative for both coreutils and findutils.
- REQ-CU-2: After commit, the CLI **SHOULD** print a short "Next steps" hint showing example `apt` commands to remove legacy packages (informational only).

Acceptance:

- After a successful coreutils or findutils commit, the CLI prints a safe reminder, e.g.: "Next steps: review 'apt purge <package>' only if you are confident; see --help for rollback." (wording may vary but MUST include a safe reminder).

---

## 7. Prompts & Non-Interactive Modes

- REQ-UX-1: On `--commit`, the CLI **SHOULD** present a summary prompt (unless `--assume-yes`) showing N planned actions and affected directories.
- REQ-UX-2: `--assume-yes` **MUST** suppress prompts for batch use.
- REQ-UX-3: Dry-run output **SHOULD** be parsable (stable keys and order) to support wrappers.

Acceptance:

- Running with `--commit` interactively shows a confirmation that includes the number of actions and top-level target dirs.

---

## 8. Output Conventions & Diagnostics

- REQ-OUT-1: Error messages **MUST** include stage context (preflight/apply) and one-line cause.
- REQ-OUT-2: Debian-specific tips **SHOULD** accompany common failures:
  - Apt/dpkg lock → tip to wait and retry.
  - Missing setuid on `sudo` → tip to verify `chown root:root` and `chmod 4755` on the replacement binary.
  - Alternatives or diversion flags missing → suggest `--use-alternatives`/`--use-divert` where supported.

---

## 9. Smoke Test Extensions (Packages)

- REQ-SMOKE-1 (sudo): Minimal additional checks **SHOULD** verify owner/mode (no execution of `sudo` required).
- REQ-SMOKE-2 (coreutils): Optional checks **MAY** validate that representative commands resolve to the unified binary (mapping remains internal).

---

## 10. Documentation & Completions

- REQ-DOC-1: The CLI **SHOULD** offer shell completion generation (bash/zsh/fish) via a dedicated command.
- REQ-DOC-2: The CLI **MAY** offer a manpage generator. Distributions **MAY** package prebuilt manpages.

---

## 11. Safety Boundaries

- REQ-SAFE-1: The CLI **MUST** never invoke package manager operations; messaging **MUST** be clearly advisory.
- REQ-SAFE-2: All swapped paths **MUST** be within `--root`. No out-of-root writes.

---

## 12. Fetching & Verification (Debian UX specifics)

- REQ-FETCH-D-1: For `use <package>`, the CLI **MUST** select artifacts appropriate for the distro and architecture.
- REQ-FETCH-D-2: The CLI **MUST** verify SHA‑256 and, when available, signatures before any mutation.
- REQ-FETCH-D-3: An offline local artifact path **MAY** be provided; verification still applies.
