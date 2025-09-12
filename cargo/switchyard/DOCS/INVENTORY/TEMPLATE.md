# Implementation Inventory Template

- Title: <feature name>
- Category: Safety | UX | DX | Infra
- Maturity: Bronze | Silver | Gold | Platinum
- Owner(s): <github handle(s)>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN, links>

## Summary

Brief description of the feature and its intent. Reference SPEC requirements if applicable.

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

## Gaps and Risks

- Known limitations
- Security/consistency risks

## Next Steps to Raise Maturity

- Concrete, small PR-sized tasks
- Golden tests or CI gates to add

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

