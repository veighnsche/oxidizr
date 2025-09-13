# Implementation Inventory Template

- Title: <feature name>
- Category: Safety | UX | DX | Infra | Concurrency | Determinism | Observability | Recovery | Attestation | Performance | API Safety
- Maturity: Bronze | Silver | Gold | Platinum
- Owner(s): <github handle(s)>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN, links>

## Summary

Brief description of the feature and its intent. Reference SPEC requirements if applicable.

## Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| <clear benefit> | <repo-relative code/test citation> |
|  |  |

| Cons | Notes |
| --- | --- |
| <tradeoff/limitation> | <operational caveat/portability note> |
|  |  |

## Implementation

- Key modules and symbols:
  - Files and functions/classes used (with paths), for example:
    - `src/foo/bar.rs::some_function()`
- Data flow and control flow:
  - How requests enter, which modules process them, and where side effects occur.
- Invariants enforced and error mapping.

## Wiring Assessment

- Entry points calling into this feature
- Adapters, traits, or policies used
- Stage(s) that emit facts or perform mutations
- Conclusion: wired correctly? yes/no and why

## Evidence and Proof

- Tests covering this feature (paths and test names)
- Observability: facts/emitted fields and schema version
- Determinism: IDs, timestamps, redaction policy

## Feature Analytics

- Complexity: <qualitative + rough LOC/modules touched>
- Risk & Blast Radius: <subsystems/filesystems/policies impacted>
- Performance Budget: <hot path? expected overhead; micro/macro benchmarks if present>
- Observability: <facts emitted, schema version, log fields>
- Test Coverage: <unit/integration/property/golden; file paths/test names>
- Determinism & Redaction: <IDs, timestamps, redaction strategy>
- Policy Knobs: <config flags and defaults>
- Exit Codes & Error Mapping: <table below must reflect this>
- Concurrency/Locking Touchpoints: <who acquires locks; deadlock surfaces>
- Cross-FS/Degraded Behavior: <EXDEV paths, fallbacks>
- Platform Notes: <Linux variants, container/fs quirks>
- DX Ergonomics: <APIs, ergonomics tradeoffs>

## Gaps and Risks

- Known limitations
- Security/consistency risks

## Next Steps to Raise Maturity

- Concrete, small PR-sized tasks
- Golden tests or CI gates to add

## Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| <policy_flag> | <default> | <enforcement behavior> |

## Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| <E_FOO> | <NN> | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |

## Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| <stage.event> | <fields> | <schema ref> |

## Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| <repo path> | <test name> | <what it proves> |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | <baseline> | <minimal guarantees> | <tests> | <ops/tooling> | Additive or Replacement |
| Silver | <hardened & observable> | <determinism, error taxonomy, facts/logs validated> | <tests> | <ops/tooling> | Additive or Replacement |
| Gold | <production-ready> | <policy gating, rollback tests, perf bounds> | <tests> | <ops/tooling> | Additive or Replacement |
| Platinum | <mission-critical> | <formal invariants, multi-platform, continuous compliance> | <tests> | <ops/tooling> | Additive or Replacement |

Upgrade Path Notes:
- Bronze→Silver: <Additive/Replacement> — <why>
- Silver→Gold: <Additive/Replacement> — <why>
- Gold→Platinum: <Additive/Replacement> — <why>

## Maintenance Checklist

- [ ] Code citations are accurate (paths and symbol names)
- [ ] Policy knobs documented reflect current `policy::Policy`
- [ ] Error mapping and `exit_code` coverage verified
- [ ] Emitted facts fields listed and schema version up to date
- [ ] Determinism parity (DryRun vs Commit) verified in tests
- [ ] Goldens added/updated and CI gates green
- [ ] Preflight YAML or JSON Schema validated (where applicable)
- [ ] Cross-filesystem or degraded-mode notes reviewed (if applicable)
- [ ] Security considerations reviewed; redaction masks adequate
- [ ] Licensing impact considered (deps changed? update licensing inventory)
- [ ] Maturity rating reassessed and justified if changed
- [ ] Observations log updated with date/author if noteworthy

## Observations log

- <YYYY-MM-DD> — <author> — <short observation>

## Change history

- <YYYY-MM-DD> — <author> — <summary of change>; PR: <#NNNN>

## Related

- SPEC sections, ADRs, and planning docs
- Closely related inventory entries

