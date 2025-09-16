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
- REQ-PKG-3: Package manager operations are part of the high-level flows (`use`, `replace`, `restore`) and occur in the CLI layer
  before and/or after the engine `plan/preflight/apply` phases. The engine steps themselves **MUST NOT** invoke `apt`/`dpkg`.
  There are no standalone package-manager-only commands. In dry-run, no mutations occur; the CLI **SHOULD** print the exact
  `apt`/`dpkg` commands it would run.
- REQ-PKG-4: Package manager operations **MUST** run against the live system root (`--root=/`). When `--root` points to a
  non-/ path, PM commands **MUST** refuse with guidance (running inside a chroot with apt configured is acceptable but
  out of the current scope).

Acceptance:

- Given `apt` is running and holds `/var/lib/dpkg/lock-frontend`, when I run `oxidizr-deb --commit ...`, then it fails with a message: "Package manager busy (dpkg/apt lock detected); retry after current operation finishes." and exits non-zero without mutating.

---

## 3. Provider Availability Invariants (Coreutils & Sudo)

- REQ-AV-1: At all times there **MUST** be at least one functional provider of `coreutils` and one provider of `sudo`
  installed. The CLI **MUST** refuse any `replace` or `restore` removal step that would leave zero providers.
- REQ-AV-2: The CLI **MUST** verify provider counts before and after PM operations and abort when invariants would be
  violated.

Acceptance:

- Given only `coreutils` is installed and `uutils-coreutils` is not installed, when I run `oxidizr-deb --commit replace coreutils`, then the command fails with an invariant error and no PM changes are performed.

---

## 4. Replace (Removal/Purge) UX

- REQ-REP-1: The CLI **MUST** provide a `replace <package|all>` command to remove/purge legacy distro packages only after the
  replacement is installed, active, and healthy (based on the last committed run and smoke status). If not active, `replace`
  **MUST** perform `use` semantics first and then proceed.
- REQ-REP-2: `replace` **MUST** respect dpkg/apt locks (see §2). If locks are present, it fails closed with a friendly diagnostic
  and does not invoke any package manager tools.
- REQ-REP-3: `replace` **MUST** require explicit confirmation when a TTY is present unless `--assume-yes` is set.
- REQ-REP-4: `replace` **MUST** use distro tools (`apt-get`/`dpkg`) to remove or purge the legacy packages. It **MUST** propagate
  exit codes and **SHOULD** capture a short stderr summary.
- REQ-REP-5: `replace` **MUST** emit a structured CLI event (not an engine fact) with fields: `pm.tool`, `pm.args`, `exit_code`,
  `stderr_tail`, and `package`. Logs **MUST NOT** contain secrets.
- REQ-REP-6: Dry‑run **MUST NOT** execute any package manager mutations and **SHOULD** print the exact command that would run.

Acceptance:

- Given coreutils is active and healthy and no locks are present, when I run `oxidizr-deb --commit replace coreutils`, then `apt-get purge -y coreutils` is invoked and the command exits 0.
- Given locks are present, when I run `oxidizr-deb replace coreutils --commit`, then the command exits non-zero without invoking `apt-get` and prints a lock diagnostic.

---

## 5. Alternatives Integration (Implementation Detail)

- REQ-ALT-1: The CLI **MAY** use `update-alternatives` under the hood to register linked binaries instead of creating direct symlinks.
- REQ-ALT-2: When used, registration **MUST** be idempotent and compatible with `restore`.
- REQ-ALT-3: The CLI **SHOULD** form appropriate groups when applicable (e.g., `[` vs `test`), without exposing applet selection.
- REQ-ALT-4: The CLI **MUST** revert the prior topology on `restore` when alternatives are in use.

Acceptance:

- After `use coreutils`, `update-alternatives --display ls` shows the configured provider pointing to the chosen source; `oxidizr-deb restore coreutils` reverts the prior topology.

---

## 6. dpkg-divert Strategy (Implementation Detail)

- REQ-DIVERT-1: The CLI **MAY** use `dpkg-divert` under the hood to move the original binary aside and place a replacement symlink.
- REQ-DIVERT-2: When diversion fails, the CLI **MUST** stop and leave the filesystem unchanged.
- REQ-DIVERT-3: `restore` **MUST** undo diversions it created.

Acceptance:

- After `use sudo`, `dpkg-divert --list | grep /usr/bin/sudo` lists the diversion; `oxidizr-deb restore sudo` removes it.

---

## 7. sudo Package Hardening

- REQ-SUDO-1: Before commit, the replacement binary for `sudo` **MUST** be `root:root` and `mode=4755` (setuid root).
- REQ-SUDO-2: If not satisfied, preflight **MUST** STOP with an error and a human-readable remediation.
- REQ-SUDO-3: The CLI **SHOULD** support `dpkg-statoverride` or capabilities checks in future versions (non-normative now).

Acceptance:

- Given the replacement `sudo-rs` is not setuid root, when I run `oxidizr-deb --commit use sudo`, then preflight fails closed with an explanation about setuid ownership/mode.

---

## 8. Coreutils and Findutils Package Ergonomics

- REQ-CU-1: Applet selection **MUST NOT** be exposed in the CLI; mappings to unified binaries are internal and conservative for both coreutils and findutils.
- REQ-CU-2: After commit, the CLI **SHOULD** print a short "Next steps" hint that references `oxidizr-deb --commit replace <package>` to remove legacy packages safely under guardrails.

Acceptance:

- After a successful coreutils or findutils commit, the CLI prints a safe reminder, e.g.: "Next steps: when confident, run 'oxidizr-deb --commit replace coreutils' to remove legacy packages; see --help for rollback." (wording may vary but MUST include a safe reminder).

---

## 9. Prompts & Non-Interactive Modes

- REQ-UX-1: On `--commit`, the CLI **SHOULD** present a summary prompt (unless `--assume-yes`) showing N planned actions and affected directories.
- REQ-UX-2: `--assume-yes` **MUST** suppress prompts for batch use.
- REQ-UX-3: Dry-run output **SHOULD** be parsable (stable keys and order) to support wrappers.

Acceptance:

- Running with `--commit` interactively shows a confirmation that includes the number of actions and top-level target dirs.

---

## 10. Output Conventions & Diagnostics

- REQ-OUT-1: Error messages **MUST** include stage context (preflight/apply) and one-line cause.
- REQ-OUT-2: Debian-specific tips **SHOULD** accompany common failures:
  - Apt/dpkg lock → tip to wait and retry.
  - Missing setuid on `sudo` → tip to verify `chown root:root` and `chmod 4755` on the replacement binary.
  - Alternatives or diversion flags missing → suggest `--use-alternatives`/`--use-divert` where supported.

---

## 11. Smoke Test Extensions (Packages)

- REQ-SMOKE-1 (sudo): Minimal additional checks **SHOULD** verify owner/mode (no execution of `sudo` required).
- REQ-SMOKE-2 (coreutils): Optional checks **MAY** validate that representative commands resolve to the unified binary (mapping remains internal).

---

## 12. Documentation & Completions

- REQ-DOC-1: The CLI **SHOULD** offer shell completion generation (bash/zsh/fish) via a dedicated command.
- REQ-DOC-2: The CLI **MAY** offer a manpage generator. Distributions **MAY** package prebuilt manpages.

---

## 13. Safety Boundaries

- REQ-SAFE-1: The engine `plan/preflight/apply` **MUST NOT** invoke package manager operations. PM operations are allowed only
  as part of the high-level flows (`use`, `replace`, `restore`) with guardrails (locks, confirmations, dry‑run safety) and the
  live-root constraint.
- REQ-SAFE-2: All swapped paths **MUST** be within `--root`. No out-of-root writes.

---

## 14. Fetching & Verification (Debian UX specifics)

- REQ-FETCH-D-1: For `use <package>` and `replace <package>`, the CLI **MUST** ensure the appropriate replacement package for the
  distro and architecture is installed via APT/DPKG.
- REQ-FETCH-D-2: Integrity and provenance **MUST** rely on the package manager’s signature verification and repository trust.
- REQ-FETCH-D-3: An offline local artifact path **MAY** be provided in future; verification would still apply.
