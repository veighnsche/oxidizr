# oxidizr-deb — Debian/Ubuntu CLI to rustify your system safely

oxidizr-deb is a small, safety-first CLI that “rustifies” key system toolchains by swapping them to their
Rust replacements (e.g., GNU coreutils → uutils-coreutils, sudo → sudo-rs). It performs safe, atomic,
reversible changes under the hood and keeps a one-step restore path.

This CLI focuses on safety and UX:

- You do not choose applets, sources, or targets manually.
- The CLI fetches the right, verified replacement for your system and applies it safely.
- You can restore to GNU/stock tools at any time.
- After validating your workloads, you can make the swap permanent and then remove GNU packages yourself.

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

Tip: Start on a fakeroot or non-critical machine; validate your workloads before making it permanent.

---

## How oxidizr-deb fetches replacements (supply-chain safety)

oxidizr-deb automatically fetches the appropriate replacement package for your system and verifies it before use.

- Source of truth: official upstream release artifacts (e.g., GitHub Releases) or distro packages when available.
- Verification: SHA‑256 and, when provided by upstream, signature verification.
- Selection: latest stable release by default; architecture and distro layout are detected automatically.

Advanced (optional):

- `--channel stable|latest` to prefer latest stable vs. latest pre-release where applicable.
- `--offline --use-local PATH` to bypass fetching and use a local artifact you provide.

The CLI applies ownership/mode guards as needed. For example, `sudo` replacement must be `root:root` with mode `4755` when committed.

---

## CLI overview

```text
oxidizr-deb [--root PATH] [--commit] <COMMAND> [ARGS]
```

- `--root PATH` (default `/`): operate inside a root tree (use a fakeroot/chroot for safety while testing)
- `--commit`: actually perform changes (without this, it’s a dry-run preview)

Commands:

- `use <package>` — download, verify, and safely switch a package to its Rust replacement
- `restore <package|all>` — restore GNU/stock tools for a package (or all) from backups
- `status` — show what is rustified, queued, or restorable
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

# Restore (rollback) coreutils
cargo run -p oxidizr-deb -- restore coreutils
```

---

## Command semantics (clear and confident)

- `use <package>`
  - What it does: downloads (or uses `--offline --use-local`) the verified Rust replacement for `<package>`, plans a safe swap with backups, and only mutates with `--commit`.
  - Idempotence: safe to re-run; if already rustified, the plan becomes a no-op.
  - Safety: refuses to commit while apt/dpkg locks are held; runs minimal smoke checks and auto‑rolls back on failure.

- `restore <package|all>`
  - What it does: restores original GNU/stock binaries from backups and removes CLI‑managed symlinks for the chosen package(s).
  - Artifacts: replacement artifacts for restored packages are removed automatically (no clutter).
  - Idempotence: safe to re-run; if already restored, the plan becomes a no‑op.

- `status`
  - What it does: reports which packages are rustified, which are restorable, and where backups/artifacts live.

---

## Glossary

- Replacement artifact: downloaded Rust implementation for a package (e.g., uutils‑coreutils, uutils‑findutils, sudo‑rs).
- Symlink topology: set of links so standard commands resolve to the replacement binary.
- Backup sidecar: saved original binaries/links created for restoration.
- Use: safely switch a package to its Rust replacement (download, verify, link with backups).
- Restore: switch back to GNU/stock binaries using backups; remove CLI‑managed symlinks.
- Make permanent (seal): protect against package‑manager overwrites and/or remove GNU packages after validation.

---

## Permanence

After a successful `use <package> --commit`, oxidizr‑deb keeps your selection active across upgrades. No extra steps are required.

---

## Cleanup

The CLI automatically removes replacement artifacts for a package after a successful `restore`. No manual cleanup is needed.

Tip: to experiment safely, use a fakeroot:

```bash
sudo mkdir -p /tmp/fakeroot/usr/bin
cargo run -p oxidizr-deb -- --root /tmp/fakeroot use coreutils
```

---

## Full command and option reference

Global flags

- `--root PATH` (default `/`): Absolute root under which all paths are scoped.
- `--commit` (default false): Perform changes; otherwise everything is dry-run.

Commands

- `rustify <package>`:
  - Packages: `coreutils`, `findutils`, `sudo`
  - Behavior: fetch + verify the correct replacement, plan a safe swap with backups, and apply on `--commit`.
- `restore <package|all>`:
  - Behavior: restore GNU/stock tools for the chosen package (or all) from backups.
- `status`:
  - Behavior: show current rustified state and what can be restored.

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
cargo run -p oxidizr-deb -- rustify coreutils
cargo run -p oxidizr-deb -- --commit rustify coreutils
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
  - The CLI keeps your system functional by repointing applets to replacements and preserving backups. Removal of packages is your decision and should be made after verifying your workload.

- Do I need root privileges?
  - To mutate live system paths like `/usr/bin`, yes. For development and testing, use `--root /tmp/fakeroot`.

- Can I run this on Ubuntu, Debian, or derivatives?
  - Yes. The CLI includes Debian/Ubuntu-friendly behaviors and conservative defaults.

---

## Contributing

- See `cargo/oxidizr-deb/SPEC/SPEC.md` and `cargo/oxidizr-deb/SPEC/DEBIAN_UX.md` for normative requirements.
- Planned features and test strategy live in `cargo/oxidizr-deb/PLAN/`.

---

## License

Apache-2.0 OR MIT
