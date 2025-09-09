# Production-Minimum Task List — **oxidizr-arch**

*Bare-minimum changes I need to feel safe shipping to real machines.*

---

## 1) Pacman post-transaction relink hook (clobber-proofing)

**Goal:** Prevent package upgrades from overwriting our managed symlinks.

* Create `/usr/share/libalpm/hooks/oxidizr-arch-relink.hook` that runs:

  ```bash
  Exec = /usr/bin/oxidizr-arch enable --assume-yes --no_update --no_progress
  ```

  …and scope it to only the **previously enabled experiments** (read from a state file; see Task 6).
* Verify: Upgrade `coreutils` or `findutils` → our links remain active.

---

## 2) `sudo-rs` post-enable verifier (setuid, ownership, PAM)

**Goal:** Ensure sudo actually works and is safe after swap.

* After linking `sudo|su|visudo`, verify on the **real binary**:

  * `uid=0,gid=0`, `mode & 04000 != 0` (setuid).
  * `/etc/pam.d/sudo` exists.
  * Smoke test: `sudo -n true` returns 0 for a sudoer user.
* If any check fails: **abort enable**, revert changes, print remediation.

---

## 3) Race-safe, no-follow filesystem operations

**Goal:** Eliminate TOCTOU and symlink-trick vulns in `/usr`.

* Replace file ops with `openat`/`fstatat(AT_SYMLINK_NOFOLLOW)` and `renameat`.
* Reject if **target or parent dir** is a symlink.
* Keep swaps **atomic within the same directory** (temp symlink + `renameat`).

---

## 4) AUR preflight for `findutils`

**Goal:** Fail early with actionable guidance; avoid half-installed states.

* Before any AUR attempt, require: `base-devel` group, `git`, `fakeroot`, `makepkg`.
* Under `--assume-yes`, auto-install; otherwise print exact `pacman` command and abort.

---

## 5) Writable/exec + immutability preflight

**Goal:** Don’t attempt changes on read-only or immutable targets.

* Confirm mount flags for `/usr` include `rw,exec`.
* For each target, reject if `chattr +i` (immutable) is set; show `chattr -i` hint.

---

## 6) Persist minimal state + targeted relink + final state report

**Goal:** Know what we manage; relink only what we own; give ops visibility.

* Write `/var/lib/oxidizr-arch/state.json`:

  * enabled experiments, managed target list, timestamp.
* Hook (Task 1) reads this to relink only those.
* After every run, emit `/var/log/oxidizr-arch/state-report.txt` listing each managed path as `regular|symlink|missing` (+ link target).

---

## 7) Package owner verification (default: warn; strict mode: block)

**Goal:** Avoid swapping files not owned by expected packages.

* For each target, run `pacman -Qo`.
* If owner is unexpected:

  * default: warn prominently and continue,
  * with `--strict-ownership`: **abort**.

---

## 8) Single-instance process lock

**Goal:** Prevent concurrent runs from corrupting backups/links.

* Acquire `flock` on `/run/lock/oxidizr-arch.lock` (or similar) at startup.
* If locked: exit with clear “another instance running” message.

---

## 9) Source path trust checks for overrides

**Goal:** Don’t link to untrusted binaries via `--bin_dir`/`--unified_binary`.

* Reject sources that are world-writable, on `noexec`, owned by non-root, or under `$HOME` unless `--force` is set.
* Log the resolved absolute source for every applet.

---

## 10) Removal guard already present → keep it strict

**Goal:** Don’t leave dangling checksum links.

* Keep (and test) the **block** on `remove coreutils` when checksum applets are still linked. Error with explicit remediation.

---

### Ship Criteria (must all pass)

* Relink hook prevents pacman clobbering in upgrade tests.
* `sudo-rs` verifier blocks unsafe states; `sudo -n true` works post-enable.
* FS ops are no-follow/atomic; symlink-race tests safe.
* AUR preflight enforced (or auto-installed under `-y`).
* RO/immutable preflights abort with clear messages.
* State persisted; hook relinks only managed set; state report matches reality.
* Owner verification works (warn by default, block with `--strict-ownership`).
* Concurrency lock prevents parallel runs.
* Override trust checks enforced (or bypassed only with `--force`).
