# oxidizr-deb — Debian/Ubuntu CLI to use Rust replacements safely

oxidizr-deb is a small, safety-first CLI that switches key system toolchains to their
Rust replacements (e.g., GNU coreutils → uutils-coreutils, sudo → sudo-rs). It performs safe, atomic,
reversible changes under the hood and keeps a one-step restore path.

This CLI focuses on safety and UX:

- You do not choose applets, sources, or targets manually.
- The CLI fetches the right, verified replacement for your system and applies it safely.
- You can restore to GNU/stock tools at any time.
After validating your workloads, you can fully replace distro packages with the Rust replacements using `replace`, which
removes the legacy distro packages under guardrails (lock checks, confirmations, dry‑run safety). No standalone package-
manager commands are exposed; installs/removals happen inside `use`, `replace`, and `restore` flows.

---

## Key properties

- SafePath-only mutations, TOCTOU-safe syscall sequence
- Backups + one-step rollback; dry‑run by default; commit with `--commit`
- Locking by default at `<root>/var/lock/oxidizr-deb.lock` to serialize mutations
- Post-apply smoke checks with auto‑rollback on failure
- Debian/Ubuntu UX guardrails: apt/dpkg lock checks, helpful diagnostics

---

## Install / Build

- Prereq: Rust toolchain
- From repo root:

```bash
# Compile
cargo build -p oxidizr-deb

# Run (debug build)
cargo run -p oxidizr-deb -- --help
```

You can also `cargo install --path cargo/oxidizr-deb` if desired.

---

## Quickstart (safe by default)

```bash
# Use coreutils (auto-fetch verified replacement); no changes without --commit
cargo run -p oxidizr-deb -- use coreutils

# Use findutils (auto-fetch verified replacement)
cargo run -p oxidizr-deb -- use findutils

# Use sudo (auto-fetch verified replacement)
cargo run -p oxidizr-deb -- use sudo

# Apply changes
cargo run -p oxidizr-deb -- --commit use coreutils

# Make it a full replacement (remove GNU coreutils under guardrails)
cargo run -p oxidizr-deb -- --commit replace coreutils
```

At any time you can check status or restore back to GNU tools:

```bash
cargo run -p oxidizr-deb -- status
cargo run -p oxidizr-deb -- restore coreutils
```

---

## Supported Rust replacements (packages)

This CLI targets package-level replacements. No applet selection is required or exposed.

- coreutils → uutils-coreutils (unified binary; CLI manages internal mapping)
- findutils → uutils-findutils (CLI manages internal mapping)
- sudo → sudo-rs (single binary)

Roadmap (pending drop-in compatibility assessment):

- grep → ripgrep, ps → procs, du → dust, ls → eza/lsd

Tip: Start on a fakeroot or non-critical machine; validate your workloads before moving to `replace`.

---

## How oxidizr-deb ensures replacements (supply-chain safety)

oxidizr-deb ensures the appropriate replacement package for your system is installed via APT/DPKG and relies on the
package manager’s signature verification and repository trust. For development and tests, `--offline --use-local PATH`
is supported to inject a local artifact under a fakeroot; this bypasses apt (still validated in future versions). The CLI
applies ownership/mode guards as needed. For example, `sudo` replacement must be `root:root` with mode `4755` when committed.

---

## CLI overview

```text
oxidizr-deb [--root PATH] [--commit] <COMMAND> [ARGS]
```

- `--root PATH` (default `/`): operate inside a root tree (use a fakeroot/chroot for safety while testing)
- `--commit`: actually perform changes (without this, it’s a dry-run preview)

Commands:

- `use <package>` — ensure the replacement is installed via APT/DPKG and safely switch to it
- `replace <package|all>` — ensure the replacement is active, then remove/purge the distro packages under guardrails
- `restore <package|all>` — restore GNU/stock tools for a package (or all) from backups
- `status` — show what is active, queued, or restorable
- `completions` — generate shell completions (bash/zsh/fish)

Examples:

```bash
# Use coreutils (auto-fetch + dry-run)
cargo run -p oxidizr-deb -- use coreutils

# Use findutils (auto-fetch + dry-run)
cargo run -p oxidizr-deb -- use findutils

# Commit the change
cargo run -p oxidizr-deb -- --commit use coreutils

# Use sudo
cargo run -p oxidizr-deb -- --commit use sudo

# Fully replace GNU coreutils with uutils and remove GNU packages
cargo run -p oxidizr-deb -- --commit replace coreutils

# Restore (rollback) coreutils
cargo run -p oxidizr-deb -- restore coreutils
```

---

## Command semantics (clear and confident)

- `use <package>`
  - What it does: ensures the verified Rust replacement for `<package>` is installed via APT/DPKG, plans a safe swap with backups, and only mutates with `--commit`.
  - Idempotence: safe to re-run; if already active, the plan becomes a no-op.
  - Safety: refuses to commit while apt/dpkg locks are held; runs minimal smoke checks and auto‑rolls back on failure.

- `replace <package|all>`
  - What it does: ensures the replacement is installed and active; then removes/purges the legacy distro packages via APT/DPKG under guardrails. Performs `use` semantics first if needed.
  - Idempotence: safe to re-run; if already fully replaced, the PM step becomes a no‑op.
  - Safety: checks invariants; refuses if it would leave zero providers.

- `restore <package|all>`
  - What it does: restores original GNU/stock binaries from backups and ensures distro packages are installed and preferred. By default removes RS packages; keep them with `--keep-replacements`.
  - Idempotence: safe to re-run; if already restored, the plan becomes a no‑op.

- `status`
  - What it does: reports which packages are active and which are restorable.

---

## Glossary

- Replacement artifact: downloaded Rust implementation for a package (e.g., uutils‑coreutils, uutils‑findutils, sudo‑rs).
- Symlink topology: set of links so standard commands resolve to the replacement binary.
- Backup sidecar: saved original binaries/links created for restoration.
- Use: safely switch a package to its Rust replacement (download, verify, link with backups).
- Restore: switch back to GNU/stock binaries using backups; remove CLI‑managed symlinks.

---

## Permanence

After a successful `use <package> --commit`, oxidizr‑deb keeps your selection active across upgrades. No extra steps are required.
If you decide to remove the legacy distro packages, use `oxidizr-deb --commit replace <package>` to do so safely.

---

## Cleanup

The CLI automatically removes replacement artifacts for a package after a successful `restore`. No manual cleanup is needed.

Tip: to experiment safely, use a fakeroot:

```bash
sudo mkdir -p /tmp/fakeroot/usr/bin
cargo run -p oxidizr-deb -- --root /tmp/fakeroot use coreutils
```
- Stage 7 — Product: Pool manager readiness (aligns with README_LLM Stage 7)

Global flags

- `--root PATH` (default `/`): Absolute root under which all paths are scoped.
- `--commit` (default false): Perform changes; otherwise everything is dry-run.

Commands

- `use <package>`:
  - Packages: `coreutils`, `findutils`, `sudo`
  - Behavior: ensure the correct replacement is installed, plan a safe swap with backups, and apply on `--commit`.
- `restore <package|all>`:
  - Behavior: restore GNU/stock tools for the chosen package (or all) from backups.
- `status`:
  - Behavior: show current active state and what can be restored.

Advanced flags (may be behind features; see `SPEC/DEBIAN_UX.md`)

- `--assume-yes`: Skip interactive confirmation prompts.
- `--channel stable|latest`: Choose which release channel to fetch (default: stable).
- `--offline --use-local PATH`: Use a local artifact instead of fetching (still verified).
  

---

## Safety model (high level)

The CLI uses a robust engine internally to ensure safety and reversibility. Highlights:

- SafePath validation for all mutating paths under `--root`.
- Planning → preflight → apply, with backups and TOCTOU‑safe operations.
- Locking to prevent concurrent applies.
- Minimal smoke checks post‑apply; failures trigger auto‑rollback.
- Cross‑filesystem degraded fallback disallowed by default for coreutils, findutils, and sudo.

---

See the engine invariants and schemas for the underlying safety guarantees (internal documentation).

---

## Exit codes and diagnostics (v0)

- Success: `0`
- Error: `1`

In a future version we may align with a published error taxonomy (e.g., `E_LOCKING`, `E_SMOKE`) and the engine’s SPEC.

Diagnostics aim to mention the stage (preflight/apply) and a one-line cause. Debian-specific hints are surfaced where helpful (e.g., dpkg/apt locks, sudo setuid guidance).

## Debian/Ubuntu specifics

- Package manager safety: If dpkg/apt locks are detected (e.g., `/var/lib/dpkg/lock-frontend`) the CLI refuses to commit and asks you to retry.
- Sudo hardening: The replacement must be `root:root` and `chmod 4755` (setuid). Otherwise, commit will fail.
  

---

## Common workflows

- Dry-run then commit:

```bash
cargo run -p oxidizr-deb -- use coreutils
cargo run -p oxidizr-deb -- --commit use coreutils
```

- Restore back to GNU:

```bash
cargo run -p oxidizr-deb -- restore coreutils
```

- Inspect status:

```bash
cargo run -p oxidizr-deb -- status
```

---

## Troubleshooting

- "Package manager busy" — An apt/dpkg lock was detected; wait for the current operation to finish and retry.
- "sudo replacement not setuid root" — Ensure `sudo-rs` (or your replacement) is owned by root and has mode `4755`.
- "Permission denied" — Mutating in `/usr/bin` typically requires root. Try a fakeroot via `--root`, or run with elevated privileges where appropriate.
- EXDEV or degraded fallback disallowed — The policy disallows cross-filesystem degraded replacements to preserve safety.

---

## Logging & audit

- By default, the embedded engine uses a no-op JSON sink for facts.
- If you enable file-backed logging (feature `file-logging`), you can configure a JSONL path and collect auditable facts.

---

## FAQs

- Is it safe to purge legacy packages after enabling replacements?
  - Yes, if your workloads are validated. Use `oxidizr-deb --commit replace <package>` to remove/purge under guardrails
    (lock checks, confirmations). You can always `restore` later from backups if needed.

- Do I need root privileges?
  - To mutate live system paths like `/usr/bin`, yes. For development and testing, use `--root /tmp/fakeroot`.

---

## Interactive dev shell (Docker Ubuntu)

To manually verify replacements with `--version` on a real live root (inside a disposable container), use the helper script:

```bash
bash scripts/ubuntu_dev_shell.sh
```

This will:

- Start an `ubuntu:24.04` container.
- Build `oxidizr-deb` inside it.
- Apply replacements on the container’s live root:
  - `--commit use coreutils`
  - `--commit use findutils`
  - `--commit use sudo` (setuid 4755 required)
- Drop you into an interactive shell to validate:
  - `which ls && ls --version | head -n1`
  - `which find && find --version` (or `find --help` when version output is not present)
  - `which sudo && sudo --version`

Your host is never modified; all changes occur inside the disposable container.

- Can I run this on Ubuntu, Debian, or derivatives?
  - Yes. The CLI includes Debian/Ubuntu-friendly behaviors and conservative defaults.

---

## Contributing

- See `cargo/oxidizr-deb/SPEC/SPEC.md` and `cargo/oxidizr-deb/SPEC/DEBIAN_UX.md` for normative requirements.
- Planned features and test strategy live in `cargo/oxidizr-deb/PLAN/`.

---

## License

Apache-2.0 OR MIT
