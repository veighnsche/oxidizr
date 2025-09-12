# Round 3 — Severity Reports

Generated: 2025-09-12 14:34:15+02:00
Coordinator: Cascade
Scope: Each AI performs a third rotation and conducts a concrete triage of gaps found so far. For every finding, decide whether it becomes a feature/bug/doc change, or is declined. Provide severity scoring, disposition, feasibility, complexity, and a clear rationale (why update vs why not). You MUST append a "Round 3 Severity Assessment" section inside each of your delegated analysis docs so Round 4 can plan against it. Also record a consolidated triage board in your own AI_ANALYSIS_#.md.

## Rotation Mapping (Left by 3 total)

- AI 1 assesses severity for AI 4’s Round 1 doc set.
- AI 2 assesses severity for AI 1’s Round 1 doc set.
- AI 3 assesses severity for AI 2’s Round 1 doc set.
- AI 4 assesses severity for AI 3’s Round 1 doc set.

## Severity Rubric

- Impact (1–5): harm to users/ops if the gap persists
- Likelihood (1–5): probability the gap manifests or regresses
- Confidence (1–5): strength of evidence and reproducibility
- Priority = ceil((Impact × Likelihood) / Confidence)
- Severity class (suggested): S1 Critical (Priority ≥ 4), S2 High (3), S3 Medium (2), S4 Low (1)

## Tasks (for all AIs)

- Enumerate findings/issues from prior rounds (verified claims that need strengthening, gaps, risky assumptions).
- Score each finding per rubric.
- Provide short rationale with citations.
- Recommend next actions: fix now, defer, or monitor.

### Classification and triage per finding
- Category: one of [Bug/Defect, Missing Feature, Documentation Gap, Policy/Default Mismatch, Performance/Scalability, DX/Usability].
- Disposition: one of [Implement (feature/bugfix), Spec-only (clarify docs/spec), Backlog, Won't fix].
- Low-hanging fruit (LHF): Yes/No (quick <1 day fix with low risk?).
- Feasibility: High/Medium/Low.
- Complexity: 1–5 (estimated engineering effort/entanglement).
- Why update?: concrete user/ops value; cost of inaction; examples.
- Why not?: risks, churn, alternatives, or deferral justification.

### Write inside the analysis docs
- Append to the end of each delegated analysis doc:
  - Heading: `## Round 3 Severity Assessment (AI <N>, <YYYY-MM-DD HH:MM TZ>)`
  - For each finding, include the structured block:
    - Title: <short name>
    - Category: <see list>
    - Impact: <1–5>  Likelihood: <1–5>  Confidence: <1–5>  → Priority: <1–4>  Severity: <S1..S4>
    - Disposition: <Implement | Spec-only | Backlog | Won't fix>  LHF: <Yes/No>
    - Feasibility: <H/M/L>  Complexity: <1–5>
    - Why update vs why not: <concise rationale>
    - Evidence: <citations to code/spec/tests/PM behavior>
    - Next step: <what should happen in Round 4 or beyond>

## Editing Rights (Round 3)
- You MAY edit only your Round 3 delegated analysis `.md` files and your own `AI_ANALYSIS_#.md`.
- You MUST NOT edit any other analysis docs or other AIs’ reports.
- Add a short footer line for traceability when you touch a document:
`Severity assessed in Round 3 by AI <N> on <YYYY-MM-DD HH:MM TZ>`

## Deliverables

- Appended `Round 3 Severity Assessment` section in each delegated analysis doc (end of file) with structured entries.
- Updated AI_ANALYSIS_#.md with a “Round 3 Severity Reports” triage board (table or list) including Category, Severity, Priority, Disposition, Feasibility, Complexity, Rationale, Evidence, and Next step per finding.

## Example Entry

- Title: EXDEV degraded fallback telemetry missing in backup path
- Category: Observability (DX/Usability)
- Impact: 3, Likelihood: 2, Confidence: 4 → Priority: 2 → Severity: S3
- Disposition: Implement  LHF: Yes
- Feasibility: High  Complexity: 2
- Why update vs why not: Clarifies degraded mode to users and ops; low risk additive facts; cost of inaction is poor diagnosability.
- Evidence: `cargo/switchyard/src/fs/backup.rs`, `cargo/switchyard/src/api/apply/mod.rs` (no `degraded=true` emission today)
- Next step: Add `degraded=true` to facts on EXDEV fallback; update schema/tests accordingly.

## Definition of Done

- Each finding includes Impact, Likelihood, Confidence, Priority, and evidence citations.
