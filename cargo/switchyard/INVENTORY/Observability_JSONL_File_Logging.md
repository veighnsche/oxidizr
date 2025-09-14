# JSONL file logging sink — API

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Optional file-backed sink appends JSON events; no rotation/retention built-in.
- Compile-time feature `file-logging` gates inclusion.
- Does not alter event content (append-only writer).

Policy Impact:

- None; sink selection is integration concern.

---

## 2) Wiring (code-referential only)

- `cargo/switchyard/src/logging/facts.rs:23-50` (FileJsonlSink impl)
- Emit usage consistent with StageLogger events.

---

## 3) Risk & Impact (scored)

- Impact: 1 — Dev/ops convenience.
- Likelihood: 2 — Simple append operations.
- Risk: 2 → Tier: Bronze

---

## 4) Maturity / Robustness

- [ ] Tests for append behavior; rotation guidance

**Current Tier:** Bronze
**Target Tier:** Silver

---

## 5) Failure Modes

- Disk full or permissions → I/O errors; not mapped to exit taxonomy here.

---

## 6) Evidence

- Code cited above; integrate via builder by passing sink as FactsEmitter.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Add tests and docs for rotation/retention | tests + docs | L: 2→1 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Ensure feature is enabled when compiling examples.
