# Golden Fixtures Strategy (Bronze → Silver → Gold → Platinum)

This document defines how we produce, store, and validate “golden” facts for `switchyard`, and provides a tiered implementation path (bronze/silver/gold/platinum).

It complements `docs/testing/TESTING_POLICY.md` and SPEC v1.1 requirements for determinism, redaction, schema validation, and CI gates.

## Goals

- Ensure emitted facts (plan, preflight, apply, rollback) are stable and reproducible after policy redaction and canonicalization.
- Catch regressions early (schema, ordering, presence/absence of fields, policy gates) without masking real differences.
- Keep the update process explicit, auditable, and reviewable.

## Terminology

- "Facts": structured JSON objects emitted by `src/api/audit.rs` (e.g., `plan`, `preflight`, `apply.attempt`, `apply.result`).
- "Canon": post-redaction, canonicalized view used for comparisons (e.g., timestamp zeroing, volatile-field removal).
- "Golden": committed, versioned files that represent expected canon output for a scenario.

## Data Model

- Source events: JSON objects (one per line) written to a sink; unit tests may capture them in-memory.
- Redaction/canonicalization: `src/logging/redact.rs::redact_event()` + scenario-specific canonical filters (e.g., keep only `stage`, `action_id`, `decision` for per‑action apply.result determinism).
- Storage format: JSON Lines (JSONL) for raw facts; optional canon JSONL or deterministic JSON arrays for golden.

## Storage Layout

```
repo/
  cargo/switchyard/
    tests/
      golden/
        minimal-plan/
          raw_apply.jsonl             # optional raw capture (not used for strict diff)
          canon_apply_result.json     # deterministic array (or .jsonl if preferred)
        preflight-ownership/
          canon_preflight.json
  docs/testing/GOLDEN_FIXTURES.md     # this doc
```

Notes:

- Prefer deterministic arrays (pretty JSON) for canon files when the collection is small; use `.jsonl` if very large.
- Raw artifacts are optional and never used for strict diffing; they aid debugging only.

## Versioning & Evolution

- Tie golden changes to SPEC updates or intentional behavior changes.
- Each PR that changes goldens must state why (SPEC_UPDATE link or ADR reference) and update tests and PLAN accordingly.
- Keep schema version on every fact (`schema_version`). Schema migration rules apply.

## Update Workflow

1) Run the scenario locally via unit tests or a small runner to regenerate raw facts.
2) Apply redaction and canonicalization to produce canon JSON.
3) Diff against committed goldens.
4) If differences are expected, update goldens and include a SPEC_UPDATE/ADR link in the PR description.
5) CI runs the same process and blocks on unexpected diffs.

## Anti‑Flake Guidance

- Filter out volatile runtime fields (timings, severity, degraded, hashes, attestation internals) for equality gates.
- Compare stable subsets per stage (e.g., per‑action `apply.result`: `stage`, `action_id`, `decision`).
- Sort by deterministic keys (stage, `action_id`).
- Never hide real differences with SKIPs; either fix product or update goldens with a documented reason.

---

## Tiers of Implementation

### Bronze (Minimal)

- Purpose: Establish a quick determinism gate with low effort.
- Scope:
  - Unit tests capture facts in-memory (via `FactsEmitter`).
  - Canonicalize to minimal stable tuples (e.g., `(action_id, decision)` for `apply.result`).
  - Compare against goldens stored as a deterministic JSON array per scenario.
- Tooling:
  - Use existing Rust tests only; no external scripts required.
  - Keep goldens under `cargo/switchyard/tests/golden/<scenario>/canon_*.json`.
- Pros/Cons:
  - + Very fast to implement, easy to maintain.
  - − Narrow coverage (typically the final stage only), limited visibility into intermediate facts.

### Silver (Useful Baseline)

- Purpose: Validate more stages with schema guarantees.
- Scope:
  - Validate facts against `SPEC/audit_event.v2.schema.json` (or Rust-side equivalent) in tests.
  - Store and diff canon for selected stages: `plan`, `preflight`, `apply.attempt`, `apply.result`.
  - Introduce a simple generator: write canon files out (behind a `--update-goldens` or test feature flag).
- Tooling:
  - Extend the Python `test_ci_runner.py` to support a `--golden <scenario>` command that:
    - Runs the scenario (calls `cargo test -p switchyard <scenario>`),
    - Emits canon JSON to a temp directory,
    - Optionally updates committed goldens when explicitly requested.
- Pros/Cons:
  - + Better coverage and schema confidence.
  - − Slightly more complexity and I/O management.

### Gold (CI Gate)

- Purpose: Enforce byte-identical canon facts in CI across stable stages.
- Scope:
  - Add a CI job that runs `test_ci_runner.py --golden all` and diffs canon outputs against committed goldens.
  - Zero SKIPs policy: any scenario not runnable fails explicitly with a reason.
  - Document PR process for golden updates (SPEC/ADR links required).
- Tooling:
  - `test_ci_runner.py` produces a deterministic artifact path and a machine-readable diff (unified or JSON diff).
  - CI uploads diffs on failure as artifacts for review.
- Pros/Cons:
  - + Strong guardrail for regressions; clear developer ergonomics.
  - − Requires discipline to keep goldens updated alongside SPEC/PLAN.

### Platinum (Acceptance & Matrix Ready)

- Purpose: Extend gating to acceptance scenarios and environment matrices while remaining deterministic.
- Scope:
  - Introduce scenario descriptors (YAML/JSON) defining inputs and expectations.
  - Run a curated subset of acceptance scenarios that are deterministic (no Docker required) on GitHub Actions.
  - Optionally add a separate workflow for containerized EXDEV acceptance when infrastructure is ready.
  - Enforce schema validation, canon diffs, and produce traceability reports (SPEC tools).
- Tooling:
  - `test_ci_runner.py` supports:
    - Scenario discovery and selection.
    - Schema validation step (jsonschema or Rust validator).
    - Traceability report generation (tie facts to SPEC requirements).
- Pros/Cons:
  - + High confidence, readiness for broader environments.
  - − Highest complexity; requires careful scenario curation to avoid flakes.

---

## Terminology Disambiguation

- "Golden fixtures" refers to the committed, canonicalized artifacts we diff in CI.
- "Gold tier" refers to the maturity level of this mechanism's process (Bronze/Silver/Gold/Platinum).
- Tiers are a general maturity framework and can apply to multiple mechanisms (e.g., golden fixtures, smoke tests, rollback checks). Do not conflate the noun "golden" with the adjective "Gold tier".

Naming discipline:

- When referring to maturity, write "Gold tier" or "Silver tier".
- When referring to artifacts, write "golden fixtures" or just "goldens".
- Avoid ambiguous phrases like "go to golden"; instead, write "promote golden fixtures to Gold tier".

## Canonicalization & Redaction Rules (Current)

- Apply redaction via `src/logging/redact.rs::redact_event()` (timestamps to `TS_ZERO`, remove timing fields, severity, degraded, content hashes, and mask attestation internals).
- For per‑action `apply.result` determinism gates, canonicalize to:
  - `stage`, `action_id`, `decision`.
- Sort canon events by `(stage, action_id)` before diffing.

## CI Integration (Roadmap)

- Bronze: rely on Rust tests only.
- Silver: add `test_ci_runner.py --golden <scenario>` locally and in CI (non-blocking to start).
- Gold: make golden diff job a required status in `.github/workflows/ci.yml` (blocking).
- Platinum: add a separate workflow for acceptance/matrix once ready.

## How to Update Goldens

1) Run `test_ci_runner.py --golden <scenario> --update` locally.
2) Review the diff; ensure it matches an intentional change.
3) Update SPEC/ADR links in the PR description and update PLAN/TODO as needed.
4) Push and ensure CI passes the new golden gate.

## FAQ

- Q: Why not diff raw JSONL facts?
  - A: Raw contains volatile fields (timings, hashes, attestation internals) and ordering differences. We compare canon, not raw, to avoid flakes.
- Q: Can we keep raw files?
  - A: Yes, for debugging, but they’re not part of strict diffs.
- Q: Where to place large scenario data?
  - A: Use a `fixtures/` directory alongside tests with README and pruning guidance.

## Immediate Recommendation

- Implement Silver now:
  - Add a basic golden generator to `test_ci_runner.py`.
  - Capture canon for `plan`, `preflight`, `apply.result` in one scenario (the existing minimal plan) and diff in CI (non-blocking).
- Upgrade to Gold after one iteration:
  - Make the golden diff gate blocking in CI.

## Tier Discipline (Read This Before Touching CI)

- Rule: Gold does not count if Silver was skipped.
  - Any attempt to enable a blocking golden diff gate in CI (Gold) without first implementing the full Silver set is non‑compliant.
  - Silver prerequisites include, at minimum:
    - Canon capture for selected stages (`plan`, `preflight`, `apply.attempt`, `apply.result`).
    - A simple, documented generator flow (e.g., `test_ci_runner.py --golden <scenario>` and `--update`).
    - Schema validation step for emitted facts (JSON Schema or Rust validator) in tests.

- Why skipping Silver is a bad idea (aka “how stupid that is”):
  - False sense of safety: a blocking CI gate on incomplete signals creates confidence without coverage.
  - Brittle process: developers can’t update or regenerate goldens predictably without a generator/update path.
  - Missing guarantees: without schema validation, regressions in structure pass undetected even if bytes match.
  - Review friction: PRs accrue noisy diffs that reviewers can’t relate back to SPEC changes.

- Enforcement guidance:
  - If a PR introduces Gold before Silver, mark the golden job as non‑blocking (or revert the change) and require Silver completion.
  - Require PR descriptions to link SPEC/ADR for any golden change and to state that Silver prerequisites are satisfied.
  - CI review checklist should include: “Silver complete? (canon for stages, generator present, schema validation active)”.
