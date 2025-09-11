# Switchyard Scrum Planning

This folder contains the epics and sprint plans for Switchyard. Plans are aligned to SPEC (via immutable SPEC_UPDATE_#### docs), PLAN implementation notes, ADRs, and the code in `src/`.

Artifacts:

- `epics.md` — Epics with goals, scope, and completion criteria.
- `sprints/` — Sprint-by-sprint plans with user stories, tasks, acceptance criteria, and deliverables.
- Traceability — Each story references SPEC requirements (REQ-*), TODO checklist IDs, and code locations.

## Sprints

- Sprint 01 — Determinism, Observability, Policy Gating, Locking, and Acceptance Foundations: `sprints/sprint-0001.md`
- Sprint 02 — Preflight Diff, Error Mapping, Smoke Runner, Acceptance, Traceability: `sprints/sprint-0002.md`

Cadence & Capacity:

- 2-week sprints.
- Team capacity: 5 AI engineers, capable of parallelizing significant work.
- Scope sizing: enough work to keep all 5 at high utilization for 2 weeks. The hard limit is the "drift threshold" (see below).

Drift Threshold Policy:

- We stop adding scope when there is a risk of significant drift between SPEC, PLAN, and implementation.
- Every normative change within a sprint MUST produce:
  - An immutable `SPEC_UPDATE_####.md` entry.
  - An ADR if a decision is made or reversed.
  - Updated PLAN impl notes.
- Doc-sync checks must remain green at all times; failing doc-sync is a sprint stopper.

Sprint Readiness Checklist (at start):

- Sprint theme ties to one or more Epics.
- Stories decomposed into tasks with clear acceptance criteria.
- For each normative change, a placeholder SPEC_UPDATE number is reserved.
- Owners assigned for code, docs, and tests.

Sprint Done Checklist (at end):

- All stories meet acceptance criteria; unit tests pass (`cargo test -p switchyard`).
- Doc Sync Checklist complete (SPEC_UPDATEs, ADRs, PLAN notes, cross-links).
- No drift detected between SPEC, PLAN, and implementation.
- If scope warrants, tag an RC build; acceptance tests green or pending with explicit risk callouts.
