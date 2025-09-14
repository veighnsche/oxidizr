# <Name> — <Type: Feature | Module | API | Concept>

**Owner:** <@handle/team>  
**Last Reviewed:** <YYYY-MM-DD> · **Next Review:** <YYYY-MM-DD>

> **Scope:** Deep Wiki already documents behavior. This file records **risk posture**, **policy context**, **wiring**, and **proof**.

---

## 1) Contract (non-negotiables)  

*(Max 3 bullets — Alpine brevity)*  

- <Invariant 1>  
- <Invariant 2>  
- <Invariant 3>  

**Policy Impact (Debian-style):**  

- Related SPEC sections: §… (link)  
- Policy knobs: `policy.*` flags that affect this  

---

## 2) Wiring (code-referential only — Arch transparency)  

- **Locations (impl):** `<path>:<line>`  
- **Callers (consumers):** `<path>:<line>`  
- **Callees (deps):** `<path>:<line>`  
- **External Surface:** CLI flags / API endpoints / crate exports  

---

## 3) Risk & Impact (scored)  

- **Impact (1–5):** <#> — why this severe  
- **Likelihood (1–5):** <#> — churn / unsafe / FS variance  
- **Risk = I×L:** <#> → **Tier:** Bronze | Silver | Gold | **Platinum**  

---

## 4) Maturity / Robustness (OpenBSD strictness + Fedora roadmap)  

- [ ] Invariants documented (→ code lines)  
- [ ] Unit tests (→ test names/paths)  
- [ ] Boundary / negative tests  
- [ ] Property / fuzz / chaos / failure-injection (if relevant)  
- [ ] E2E tests (happy/sad/idempotence/rollback)  
- [ ] Platform/FS matrix (if FS/OS-sensitive)  
- [ ] Telemetry (metrics/logs/events + alerts/SLOs)  
- [ ] Rollback / Recovery proof (tests or playbook)  
- [ ] Determinism check (goldens, UUIDv5, redactions — NixOS influence)  
- [ ] Independent review recorded  

**Current Tier:** <Bronze|Silver|Gold|Platinum>  
**Target Tier & ETA (Fedora-style):** <Tier> by <release/date>  

---

## 5) Failure Modes (observable)  

- <Mode> → <symptom / error code / audit event>  
- <Mode> → <…>  

---

## 6) Evidence (links only — Arch style)  

- **Tests:** `<path>::<test_name>` (3–7 load-bearing)  
- **Schemas/Specs:** `<path>`  
- **Telemetry:** metric/log/event names, dashboards  
- **Playbooks:** rollback/recovery procedures  

---

## 7) Gaps → Actions (mandatory — OpenBSD strictness)  

| ID  | Action | Evidence target | Expected effect (I/L ↓) | Owner  | Due        |  
|-----|--------|-----------------|--------------------------|--------|-----------|  
| G-1 | <…>    | <tests/dashboards> | L: 4→3 | @handle | YYYY-MM-DD |  

*(If no gaps, explicitly write “None (verified)”)*  

---

## 8) Notes for IDE-AI (verification rules)  

- Resolve **all** references to **file:line** at HEAD.  
- Derive callers/callees via static analysis.  
- Confirm invariants with guard conditions or post-effects in code.  
- Verify tests assert the invariant.  
- Locate telemetry emit sites + dashboards.  
- Detect policy knobs (`policy.*`) that affect behavior (Gentoo configurability).  
- Compute 90-day **churn score** (git) to inform Likelihood.  
- If unverifiable → mark as **Gap**.  

---
