# oxidizr-deb User Flows (Debian/Ubuntu)

This file documents end-to-end user flows for oxidizr-deb. It reflects the updated SPEC where the CLI manages both
replacement and distro packages via APT/DPKG, and uses Switchyard for safe filesystem swaps.

---

## Guardrails and Invariants (Always On)

- At least one provider of coreutils and one provider of sudo must always be installed.
  - Providers (coreutils): GNU `coreutils` or `rust-coreutils` (replacement).
  - Providers (sudo): GNU `sudo` or `sudo-rs` (replacement).
- Package manager operations require the live root (`--root=/`).
- Package manager locks abort operations with a friendly message.
- Dry-run never mutates: the CLI prints the exact apt/dpkg command it would run.
- The CLI exposes only three high-level operations: `use`, `replace`, and `restore`. There are no standalone package-manager-only commands; installs/removals happen inside these flows.
- Engine-backed swaps (`use`/`replace`/`restore`) enforce SafePath, TOCTOU-safe operations, backups + rollback, and minimal smoke tests.

---

## Quick Command Cheat‑Sheet

- Use replacements (installs if needed; makes them preferred): `oxidizr-deb --commit use <target>`
- Replace distro with replacements (installs if needed; makes them preferred; removes distro packages): `oxidizr-deb --commit replace <target>`
- Restore to distro (reinstalls GNU packages if needed; makes them preferred). By default removes RS packages; keep them with `--keep-replacements`: `oxidizr-deb --commit restore <target> [--keep-replacements]`
- Status / Doctor: `oxidizr-deb status`, `oxidizr-deb doctor`

`<target>` ∈ { `coreutils`, `findutils`, `sudo` }

Note: Mutating flows (`use`, `replace`) ensure the relevant rust replacement packages are installed and upgraded to the latest available version via APT/DPKG.

---

## Flow 1 — Switch coreutils to the latest rust-coreutils (safe swap)

1) Preview (dry‑run):
   - `oxidizr-deb use coreutils`
   - Ensures `rust-coreutils` can be installed (will be installed during commit if missing).
   - Prints planned action count; no changes.

2) Commit:
   - `oxidizr-deb --commit use coreutils`
   - Pre-checks APT locks; confirms (unless `--assume-yes`).
   - Installs/updates `rust-coreutils` (latest) via apt if missing/outdated.
   - Switchyard plan → preflight → apply to set the symlink topology under `/usr/bin` with backups.
   - Runs minimal smoke tests; auto‑rollback on failure; exits non‑zero with diagnostics if failed.

3) Verify:
   - `oxidizr-deb status` shows `coreutils: active`.

---

## Flow 2 — Replace coreutils (remove GNU `coreutils`)

1) Preconditions:

- `coreutils` is active (symlinks point to rust-coreutils).
- APT is not holding locks.

2) Command:
   - `oxidizr-deb --commit replace coreutils`

3) Behavior:

- Confirms (unless `--assume-yes`).
- Ensures `rust-coreutils` is installed/updated and preferred (performs "use" semantics if not already active).
- Verifies availability invariant will still hold (rust replacement remains installed).
- Runs `apt-get purge -y coreutils` and emits a `pm.purge` event with tool/args/exit code/stderr tail.

---

## Flow 3 — Restore coreutils to GNU (with optional keep)

1) Command:

- `oxidizr-deb --commit restore coreutils [--keep-replacements]`

2) Behavior:

- Ensures GNU `coreutils` is installed (installs if missing) and makes it preferred: Switchyard restores backups and removes CLI‑managed symlinks to reinstate the prior GNU topology.
- By default, removes the replacement package (`rust-coreutils`) via apt; if `--keep-replacements` is provided, keeps it installed but de‑preferred.
- Runs minimal smoke tests; auto‑rollback on failure; exits non‑zero with diagnostics if failed.

---

## Flow 4 — sudo specifics

- On `use sudo`, preflight requires the replacement binary (`sudo-rs`) to be `root:root` and mode `4755` (setuid root) before commit.
- Availability invariant applies: cannot remove both `sudo` and `sudo-rs`.
- Replace for sudo removes `sudo` after `sudo-rs` is active and healthy: `oxidizr-deb --commit replace sudo`.

---

## Flow 5 — Diagnostics and health

- `oxidizr-deb status [--json]` — shows active states for `coreutils`, `findutils`, `sudo`.
- `oxidizr-deb doctor [--json]` — checks common issues (paths, locks) and prints tips.

---

## Flow 6 — Common failure cases and resolutions

- APT/DPKG lock present
  - Symptom: "Package manager busy (dpkg/apt lock detected)".
  - Action: Wait for apt/dpkg to finish; retry.

- Invariant violation (last provider removal attempted)
  - Symptom: Refusal to remove package; message indicates it would leave zero providers.
  - Action: Run `oxidizr-deb --commit restore <target>` to ensure the GNU package is installed and preferred; optionally add `--keep-replacements` if you want RS packages to remain installed but de‑preferred.

- Smoke test failure after `use`
  - Symptom: `--commit use` fails, auto‑rollback triggers.
  - Action: Inspect logs/facts; resolve compatibility issues; optionally run `doctor`. If using `replace`, the same guidance applies.

- Non‑live root (`--root` not `/`) for PM operations
  - Symptom: PM command refuses and prints guidance.
  - Action: Run on the live system (or inside a chroot with apt configured; currently out of scope).

---

## Notes and Future Extensions

- Version pinning: CLI may allow `--version` for install commands in the future; latest remains default.
- Alternatives/diversions: optional integration behind features; `restore` undoes what `use` configured.
- Telemetry: PM operations emit CLI-level structured events (pm.install/remove/purge).
