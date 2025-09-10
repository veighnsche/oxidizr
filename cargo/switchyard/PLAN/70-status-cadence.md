# Status Cadence

## Weekly Template

- Summary
- Done
- Planned
- Risks/Blocks
- Demos/Evidence
- ADRs changed/added

## Review Checklist

- SPEC trace updated?
- CI gates green?
- New risks captured?

---

## Schedule & Roles (planning phase)

- Monday
  - Standup (15m): status since last week, immediate blockers
  - Owner updates: `PLAN/00-charter.md`, `PLAN/10-architecture-outline.md`

- Wednesday
  - Working session (30–45m): resolve open questions; draft ADRs
  - Owner updates: `PLAN/20-spec-traceability.md`, `PLAN/50-risk-register.md`

- Friday
  - Review (30m): sign-offs per `PLAN/40-quality-gates.md`; capture decisions
  - CI evidence: run `SPEC/tools/traceability.py`; docs/links lint

## Artifact Update Expectations

- `PLAN/00-charter.md`: adjust scope/NFRs if risks change; confirm constraints/MSRV
- `PLAN/10-architecture-outline.md`: update planned API and open questions decisions
- `PLAN/20-spec-traceability.md`: keep mapping current; add acceptance criteria details
- `PLAN/30-delivery-plan.md`: adjust milestone dates and entry/exit as needed
- `PLAN/40-quality-gates.md`: refine gates and automation details
- `PLAN/50-risk-register.md`: add triggers; assign owners
- `PLAN/adr/*`: add/accept/supersede decisions as information changes

## Ownership

- Maintainers: Charter, Architecture, API planning
- QA/Requirements: Traceability mapping, acceptance criteria
- SRE/CI: Quality gates, automated checks
- PM/Tech lead: Delivery plan, risks, status reporting
