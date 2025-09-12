# Round 2 — Gap Analysis (Consumer invariants)

Generated: 2025-09-12 14:34:15+02:00
Coordinator: Cascade
Scope: Each AI performs a second rotation and conducts a Gap Analysis from the perspective of CLI consumers that integrate Switchyard. Assess whether the current analysis documents, as updated in Round 1, cover real-world invariants and expectations (e.g., durability of symlink/unified-binary activation across package upgrades). You MUST append a "Round 2 Gap Analysis" section at the end of each delegated analysis doc, and also record findings in your own AI_ANALYSIS_#.md.

## Rotation Mapping (Left by 2 total)

- AI 1 meta-reviews AI 3’s Round 1 outputs and the same doc set AI 3 reviewed.
- AI 2 meta-reviews AI 4’s Round 1 outputs and the same doc set AI 4 reviewed.
- AI 3 meta-reviews AI 1’s Round 1 outputs and the same doc set AI 1 reviewed.
- AI 4 meta-reviews AI 2’s Round 1 outputs and the same doc set AI 2 reviewed.

## Tasks (for all AIs)

- Identify consumer invariants
  - Extract the implicit/explicit invariants a CLI consumer would rely on (e.g., persistence of activation state, atomicity across PM operations, retention expectations, preservation guarantees).
- Compare assumptions vs reality
  - For each invariant, state the assumption suggested by the analysis doc, the field reality (with citations to code/tests/specs/PM behavior), and the resulting gap.
- Propose mitigations
  - Suggest policy toggles, telemetry/facts, operational guidance, or test additions to address or make the gap explicit.
- Write inside the analysis docs
  - Append a new section at the end of each delegated analysis doc:
    - Heading: `## Round 2 Gap Analysis (AI <N>, <YYYY-MM-DD HH:MM TZ>)`
    - Use the template below per identified invariant:

      - Invariant: <short name>
      - Assumption (from doc): <quote/summary>
      - Reality (evidence): <citations to code/spec/tests/packaging behavior>
      - Gap: <what fails or is missing>
      - Mitigations: <policy/settings/docs/tests>
      - Impacted users: <who>
      - Follow-ups: <what to change next rounds>

- Record findings in your AI report
  - Add a “Round 2 Gap Analysis” section to your AI_ANALYSIS_#.md consolidating gaps and proposed mitigations per document.

## Deliverables

- Appended `Round 2 Gap Analysis` section in each delegated analysis doc (end of file).
- Updated AI_ANALYSIS_#.md with “Round 2 Gap Analysis” section and a per-doc checklist.

## Editing Rights (Round 2)

- You MAY edit only your Round 2 delegated analysis `.md` files and your own `AI_ANALYSIS_#.md`.
- You MUST NOT edit any other analysis docs or other AIs’ reports.
- Add a short footer line for traceability when you touch a document:
`Gap analysis in Round 2 by AI <N> on <YYYY-MM-DD HH:MM TZ>`

## Rubric

- Relevance: consumer invariants clearly articulated
- Evidence: specific citations and reproducible commands (code/spec/tests/PM docs)
- Actionability: concrete mitigations (policy/telemetry/tests/docs)
- Scope: considers packaging/update scenarios, cross-filesystems, and permissions

## Suggested Tools

- Code search: `rg` queries for functions/symbols
- Quick local runs: subset of `cargo test -p switchyard`
- PM behavior references: distribution package manager (PM) documentation and upgrade workflows

## Definition of Done

- Gap analysis recorded inside the analysis docs and in your AI report, with actionable notes and flagged issues for Round 3 severity scoring and Round 4 planning.

## Example invariants to stimulate analysis (non-exhaustive)

- Core utilities activation permanence
  - Assumption: unified binary/symlink activation remains permanently active once switched.
  - Reality: package manager upgrades may replace/unlink targets, invalidating activation.
  - Consider: monitoring and re-application on upgrade, policy to require post-upgrade verification, facts to log activation drift, tests simulating upgrade.

- Cross-filesystem atomicity
  - Assumption: atomic operations across mount boundaries.
  - Reality: EXDEV occurs; degraded fallback may be disallowed by policy.
  - Consider: explicit guidance for cross-FS plans, preflight detection and user-facing messaging.

- Preservation guarantees
  - Assumption: owner/timestamps/xattrs preserved by default.
  - Reality: current tier may be Basic; extended requires capabilities/privileges.
  - Consider: document tiers in user-facing CLI, add tests/features gated by policy.

- Retention and rollback availability
  - Assumption: backups always available to restore.
  - Reality: no retention policy means operator might prune manually; missing sidecar scenarios.
  - Consider: default retention knobs, CLI prune guidance, telemetry on restore readiness.
