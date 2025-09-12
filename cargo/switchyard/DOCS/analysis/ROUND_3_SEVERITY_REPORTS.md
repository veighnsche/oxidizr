# Round 3 — Severity Reports

Generated: 2025-09-12 14:34:15+02:00
Coordinator: Cascade
Scope: Each AI performs a third rotation and produces severity assessments for each issue/claim in the reviewed documents. The report must include impact, likelihood, evidence, and prioritization. You MUST also append a "Round 3 Severity Assessment" section inside each of your delegated analysis docs so Round 4 can plan against it. Record outputs in your own AI_ANALYSIS_#.md as well.

## Rotation Mapping (Left by 3 total)

- AI 1 assesses severity for AI 4’s Round 1 doc set.
- AI 2 assesses severity for AI 1’s Round 1 doc set.
- AI 3 assesses severity for AI 2’s Round 1 doc set.
- AI 4 assesses severity for AI 3’s Round 1 doc set.

## Severity Rubric

- Impact (1–5): harm if inaccurate/missing
- Likelihood (1–5): chance the claim is wrong or drifts
- Confidence (1–5): strength of the evidence
- Priority = ceil((Impact × Likelihood) / Confidence)

## Tasks (for all AIs)

- Enumerate findings/issues from prior rounds (verified claims that need strengthening, gaps, risky assumptions).
- Score each finding per rubric.
- Provide short rationale with citations.
- Recommend next actions: fix now, defer, or monitor.

### Write inside the analysis docs
- Append to the end of each delegated analysis doc:
  - Heading: `## Round 3 Severity Assessment (AI <N>, <YYYY-MM-DD HH:MM TZ>)`
  - Content: list of findings with Impact, Likelihood, Confidence, Priority, and brief rationale with citations.

## Editing Rights (Round 3)
- You MAY edit only your Round 3 delegated analysis `.md` files and your own `AI_ANALYSIS_#.md`.
- You MUST NOT edit any other analysis docs or other AIs’ reports.
- Add a short footer line for traceability when you touch a document:
`Severity assessed in Round 3 by AI <N> on <YYYY-MM-DD HH:MM TZ>`

## Deliverables

- Appended `Round 3 Severity Assessment` section in each delegated analysis doc (end of file).
- Updated AI_ANALYSIS_#.md with a “Round 3 Severity Reports” section containing a table/list of findings with scores and rationale.

## Example Entry

- Topic: EXDEV degraded fallback telemetry missing in backup path
  - Impact: 3, Likelihood: 2, Confidence: 4 → Priority: 2
  - Rationale: backup path lacks explicit `degraded` field; see `src/fs/backup.rs`. Not a blocker but helpful for observability.

## Definition of Done

- Each finding includes Impact, Likelihood, Confidence, Priority, and evidence citations.
