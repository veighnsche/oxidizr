# 0) Scope & Terminology

* **Experiments**: `coreutils`, `findutils`, `checksums`, `sudo-rs`.
* **Commands**: `enable`, `disable`, `remove`, `check`, `listtargets`.
* **Global flags**: `--assume-yes/-y`, `--no_update`, `--all`, `--experiments`, `--experiment`, `--skip-compat-check`, `--aur_helper`, `--aur_user`, `--package`, `--bin_dir`, `--unified_binary`, `--dry_run`, `--wait_lock`, `--no_progress`, `--force_restore_best_effort`.
* **Backups**: `/.<name>.oxidizr.bak` in same dir as target.
* **Exit codes** (verify mapping): `Incompatible → 10`, `NothingToLink → 20`, `RestoreBackupMissing → 30`, `RepoGateFailed → 40`, others → `1`, success → `0`.

Test on Arch-based images (Arch, EndeavourOS, CachyOS, Manjaro) and one unsupported (Ubuntu/Debian).

---

# 1) CLI Parsing & Help

**T-CLI-01** Help/usage

* **Action**: `oxidizr-arch -h` and `oxidizr-arch --help`.
* **Expect**: Usage text lists commands and all flags with correct names/aliases and descriptions. Exit `0`.

**T-CLI-02** Version

* **Action**: `oxidizr-arch -V/--version`.
* **Expect**: Version string; exit `0`.

**T-CLI-03** Unknown subcommand / flag

* **Action**: typos (e.g., `enbale`, `--assume_yesz`).
* **Expect**: Clap error, usage shown, non-zero exit (typically `1`).

**T-CLI-04** Experiment selection precedence

* **Matrix**: `--all`, `--experiments a,b`, repeated `--experiments`, `--experiment x` (deprecated), combinations.
* **Expect**:

  * `--all` selects all; others ignored.
  * Multiple `--experiments` merged.
  * `--experiment` honored only when others absent.
  * Unknown names → “no experiments matched selection” error, exit `1`.

**T-CLI-05** Mutual exclusivity validation

* **Action**: N/A (informational).
* **Expect**: If code enforces any exclusive flags, verify errors; else no conflict.

---

# 2) Permissions & Root Requirements

**T-PERM-01** Root required (non-dry-run)

* **Matrix**: commands `enable|disable|remove` under non-root, with and without `--dry_run`.
* **Expect**:

  * Non-root & not `--dry_run` → error “must be run as root”, exit `1`.
  * With `--dry_run` → allowed to proceed, no FS changes.

---

# 3) Distro Compatibility

**T-COMP-01** `check` reports compatibility

* **Matrix**: Arch, EndeavourOS, CachyOS, Manjaro, Ubuntu.
* **Action**: `check --experiments coreutils,findutils,checksums,sudo-rs`.
* **Expect**: Supported distros → `Compatible: true`; unsupported → `false`. Exit `0`.

**T-COMP-02** `enable` blocks on unsupported distro

* **Action**: On Ubuntu, `enable --experiments coreutils`.
* **Expect**: `Incompatible` error, exit `10`.
* **Variant**: With `--skip-compat-check` → proceeds to next stages.

---

# 4) Pacman/AUR Pre-requisites & Repo Gates

**T-REPO-01** Official repo availability (coreutils, sudo-rs)

* **Setup**: Disable `[extra]` or simulate `pacman -Si` failure.
* **Action**: `enable coreutils` / `enable sudo-rs`.
* **Expect**: `RepoGateFailed`, exit `40`.

**T-REPO-02** AUR helper presence (findutils)

* **Matrix**: `paru`, `yay`, `trizen`, `pamac`, none; `--aur_helper auto|paru|yay|trizen|pamac|none`; `--aur_user userX`.
* **Action**: `enable findutils`.
* **Expect**:

  * With helper present → install path used (verify called binary & args, `--batchinstall` for `paru` when `-y`).
  * With `--aur_user` → executed via `su - <user>`.
  * No helper & `--aur_helper none` → error “no AUR helper”, exit `40`.

**T-REPO-03** Package already installed (reuse prompt)

* **Setup**: Pre-install `uutils-coreutils` or `uutils-findutils-bin` or `sudo-rs`.
* **Matrix**: `--assume-yes` vs interactive; answer `Y`/Enter vs `n`.
* **Expect**:

  * Reuse accepted → skip reinstall; logs audit of reuse.
  * Reinstall requested → attempt reinstall.

**T-REPO-04** `--no_update` behavior

* **Action**: with/without `--no_update`, intercept `pacman -Sy`.
* **Expect**: Update skipped when set.

**T-REPO-05** Pacman DB lock handling

* **Setup**: Hold `/var/lib/pacman/db.lck`.
* **Matrix**: `--wait_lock` absent, `--wait_lock=0`, `--wait_lock=5`.
* **Expect**:

  * No wait flag → immediate failure.
  * Wait with timeout → retries until timeout; success if lock clears; failure otherwise (error message includes timeout).

---

# 5) Dry-Run & Progress Output

**T-DRY-01** Dry-run doesn’t change system

* **Action**: For each experiment `enable|disable|remove` with `--dry_run`.
* **Expect**: No FS modifications or package changes; commands printed/logged; exit `0`.

**T-UI-01** `--no_progress` suppresses progress bars

* **Action**: Link/restore operations with and without `--no_progress`.
* **Expect**: No progress UI when set; per-item info logs present instead.

---

# 6) Discovery of Applets/Binaries

## 6.1 coreutils discovery

**T-DISC-CU-01** Unified dispatcher path

* **Setup**: Provide `/usr/bin/coreutils` (or override via `--unified_binary`).
* **Action**: `enable coreutils`.
* **Expect**: All applets discovered via unified binary; **exclude checksum applets** (see preservation); logs show unified mode used.

**T-DISC-CU-02** Per-applet discovery

* **Setup**: No unified binary; provide applets in:

  * override `--bin_dir`
  * `/usr/bin/uu-<name>`
  * `/usr/lib/cargo/bin/coreutils/<name>`
  * `/usr/lib/cargo/bin/<name>`
  * PATH
* **Expect**: Found in the first location that matches per applet. Missing applets skipped. If **none found** → `NothingToLink`, exit `20`.

**T-DISC-CU-03** Partial availability

* **Setup**: Some applets present, some not.
* **Expect**: Only present applets linked; skipped ones logged; overall success if at least one linked.

## 6.2 findutils discovery

**T-DISC-FU-01** Find/xargs present

* **Setup**: After install, provide `find`, `xargs` in expected places.
* **Expect**: Both discovered; 2 links created.

**T-DISC-FU-02** One missing

* **Setup**: Only one present.
* **Expect**: Missing one warned; link created for found; if **none found** → `NothingToLink`, exit `20`.

**T-DISC-FU-03** Preflight sha256sum warning

* **Setup**: Remove `sha256sum` from PATH before AUR build.
* **Expect**: Warns about makepkg checksum preflight; still attempts install.

## 6.3 sudo-rs discovery

**T-DISC-SR-01** Locate binaries

* **Setup**: Provide `sudo-rs` binaries via any of: `/usr/lib/cargo/bin/<name>`, `/usr/bin/<name>-rs`, PATH `<name>-rs`.
* **Expect**: For each of `sudo|su|visudo`, found source path. Missing any → error with specific message.

---

# 7) Symlink Creation & Backup Semantics

**T-LINK-01** Atomic replace & backup creation

* **Setup**: For a target (e.g., `/usr/bin/ls`) that is a regular file.
* **Action**: Link operation.
* **Expect**:

  * `/.ls.oxidizr.bak` created, original moved there.
  * `/usr/bin/ls` becomes symlink to source (or to alias for sudo-rs).
  * Ownership/permissions on backup preserved; new symlink has sane default perms.

**T-LINK-02** Target already a symlink

* **Setup**: `/usr/bin/ls` is symlink to **somewhere else**.
* **Action**: Link to new source.
* **Expect**: Resolved cleanly—either replaced or left if already pointing correctly; no double-symlink loops; backup semantics documented (backup of previous target path).

**T-LINK-03** Idempotent linking

* **Action**: Run `enable` twice.
* **Expect**: Second run is no-op (links already correct); no duplicate backups; clean exit `0`.

**T-LINK-04** Error propagation

* **Setup**: Make target dir read-only or deny permissions.
* **Action**: Link.
* **Expect**: Fails with informative error; exit `1`; no partial corruption (target state unchanged).

**T-LINK-05** Path traversal safety

* **Setup**: Malicious `--bin_dir` or `--unified_binary` with `..` segments.
* **Expect**: Normalization and safe handling; no writes outside intended dirs.

---

# 8) Special Linking Rules per Experiment

## 8.1 coreutils: checksum preservation

**T-CU-CHK-01** Preserve checksum tools

* **Setup**: Normal coreutils enable.
* **Expect**: `b2sum md5sum sha1sum sha224sum sha256sum sha384sum sha512sum` **not** linked by coreutils enable (left untouched).

**T-CU-CHK-02** Interaction with checksums experiment

* **Action**: After coreutils enable, run `enable checksums`.
* **Expect**: Only now are checksum tools linked to Rust (if present).

## 8.2 sudo-rs: two-level linking

**T-SR-LINK-01** Alias creation

* **Action**: `enable sudo-rs`.
* **Expect**:

  * `/usr/bin/sudo.sudo-rs` → symlink to actual new sudo binary.
  * `/usr/bin/sudo` → symlink to `/usr/bin/sudo.sudo-rs`.
  * Same for `su` and `visudo` (note `visudo` target is `/usr/sbin/visudo`).
  * Post-verify that the *final* target resolves to real binary; intermediate alias exists.

---

# 9) Disable (Restore) Behavior

**T-REST-01** Normal restore

* **Setup**: After any successful enable.
* **Action**: `disable` for that experiment.
* **Expect**:

  * Symlinks removed, backups renamed back into place.
  * Targets become **regular files** (not symlinks).
  * Exit `0`.

**T-REST-02** Missing backup (default strict)

* **Setup**: Remove a `.oxidizr.bak` file.
* **Action**: `disable`.
* **Expect**: Fails with `RestoreBackupMissing`, exit `30`.

**T-REST-03** Missing backup (best-effort)

* **Action**: `disable --force_restore_best_effort`.
* **Expect**: Warns and continues; leaves file as-is for missing cases; overall exit `0`.

**T-REST-04** Idempotent restore

* **Action**: `disable` twice.
* **Expect**: Second run reports nothing to restore; exit `0`.

**T-REST-05** sudo-rs verification

* **Action**: `disable sudo-rs`.
* **Expect**: After restore, `sudo|su|visudo` are **not** symlinks (explicit check). Any still-symlink → error.

---

# 10) Remove (Restore + Uninstall)

**T-RM-01** remove coreutils blocks if checksums linked

* **Setup**: Enable `checksums`, then attempt `remove coreutils`.
* **Expect**: Refuses with clear message to disable checksums first; exit `1`.

**T-RM-02** remove after disable

* **Action**: `disable coreutils` then `remove coreutils`.
* **Expect**: Package removed via pacman; verify not installed; exit `0`.

**T-RM-03** remove findutils / sudo-rs happy path

* **Expect**: Restored + package uninstalled; not installed post-check.

**T-RM-04** remove when package already absent

* **Setup**: Uninstall package manually.
* **Action**: `remove` again.
* **Expect**: Skips removal gracefully; still restores; exit `0`.

**T-RM-05** Dry-run remove

* **Expect**: No uninstall performed; actions logged.

---

# 11) `check` Command Details

**T-CHK-01** Per-experiment result formatting

* **Action**: `check --experiments <each>` and `--all`.
* **Expect**: One line per experiment: `<name>\tCompatible: <true|false>`. Exit `0`.

---

# 12) `listtargets` Output

**T-LT-01** coreutils target list excludes checksums

* **Expect**: Full list of applets except checksum tools.

**T-LT-02** findutils target list

* **Expect**: Exactly `/usr/bin/find` and `/usr/bin/xargs`.

**T-LT-03** sudo-rs target list

* **Expect**: `/usr/bin/sudo`, `/usr/bin/su`, `/usr/sbin/visudo`.

**T-LT-04** checksums target list

* **Expect**: All checksum applets’ `/usr/bin/<name>`.

---

# 13) State Machine & Invariants

**T-INV-01** Reversibility

* **Sequence**: `enable` → verify → `disable` → verify → `enable` → verify.
* **Expect**: Bitwise equality of original files restored after first disable (hash/size/perm/owner), and symlink states correct after re-enable.

**T-INV-02** Non-destructive removals

* **Sequence**: `enable` → `remove`.
* **Expect**: Originals restored; provider package uninstalled (except checksums); no leftover altered regular files (only harmless alias symlinks for sudo-rs may remain, but originals not symlinks after disable).

**T-INV-03** Filesystem permissions/ownership

* **Expect**: Backups retain original uid/gid/mode; restored files match originals’ metadata.

**T-INV-04** No orphan backups

* **Action**: After successful disable/remove, scan for `.oxidizr.bak`.
* **Expect**: None remaining for targets that were restored (unless best-effort skipped).

---

# 14) Error Handling & Exit Codes

**T-ERR-01** Incompatible → exit 10
**T-ERR-02** NothingToLink → exit 20
**T-ERR-03** RestoreBackupMissing → exit 30
**T-ERR-04** RepoGateFailed → exit 40
**T-ERR-05** Generic failures (fs, command exec) → exit 1

* **Action**: Force each error path; verify exit code mapping and message text.

---

# 15) Logging & Audit

**T-LOG-01** Initialization log & environment echo

* **Expect**: Startup logs include parsed args (excluding secrets), distro, dry-run, no\_progress, wait\_lock.

**T-LOG-02** Audit events around critical ops

* **Expect**: Events for: repo gate checks, package install/remove attempts (pacman/AUR), symlink start/done, restore start/done, prompts and responses.

**T-LOG-03** Prompt text

* **Expect**:

  * `enable` confirmation prompt when not `-y`.
  * Reuse-installed prompt for already-installed package unless `-y`.
  * Correct default answers (Enter == default).

**T-LOG-04** `--no_progress` vs normal

* **Expect**: With progress disabled, individual per-file info logs appear.

---

# 16) Overrides & Customization Flags

**T-OVR-01** `--package` override

* **Action**: Point to alt package names.
* **Expect**: That name used for install/remove and repo checks; errors if not found.

**T-OVR-02** `--bin_dir` / `--unified_binary` override

* **Expect**: Discovery prefers override; falls back otherwise; errors when override path invalid.

**T-OVR-03** `--aur_user`

* **Expect**: AUR helper executed under specified user via `su -`; failure if user missing.

---

# 17) Concurrency & Race Conditions

**T-RACE-01** Pacman lock contention (see T-REPO-05).
**T-RACE-02** Concurrent runs protection

* **Setup**: Start two `enable` processes on same experiment.
* **Expect**: One fails gracefully or both serialize via pacman lock and fs ops—no corrupted state, backups consistent.

**T-RACE-03** Power failure simulation

* **Setup**: Kill process mid-link.
* **Expect**: On next run, either restore or complete; no orphaned temp files; no broken targets without backup.

---

# 18) Negative/Edge FS States

**T-EDGE-01** Target missing before enable

* **Setup**: Remove `/usr/bin/<target>` before linking.
* **Expect**: Link still succeeds (no backup created); disable should then fail with missing backup unless best-effort.

**T-EDGE-02** Backup exists but target already symlink to correct source

* **Expect**: No duplicate backup; operation idempotent.

**T-EDGE-03** Non-standard files (FIFO/socket) at target

* **Expect**: Meaningful error; no destructive change.

---

# 19) Performance & Scale

**T-PERF-01** coreutils large link set

* **Action**: Measure total runtime for linking all coreutils applets; ensure within acceptable bound (define threshold).
* **Expect**: Progress bar updates smoothly; no stalls.

---

# 20) Security

**T-SEC-01** Shell injection resistance

* **Setup**: Malicious values in `--package`, `--aur_user`, overrides.
* **Expect**: Used as argv (no shell), safe; failure only if invalid inputs.

**T-SEC-02** Privilege boundaries

* **Expect**: Only AUR helper runs under `--aur_user`; pacman invocations require root; no unintended privilege drops/escalations.

**T-SEC-03** Path trust

* **Expect**: Sources resolved to actual files within expected locations; no linking to world-writable or suspicious paths.

---

# 21) Cross-Experiment Interactions & Order

**T-XP-01** Ordered enable for `findutils` then `coreutils`

* **Action**: `enable --all` or both; ensure order is `findutils` → `coreutils`.
* **Expect**: Log mentions order; builds succeed (checksum tools available during AUR).

**T-XP-02** Mixed operations

* **Sequence**: `enable coreutils` → `enable checksums` → `disable coreutils` (expect block? no—checksums independent) → verify checksum symlinks still active.
* **Sequence**: Attempt `remove coreutils` while checksums enabled → blocked (see T-RM-01).
* **Sequence**: `remove checksums` (alias of disable) → then `remove coreutils` → success.

---

# 22) Artifacts to Capture Per Test

* Command line, return code, stdout/stderr.
* Structured logs & audit log (timestamps, event types).
* File snapshots (before/after): presence, type, owner, mode, inode, link target; backup files.
* Package database state (`pacman -Qi`, `-Qs`).
* Distro info snapshot (`/etc/os-release`).
* Timing for performance tests.

---

# 23) Test Environments & Fixtures

* **Containers/VMs**: Arch, EndeavourOS, CachyOS, Manjaro, Ubuntu.
* **Users**: root, non-root, `aur_user` account.
* **Helpers**: Install `paru`, `yay`, etc., or purposefully omit.
* **Repo toggles**: Enable/disable `[extra]`.
* **Lock simulator**: Process that holds `db.lck`.
* **FS harness**: Helpers to create/remove targets/backups, flip perms, simulate read-only dirs.
* **Kill-switch**: Utility to terminate process mid-operation.

---

# 24) Idempotency & Round-Trip Suites

* **Suite A**: For each experiment: `enable → disable → enable → remove`. Validate invariants every step.
* **Suite B**: Global: `--all enable → listtargets snapshot → check → disable --all → remove --all`.
