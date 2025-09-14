# Backup retention and prune — Module

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Prevent unbounded backup growth by pruning per count and/or age while always retaining the newest.
- Delete sidecars alongside payloads; fsync parent directory best‑effort.
- Deterministic newest→oldest selection by timestamp.

Policy Impact:

- `retention_count_limit`, `retention_age_limit`, `backup.tag`.

---

## 2) Wiring (code-referential only)

- Implementation: `cargo/switchyard/src/fs/backup/prune.rs:25-134`
- API wrapper and facts: `cargo/switchyard/src/api/mod.rs:194-266` (`Switchyard::prune_backups()` emits `prune.result`)

---

## 3) Risk & Impact (scored)

- Impact: 2 — Storage hygiene; wrong deletion could remove useful artifacts.
- Likelihood: 3 — Parsing/selection edge cases; mitigated by newest-retained rule.
- Risk: 6 → Tier: Bronze

---

## 4) Maturity / Robustness

- [x] Deterministic selection logic
- [ ] Unit tests for selection and deletion
- [ ] Golden `prune.result` facts

**Current Tier:** Bronze
**Target Tier:** Silver

---

## 5) Failure Modes

- I/O failure during deletion or directory sync → `E_GENERIC` classification via API wrapper.

---

## 6) Evidence

- Code lines cited; API wrapper emits counts and policy parameters.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Add unit tests (count/age) and golden for `prune.result` | tests + CI | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Confirm newest entry always retained; validate age cutoff and count clamping.
