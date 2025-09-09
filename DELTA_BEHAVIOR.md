# Product Behavior Documentation (Expected) — oxidizr-arch

Authoritative, implementation-agnostic specification for the Rust product under `src/`. Mirrors the structure of the current behavior document for easy side-by-side review. “Delta vs CURRENT” lines call out the exact differences from today’s behavior. (See the live current doc for reference.)

## High-level overview

The binary `oxidizr-arch` switches selected system tools to Rust replacements by **installing provider packages (pacman/AUR)** and performing **link-aware, atomic** backup/symlink/restore operations. Experiments are **explicitly selected**; there are **no default experiments**.

**Delta vs CURRENT:** CURRENT enables a default set (`coreutils` + `sudo-rs`) when no selector is provided; this spec removes defaults and requires explicit selection.

---

## Binaries, library, features

* Crate: `oxidizr_arch`; Binary: `oxidizr-arch`.
* No feature gating of core behavior at runtime.
* Any compile-time assets used at runtime (e.g., applet lists) live under `src/assets/…`.

**Delta vs CURRENT:** CURRENT loads coreutils applet list from `tests/lib/rust-coreutils-bins.txt`; expected location is under `src/assets/…`.

---

## CLI inputs and subcommands

**Global flags (normalized):**

* Keep: `--assume-yes`, `--no-update`, `--dry-run`, `--experiments` (multi), `--all` (optional aggregator), `--bin-dir`, `--unified-binary`, `--package` (per experiment), `--aur-helper=auto|none|paru|yay|trizen|pamac`, `--aur-user=<name>`, `--skip-compat-check`, `--wait-lock` (kebab-case).
* Remove/Dedupe: drop `--no-compatibility-check`; drop `--package-manager`; rename `--wait_lock` → `--wait-lock`.

**Subcommands:**

* `enable` — flip to provider via link-aware atomic ops.
* `disable` — restore from backups; **never uninstalls**.
* `remove` — `disable` then uninstall providers.

**Delta vs CURRENT:**

* CURRENT has duplicate flags (`--skip-compatibility-check` & `--no-compatibility-check`; `--aur-helper` & `--package-manager`), and `disable` can uninstall by default under `--assume-yes`. Expected CLI dedupes flags and makes `disable` restore-only.

---

## Experiments and their behaviors

### Execution order (when `--all`):

`findutils` → `coreutils` → `sudo-rs` → `checksums`.

### Coreutils

* Provider: `uutils-coreutils` (repo-gated).
* **Presence-aware** plan excludes checksum applets (those belong to `checksums`).
* Per-applet symlinks or unified dispatcher; link-aware backups; atomic swaps.

**Delta vs CURRENT:** Behavioral intent is the same; expected doc formalizes link-aware backups & atomicity guarantees.

### Findutils

* Provider: **AUR** `uutils-findutils-bin` (AUR is required).
* Install via detected helper (`--aur-helper`), optionally as `--aur-user`; fail with guidance if helper/user invalid.
* **No provider synthesis by default** (no copying binaries into “canonical” dirs). If discovery yields nothing post-install → see Exit code `20`.

**Delta vs CURRENT:** CURRENT may “synthesize canonical sources” by copying binaries to a canonical bin dir; expected behavior removes this by default.

### Sudo-rs

* Provider: `sudo-rs` (repo-gated).
* Creates stable alias symlinks and flips `/usr/bin/sudo`, `/usr/bin/su`, `/usr/sbin/visudo` via link-aware ops; restores faithfully.

**Delta vs CURRENT:** No functional delta; expected doc re-states link-aware restore.

### Checksums

* Discovers `{b2sum, md5sum, sha1sum, sha224/256/384/512sum}` from unified/per-applet dirs.
* Only discovered applets are linked; missing → `skip_applet` (WARN).
* If *none* discovered even after ensuring provider installed → **exit code `20`** (`nothing_to_link`) with guidance.

**Delta vs CURRENT:** CURRENT returns success on “nothing found”; expected behavior returns distinct non-zero (or at minimum emits a prominent WARN + summary).

---

## Package and repository behaviors

* `update_packages`: optional `pacman -Sy` honoring `--no-update` and `--wait-lock`.
* **Repo gating** (`coreutils`, `sudo-rs`): require `extra` + `pacman -Si <pkg>` success; else **hard fail** with structured event `repo_gate_failed{required_repo,pkg,checks[…]}`
* **AUR** (`findutils`): helper discovery via `--aur-helper`; **no hardcoded `builder` assumption**. If `--aur-user` provided, validate user; else run as invoking user. Pass non-interactive flags when `--assume-yes`.

**Delta vs CURRENT:** CURRENT may run helpers as `su - builder -c …` and relies on heuristic helper ordering and `--package-manager`; expected behavior validates/parameterizes the user and removes `--package-manager`.

---

## Distro and compatibility

* Supports Arch-family IDs (`arch`, `endeavouros`, `cachyos`, `manjaro`), unless `--skip-compat-check` is used.
* Incompatibility → `Incompatible` error (non-zero).

**Delta vs CURRENT:** No functional delta (naming cleanup for flags applies).

---

## Symlink and backup behavior

* **Link-aware backups:**

  * If target is a **symlink** → back up the **link itself** (record link target & link metadata). Restore recreates the **symlink**.
  * If target is a **regular file** → copy to backup (preserve metadata), then replace with symlink.
* **Atomic swaps:** temp path + atomic rename; fsync parent dirs for critical steps.
* **On restore:** remove current target; restore from the backup (symlink or file accordingly).
* **Missing backup for a modified target:** **error + non-zero** (unless `--force-restore-best-effort`).

**Delta vs CURRENT:** CURRENT backs up the resolved destination of a symlink (can restore as a regular file) and only warns on missing backup; expected behavior is link-aware and fails missing-backup.

---

## Logging and audit behavior

* **Verbosity (human logs via `VERBOSE` env):** `0=ERROR`, `1=INFO` (default), `2=DEBUG`, `3=TRACE`.
* **Structured audit JSONL** (independent of TTY):
  `enabled`, `removed_and_restored`, `link_started`, `link_done`, `backup_created`, `restore_started`, `restore_done`, `skip_applet{name,reason}`, `package_install`, `package_remove`, `repo_gate_failed{…}`, `nothing_to_link`.

**Delta vs CURRENT:** CURRENT emits structured audit but lacks some first-class events and can suppress INFO per-item logs under a progress bar; expected behavior never suppresses structured events and adds explicit events.

---

## Progress and UI behavior

* Progress bar on **TTY**; `--no-progress` disables (CI-friendly).
* Structured events always emitted; human INFO suppression must **not** affect audit JSONL.

**Delta vs CURRENT:** Aligns intent but makes “structured never suppressed” explicit; adds `--no-progress` knob (and retains honoring non-TTY).

---

## Outputs and side effects (filesystem/process)

* Backups: `.<name>.oxidizr.bak` next to targets.
* Symlinks at standard paths (`/usr/bin/*`, `/usr/sbin/visudo`) pointing at providers.
* Audit JSONL at stable path (system or user fallback).
* External commands limited to pacman/AUR/helper calls and discovery (`which`, `pacman-conf`).

**Delta vs CURRENT:** No change besides link-aware restore guarantee.

---

## Error model

* Key errors: `ExecutionFailed`, `Io`, `Incompatible`.
* Unused variants pruned (or wired): remove `CommandNotFound`, `InvalidImplementation` if not used.

**Delta vs CURRENT:** CURRENT declares unused variants; expected behavior removes or uses them.

---

## Invariants and safety constraints

* Presence-aware linking (no thresholds).
* `coreutils` must never flip checksum applets (those are in `checksums`).
* `remove` refuses when checksum targets are still linked; instruct to disable `checksums` first.
* Root required unless `--dry-run`.

**Delta vs CURRENT:** Same intent; explicitly presence-aware without counts.

---

## Environment variables

* `VERBOSE` controls human logs; progress can be influenced by `--no-progress` (prefer flags over env toggles).

**Delta vs CURRENT:** CURRENT lists multiple env knobs to force progress; expected favors the flag and allows existing envs as secondary.

---

## Observability and auditability

* Every mutation emits structured events with timings and paths.
* Secrets are redacted by a basic masker in audit.
* Exit codes are **enumerated** (see below) for precise assertions.

**Delta vs CURRENT:** CURRENT does not enumerate distinct exit codes for special cases; expected adds them.

---

## List-targets and check outputs

* `list-targets`: `<experiment>\t<absolute-target-path>` (resolved from product assets).
* `check`: `<experiment>\tCompatible: <true|false>`.

**Delta vs CURRENT:** Same format; source for lists moves to `src/assets`.

---

## Notes drawn from README alignment

* README must reflect: no default experiments; link-aware backups; AUR requirement for findutils; repo gating; enumerated exit codes.

**Delta vs CURRENT:** CURRENT README implies default experiments and different backup semantics; expected updates these.

---

## Known interactions and constraints

* Repo gating failures → hard fail with `repo_gate_failed`.
* Findutils requires AUR; absence of helper/user → fail with guidance.
* Checksums zero-discovery → exit code `20` (`nothing_to_link`) with WARN + guidance.

**Delta vs CURRENT:** CURRENT allows success on zero-discovery; expected returns distinct non-zero (or at least prominent WARN).

---

## Non-goals

* Product does **not**: normalize repos/mirrors, set locales, install alternate toolchains, or alter host/runner logging.
* Product does **not**: mutate `/usr/bin/*` outside enable/disable/remove subcommands.

**Delta vs CURRENT:** Same intent; clarified explicitly.

---

## Exit codes (enumerated)

* `0`  — success (performed intended actions).
* `1`  — general failure.
* `10` — incompatible distro (unless skipped).
* `20` — `nothing_to_link` after provider ensured.
* `30` — `restore_backup_missing` (without `--force-restore-best-effort`).
* `40` — `repo_gate_failed`.

**Delta vs CURRENT:** CURRENT doesn’t define special codes for these cases; expected adds them for testability.
