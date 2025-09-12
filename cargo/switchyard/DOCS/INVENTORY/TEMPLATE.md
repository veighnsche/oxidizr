# Implementation Inventory Template

- Title: <feature name>
- Category: Safety | UX | DX | Infra
- Maturity: Bronze | Silver | Gold | Platinum

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

## Related

- SPEC sections, ADRs, and planning docs
- Closely related inventory entries
