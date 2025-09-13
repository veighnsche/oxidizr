# Backup retention and prune

- Category: Infra
- Maturity: Bronze

## Summary

Prunes older backups and sidecars by count and age, preserving the newest. Emits facts via API wrapper.

## Implementation

- Engine: `cargo/switchyard/src/fs/backup.rs::prune_backups()` selects deletion set; fsyncs parent.
- API: `cargo/switchyard/src/api.rs::Switchyard::prune_backups()` emits `prune.result` facts with policy parameters.

## Wiring Assessment

- Public API exposed; facts emitted with `backup_tag` and retention knobs.
- Conclusion: wired; needs more tests.

## Evidence and Proof

- No explicit unit tests detected for prune; indirect validation via usage recommended.

## Gaps and Risks

- Potential off-by-one in count semantics and age parsing; lack of tests.

## Next Steps to Raise Maturity

- Add unit tests for pruning selection and file deletion; add golden for `prune.result`.

## Related

- Policy `retention_count_limit`, `retention_age_limit`.
