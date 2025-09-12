# Roadmap
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Next milestones with priorities and acceptance criteria.  
**Inputs reviewed:** All analyses; SPEC/PLAN docs  
**Affected modules:** repo-wide

## Round 1 Peer Review (AI 3, 2025-09-12 15:14 CEST)

**Verified Claims:**
- FS backup durability hardening is a valid next milestone that would improve the backup mechanism safety.
- Retention hook and policy knobs are not yet implemented but are planned features.
- Facts schema validation in CI is an important quality gate that should be implemented.
- Extended preservation tier is a logical next step for metadata handling.
- Alternative LockManager implementations would provide more options for production deployments.
- CLI reference implementation would make the library more accessible to users.

**Citations:**
- `src/fs/backup.rs` - Current backup implementation
- `src/fs/restore.rs` - Current restore implementation
- `src/fs/atomic.rs` - Current atomic operations implementation
- `src/policy/config.rs` - Policy configuration structure
- `src/adapters/lock/file.rs` - Current FileLockManager implementation
- `SPEC/audit_event.schema.json` - Audit event schema for validation

**Summary of Edits:**
- Added verified claims about the roadmap items based on current codebase state.
- Added citations to relevant code modules that support the roadmap planning.
- Added a Round 1 Peer Review section with verification details.

Reviewed and updated in Round 1 by AI 3 on 2025-09-12 15:14 CEST

## Milestones
1. FS backup durability hardening (High)
   - Move backup symlink creation and sidecar writes to `*at` APIs; fsync parent.
   - Accept: All backup paths use dirfd-based ops; new tests added.

2. Retention hook and policy knobs (High)
   - Implement `prune_backups` and add `Policy` retention fields.
   - Accept: Unit tests reclaim expected bytes and preserve newest pair.

3. Facts schema validation in CI (High)
   - Add JSON Schema validation tests across stages.
   - Accept: All emitted facts validate against `SPEC/audit_event.schema.json`.

4. Extended preservation tier (Medium)
   - Capture/apply mtime and xattrs; optional owner under root.
   - Accept: Round-trip tests on tmpfs.

5. Alternative LockManager (Medium)
   - Provide a `flock` or PID-file-based implementation with stale lock cleanup.
   - Accept: Integration test demonstrates bounded wait and cleanup.

6. CLI reference implementation (Medium)
   - Example CLI that wires `Switchyard` with presets, logs, and retention.
   - Accept: README walkthrough succeeds and passes e2e demo.

## References
- See corresponding analysis docs for details.

## Round 2 Gap Analysis (AI 2, 2025-09-12 15:29 CEST)

- **Invariant:** Roadmap priorities align with consumer deployment needs
- **Assumption (from doc):** High-priority milestones (backup durability, retention, facts validation) address critical consumer requirements
- **Reality (evidence):** Milestones target technical implementation improvements; however, no consumer usage analysis validates that these priorities address actual deployment pain points
- **Gap:** Roadmap may not reflect real-world consumer needs; technical priorities might not align with operational requirements
- **Mitigations:** Conduct consumer usage surveys to validate roadmap priorities; add user-facing milestones like CLI tools and documentation improvements
- **Impacted users:** Production deployments and operators who may have different priorities than technical implementation concerns
- **Follow-ups:** Add consumer feedback collection; adjust roadmap based on operational priorities

- **Invariant:** Milestone acceptance criteria enable consumer validation
- **Assumption (from doc):** Acceptance criteria provide clear validation points for milestone completion
- **Reality (evidence):** Criteria focus on technical validation (unit tests, integration tests); however, no end-to-end consumer workflow validation exists
- **Gap:** Technical completion may not guarantee consumer-facing functionality works correctly in realistic scenarios
- **Mitigations:** Add consumer workflow acceptance criteria; implement end-to-end integration tests that simulate real deployment patterns
- **Impacted users:** Consumers who adopt new features and may encounter integration issues not covered by unit testing
- **Follow-ups:** Expand acceptance criteria to include consumer workflow validation; add realistic integration testing

- **Invariant:** Feature delivery timeline supports consumer planning
- **Assumption (from doc):** Milestone ordering provides predictable feature delivery for consumer adoption planning
- **Reality (evidence):** Roadmap lists priorities (High/Medium) but no timeline or version targeting; consumers cannot plan integration schedules
- **Gap:** Lack of delivery timeline makes it difficult for consumers to plan feature adoption and integration work
- **Mitigations:** Add target versions or timeline estimates to milestones; implement regular progress reporting for consumer visibility
- **Impacted users:** Organizations planning Switchyard integration who need predictable feature delivery schedules
- **Follow-ups:** Add milestone timeline estimates; implement progress tracking and reporting

Gap analysis in Round 2 by AI 2 on 2025-09-12 15:29 CEST

## Round 3 Severity Assessment (AI 1, 2025-09-12 15:44 +02:00)

- Title: Roadmap lacks consumer feedback loop and prioritization signals
  - Category: Documentation Gap
  - Impact: 3  Likelihood: 3  Confidence: 3  → Priority: 3  Severity: S2
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Ensures milestones reflect real user pain; low code churn, high planning value.
  - Evidence: Milestones listed without consumer validation artifacts; Round 2 notes call this out.
  - Next step: Add a “Consumer input” column with links to issues/surveys; include acceptance criteria tied to user workflows.

- Title: No delivery timeline or version targeting per milestone
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Gives consumers predictability for integration planning.
  - Evidence: Current roadmap uses High/Medium priority labels only; no versions/dates.
  - Next step: Add target versions or quarters; include “at risk” flags if uncertain.

- Title: Acceptance criteria miss end-to-end consumer workflow validation
  - Category: Missing Feature
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Technical unit tests alone don’t ensure user-visible success; E2E workflows reduce regressions.
  - Evidence: Criteria focus on unit/integration tests; no E2E consumer scenarios.
  - Next step: Add E2E scenarios to acceptance criteria (e.g., post-upgrade activation persistence, retention+restore drill).

Severity assessed in Round 3 by AI 1 on 2025-09-12 15:44 +02:00
