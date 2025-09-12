# Round 1 — Peer Review (Active)

Generated: 2025-09-12 14:34:15+02:00
Coordinator: Cascade
Scope: Each AI peer-reviews the next AI’s analysis documents (left rotation). Verify claims, provide proofs, and patch missing content in those analysis docs. Record findings in your own AI_ANALYSIS_#.md under “Round 1 Peer Review”.

## Rotation Mapping (Left by 1)

- AI 1 reviews AI 2’s doc set:
  - FS_SAFETY_AUDIT.md, API_SURFACE_AUDIT.md, OBSERVABILITY_FACTS_SCHEMA.md, ERROR_TAXONOMY.md, INDEX.md
- AI 2 reviews AI 3’s doc set:
  - PRESERVATION_FIDELITY.md, PREFLIGHT_MODULE_CONCERNS.md, POLICY_PRESETS_RATIONALE.md, LOCKING_STRATEGY.md, idiomatic_todo.md, SECURITY_REVIEW.md, RELEASE_AND_CHANGELOG_POLICY.md
- AI 3 reviews AI 4’s doc set:
  - BACKWARDS_COMPAT_SHIMS.md, BEHAVIORS.md, EXPERIMENT_CONSTANTS_REVIEW.md, REEXPORTS_AND_FACADES.md, RETENTION_STRATEGY.md, PERFORMANCE_PLAN.md, TEST_COVERAGE_MAP.md, MIGRATION_GUIDE.md, ROADMAP.md, CODING_STANDARDS.md, CONTRIBUTING_ENHANCEMENTS.md
- AI 4 reviews AI 1’s doc set:
  - EDGE_CASES_AND_BEHAVIOR.md, CORE_FEATURES_FOR_EDGE_CASES.md, CLI_INTEGRATION_GUIDE.md

## Tasks (for all AIs)

- Evidence every claim
  - For each assigned document, verify all claims against code/tests/specs.
  - Add citations (paths and symbol names) and include command outputs where relevant.
- Patch gaps and inaccuracies
  - Edit the assigned analysis documents directly to fix missing or incorrect statements.
  - Keep changes minimal and well-justified; note edits in your AI report.
  - Append a summary section at the end of each delegated analysis doc:
    - Heading: `## Round 1 Peer Review (AI <N>, <YYYY-MM-DD HH:MM TZ>)`
    - Content: brief list of claims verified, key citations (code/spec/tests), and a short summary of edits.
- Record findings in your AI report
  - Add a “Round 1 Peer Review” section to your AI_ANALYSIS_#.md.
  - For each doc: list verified claims, new/updated citations, and edits made.

## Editing Rights (Round 1)

- You MUST write the verified content and fixes directly into the assigned analysis `.md` files you are reviewing. This ensures Round 2 can review your updated source of truth.
- You MAY also edit your own `AI_ANALYSIS_#.md` to record evidence and changes.
- You MUST NOT edit any analysis files outside your Round 1 target list, and MUST NOT edit other AIs’ `AI_ANALYSIS_#.md` files.

For traceability, add a short footer line at the end of each analysis you update:
`Reviewed and updated in Round 1 by AI <N> on <YYYY-MM-DD HH:MM TZ>`

## Deliverables

- Updated analysis docs you reviewed (only your Round 1 targets).
- Updated AI_ANALYSIS_#.md with a “Round 1 Peer Review” section including:
  - Checklist (per doc)
  - Claim → Proof mapping
  - Changes Made
  - Open Questions

## Editing Boundaries

- You may only edit:
  1) The Round 1 target analysis docs (listed above for your AI), and
  2) Your own AI_ANALYSIS_#.md.

## Suggested Commands

- Search symbols: `rg --smart-case "fn\s+atomic_symlink_swap|fsync_parent_dir" cargo/switchyard/src`
- Build/tests: `cargo test -p switchyard` (with env toggles as needed)
- Facts schema: validate emissions against `SPEC/audit_event.schema.json` (tooling of your choice)

## Definition of Done

- All claims in your Round 1 target docs are substantiated with citations or corrected.
- Your AI report includes evidence and a concise change log per doc.
