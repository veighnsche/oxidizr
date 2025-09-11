🚩 High-risk gaps before production

Rollback

REQ-R2 (restore exact topology) and REQ-R3 (idempotent rollback) — without these, rollback may leave subtle corruption or fail in chained apply/rollback cycles.

Locking

REQ-L4 (LockManager required in production) — currently optional; production needs hard enforcement + doc.

Rescue

REQ-RC1 (rescue profile) — fallback must be verifiably available; otherwise failure modes are brittle.

Health verification

REQ-H3 (health verification in commit) — commit isn’t really safe until smoke/health passes.

Filesystem coverage

REQ-F3 (supported FS verified) — otherwise you can’t claim portability guarantees.

CI

REQ-CI2/3 — zero-SKIP + golden diff gate are needed to enforce determinism in practice.