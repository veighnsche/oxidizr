# ✅ Immediate Start Tasks — Production Requirements

---

## 1) **R2 — Restore exact topology**

**Requirements for production safety:**

* Every rollback step must restore the **same kind of node** as before (file ↔ symlink ↔ none).
* Sidecar metadata (`backup_meta.v1`) is **always written before mutation** and survives crashes.
* Restore logic:

  * `prior_kind=file` → restore file contents + mode.
  * `prior_kind=symlink` → restore symlink pointing to original `prior_dest`.
  * `prior_kind=none` → ensure path absent.
* TOCTOU safety: all restores performed via parent dir FD and atomic syscalls.
* Sidecar **must remain after restore** to allow later idempotent checks (R3).
* Tests: file→symlink→restore, symlink→symlink→restore, none→symlink→restore, sidecar integrity.
* Audit facts: include `before_kind` + `after_kind`.

---

## 2) **R3 — Idempotent rollback**

**Requirements for production safety:**

* Rollback is **safe to run multiple times** without changing a correct state.
* Short-circuit rules:

  * If target already matches `prior_kind`, return `Ok` without touching disk.
* If backup payload missing:

  * Return `Ok` if current state already correct.
  * Otherwise return `E_ROLLBACK` (or NotFound) so facts show partial failure.
* Telemetry: rollback fact includes `reason="idempotent_noop"` when short-circuited.
* Tests: run rollback twice for each `prior_kind`; verify second run is noop + fact reason.

---

## 3) **L4 — Require LockManager in production**

**Requirements for production safety:**

* `Policy.require_lock_manager = true` by default in production.
* `apply.rs`:

  * If Commit mode and no lock present → immediate fail:

    * `error_id=E_LOCKING`, exit code = 30.
    * No mutations performed.
* In dry-run or dev mode: allow override (`require_lock_manager=false`) but emit `no_lock_manager=true` fact as WARN.
* Tests:

  * Commit with no lock → fails `E_LOCKING`.
  * Dry-run with no lock → passes but logs WARN.

---

## 4) **H3 — Health verification part of commit**

**Requirements for production safety:**

* `Policy.require_smoke_in_commit = true` by default.
* Commit flow enforces:

  * Smoke runner must exist and pass.
  * If runner missing/fails → emit `E_SMOKE`, exit code = 80, auto-rollback unless disabled.
* Smoke checks at minimum:

  * Target executable exists.
  * Execute bit set.
  * Running `--version` or `--help` succeeds within timeout.
* Telemetry: `apply.result` includes `smoke_status`.
* Tests:

  * Commit with missing runner → fails + rollback.
  * Commit with failing smoke runner → fails + rollback.
  * Policy override (`require_smoke_in_commit=false`) → commit skips smoke, passes.

---

## 5) **RC1 — Rescue profile availability**

**Requirements for production safety:**

* Before Commit with `require_rescue=true`, verify:

  * Either BusyBox present **and executable**, OR ≥6/10 GNU tools present **and executable**.
* `rescue.rs` check must include **X\_OK probe** (access or tiny `--help` run) with timeout.
* `apply.rs`: if rescue check fails, STOP with `E_POLICY` before mutation.
* Telemetry: `plan/preflight` facts include `rescue_profile` (busybox|gnu-subset|none).
* Tests:

  * PATH cleared → Commit with `require_rescue=true` fails.
  * PATH with BusyBox → passes.
  * PATH with partial GNU tools (<6/10) → fails.

---

## 6) **CI2 / CI3 — Zero-SKIP + Golden diff gate**

**Requirements for production safety:**

* **Golden diff gate:**

  * CI runs tests with `GOLDEN_OUT_DIR`.
  * Canonical JSONL facts must match committed fixtures **byte-for-byte**.
  * Regeneration allowed only via explicit `GOLDEN_UPDATE=1` workflow.
* **Zero-SKIP gate:**

  * CI fails if any test reports SKIP/XFAIL.
  * For YAML suites, grep logs for SKIP markers and fail job.
* Makefile target: `make switchyard-golden` to regenerate fixtures.
* Tests: Add golden update tests; assert failures when diffs exist.
* Docs: README explains golden workflow and zero-SKIP enforcement.

---

# 📋 Cross-cutting requirements

* **Traceability:** All new code/tests linked from `SPEC/traceability.md` back to REQ IDs.
* **Facts schema:** Extend only with additive fields (never break schema v1).
* **Error IDs:** Use stable IDs from `src/api/errors.rs`. No new IDs unless unavoidable.
* **Docs:** SPEC and CHECKLIST updated when gap closed; include sidecar glossary and rescue profile semantics.

---

# 🚦 Summary of “safe for production” bar

* **Rollback**: exact topology, idempotent, tested, facts enriched.
* **Locking**: mandatory in production, enforced fail-closed.
* **Health**: smoke test required in commit, rollback on fail.
* **Rescue**: verifiable toolset on PATH before commit.
* **CI**: golden fixtures locked, no silent skips.
* **Traceability**: REQ→Test→Spec linked, schema stable, error codes fixed.

With these six complete, you have a **minimum viable production-safe Switchyard**. F3 (filesystem matrix) is the only big piece left that’s infra-bound.
