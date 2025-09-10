# 🔒 High-Risk System Tool Audit Checklist

## 1. Transactionality

* [x] All changes happen in a **staging area** before touching live files.
* [x] Commit is a **single atomic operation** (e.g., `rename(2)` of a symlink or directory).
* [ ] If any step fails, rollback is **automatic** and **complete**.
* [x] Re-running the same operation is **idempotent** (safe to repeat).

## 2. Auditability

* [x] Every operation generates a **structured log** entry (machine-readable, timestamped, op_id).
* [ ] Logs include **before/after cryptographic hashes** of all affected files.
* [ ] Logs record **actor, versions, source of binaries, and exit codes**.
* [x] An **append-only change journal** exists for long-term auditing.

## 3. Least Intrusion

* [ ] Only the intended files are modified — verified by a **preflight diff**.
* [ ] Ownership, permissions, and timestamps are **preserved**.
* [ ] Extended attributes (xattrs, ACLs, capabilities) are **preserved and verified**.

## 4. Determinism

* [ ] Given identical inputs, outputs are **bit-for-bit reproducible**.
* [ ] No reliance on environment variables, locale, or time (unless explicitly controlled).
* [ ] Results are invariant across multiple runs in the same environment.

## 5. Conservatism

* [ ] If behavior differences are detected (e.g., GNU vs uutils flags), the tool **fails closed**.
* [x] Unsafe or ambiguous operations require **explicit override flags**.
* [ ] Dry-run mode is **default** unless `--assume-yes` is given.

## 6. Minimal Trusted Surface

* [ ] Dependencies are reduced to the **minimum necessary**.
* [x] No bundled complexity (databases, interpreters, or services) without necessity.
* [x] The mechanism is auditable by a single engineer in a reasonable time.

## 7. Recovery First

* [ ] A complete **backup of links and metadata** is created before swap.
* [ ] A **rescue toolset** (busybox or static fallback) is available in `$PATH`.
* [x] Rollback can be executed in **one step**, even in degraded mode.
* [ ] Initramfs/emergency shell has guaranteed access to **essential commands**.

## 8. Health Verification

* [ ] A **smoke test suite** runs automatically after swap (`ls`, `cp`, `mv`, `rm`, etc.).
* [ ] Failure in smoke test triggers **rollback**.
* [ ] Optional: real-world scripts/units are **canary-tested** in isolation first.

## 9. Explicit Consent

* [x] Destructive or system-wide changes require a **confirmation flag**.
* [x] Interactive prompts are clear and include **what will change**.
* [x] No hidden defaults: the operator must **consciously approve risk**.

## 10. Supply Chain Integrity

* [ ] Binaries are verified against **signatures and checksums** before use.
* [ ] Provenance of each binary (version, source, hash) is **logged in audit trail**.
* [ ] Optional: generate **SBOM fragments** for each installed version.

---

## ✅ Final Reviewer’s Pass

* [x] Transaction model is correct (staging → validate → atomic commit).
* [ ] Logs are complete, verifiable, and stored safely.
* [ ] All security contexts and metadata preserved.
* [ ] Recovery plan tested and documented.
* [ ] Smoke tests and canary checks pass reliably.
* [ ] No undocumented side-effects.
