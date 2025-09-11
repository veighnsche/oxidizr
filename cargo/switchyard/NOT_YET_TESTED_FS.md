# Filesystem Matrix Testing — Deferred

This document explains why **Switchyard** does not yet run automated tests across multiple filesystems (ext4, btrfs, xfs, tmpfs, overlayfs).

---

## Background

* **Requirement:** SPEC v1.1 §F3 (“Supported filesystems verified”) requires that Switchyard be tested on at least one journaling filesystem (ext4) and one copy-on-write or alternative FS (e.g. btrfs, xfs).
* **Rationale:** Atomic swap semantics (renameat + fsync) differ between filesystems and mount options. Testing ensures that atomicity, rollback, and degraded EXDEV fallback work correctly under real conditions.

---

## Current Status

* **Unit & integration tests** run only on the CI host’s default filesystem (ext4 on GitHub runners).
* No automated coverage yet for btrfs, xfs, overlayfs, tmpfs, or union filesystems.
* Manual spot-tests were done locally on ext4 and tmpfs only.

---

## Why not tested yet?

1. **CI environment limits:**

   * GitHub Actions runners support `sudo`, but privileged loop-mount orchestration requires extra setup (mkfs tools, losetup, unmount cleanup).
   * The current `.github/workflows/ci.yml` is kept minimal to prioritize fast determinism & golden checks.

2. **Cost constraints:**

   * No budget for paid CI or hosted servers with persistent block devices.
   * Self-hosted runners not yet available.

3. **Risk management:**

   * Gaps are documented here; other high-risk blockers (rollback topology, idempotency, lock enforcement, rescue profile, smoke health, golden diff gates) are being addressed first.
   * F3 is acknowledged as a **production blocker**, but deferred until infra is in place.

---

## Plan

* Add `tests/fs-matrix.sh` harness using loopback sparse files.
* Run minimal swap/rollback cycle under ext4, btrfs, xfs, tmpfs.
* Expose mount root via `SWITCHYARD_FS_UNDER_TEST` env var to integration tests.
* Wire into GitHub Actions as a **separate job**:

  * Required on `main` and nightly.
  * Optional / non-blocking on PRs to keep CI fast.

---

## Definition of Done

* Loop-mounted FS matrix test job runs on GitHub Actions Ubuntu runners.
* At least ext4 and one additional FS (btrfs or xfs) must pass round-trip tests.
* Facts record `fs_type` for debugging.
* SPEC v1.1 REQ-F3 marked complete; `SPEC_CHECKLIST.md` updated.

---

## 6) REQ‑F3 Supported filesystems verified — Blocker

Objective: Demonstrate Switchyard works on a representative set of filesystems (at least ext4 and one additional such as btrfs or xfs) using the EXDEV/degraded path as applicable.

Plan:

* Add end‑to‑end tests that run the symlink swap + restore cycle on loop‑mounted filesystems within the Docker container (privileged job in the test orchestrator):
  * Create sparse files, `mkfs.ext4`, `mkfs.btrfs` (or `mkfs.xfs` when available), mount them at `/mnt/fs_under_test`, and run a small Rust test binary (or use the library test with `#[ignore]` + YAML runner) to execute `replace_file_with_symlink()` and `restore_file()` there.
  * Capture facts and ensure `degraded` flag is correctly set on EXDEV fallbacks and that operations succeed.
* Emit filesystem type in facts for visibility during testing (from `preflight` using `/proc/mounts` or `statfs`).

Code changes:

* Optional: `src/api/fs_meta.rs` helper to fetch fs type for diagnostics; include in apply facts when available.

Tests/Orchestrator:

* Add YAML tasks under `tests/` or `test-orch/container-runner` to perform the mount lifecycle (requires root inside the container). Use the existing orchestrator (`test-orch/`) to run these steps safely and in isolation.
* Mark as mandatory in CI matrix for at least ext4; btrfs/xfs can be additional but should be part of a scheduled job if flaky.

Acceptance criteria:

* F3 checked off; fixture evidence captured; docs updated (`SPEC/features/fs.md` if needed).

Estimate: 1.0–1.5 days depending on orchestrator plumbing.

Risks/Mitigations:

* Docker base image may lack `mkfs.*`; ensure the container runner installs needed packages (`e2fsprogs`, `btrfs-progs`, `xfsprogs`) for the test phase only.

Feasibility & Complexity:

* Feasibility: Medium — product code is ready; infra changes needed for privileged mounts.
* Complexity: High — requires privileged container runs and additional packages in the image plus CI/orchestrator wiring.
* Evidence:
  * Current Dockerfile lacks mkfs tool packages: `test-orch/docker/Dockerfile`.
  * Host orchestrator `docker run` args lack `--privileged`/cap adds: `test-orch/host-orchestrator/dockerutil/run_args.go`.
  * GH Actions workflow does not run the containerized suite; only cargo and golden diff: `.github/workflows/ci.yml`.
* Blockers:
  * GH-hosted runners typically disallow privileged containers. Mitigate via self-hosted runners or make FS matrix non-blocking in GH CI.
