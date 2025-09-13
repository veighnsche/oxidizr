# Preservation capabilities probe

- Category: Safety
- Maturity: Silver
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Probe filesystem support for preservation dimensions (owner, mode, timestamps, xattrs, ACLs, capabilities) and record both the desired preservation and what is supported.

## Behaviors

- Probes platform/filesystem to detect if preservation dimensions are supported.
- Populates `preservation` (desired) and `preservation_supported` (detected) in preflight rows.
- Feeds policy gating when strict preservation is required.
- Exposed via YAML exporter for operator visibility.

## Implementation

- Probe: `cargo/switchyard/src/fs/meta.rs::detect_preservation_capabilities()` — best-effort detection of supported preservation features.
- Preflight rows: exporter preserves `preservation` and `preservation_supported` fields
  - `cargo/switchyard/src/preflight/yaml.rs::to_yaml()` includes these keys when present.
- Policy influence: `policy::Policy` may require certain preservation guarantees (e.g., timestamps, ownership), used in preflight gating.

## Wiring Assessment

- Preflight populates capability information into rows; YAML exporter includes fields; apply/restore attempt to preserve when supported.
- Conclusion: wired correctly for advisory capability reporting.

## Evidence and Proof

- Presence of fields in YAML output; apply/restore code paths preserve owner/mode and attempt timestamps/xattrs where available.

## Gaps and Risks

- Probes are best-effort and environment-sensitive; lack of schema validation for these fields.

## Next Steps to Raise Maturity

- Add explicit tests for capability detection on supported platforms; add schema validation for preflight rows.

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- `cargo/switchyard/src/fs/meta.rs`
- `cargo/switchyard/src/preflight/yaml.rs`
