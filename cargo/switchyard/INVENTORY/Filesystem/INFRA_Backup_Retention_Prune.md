# Backup retention and prune

- Category: Infra
- Maturity: Bronze

## Summary

Prunes older backups and sidecars by count and age, preserving the newest. Emits facts via API wrapper.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Prevents unbounded growth of backups | `cargo/switchyard/src/fs/backup.rs::prune_backups()` implements selection and deletion |
| Preserves newest artifacts | Selection logic retains most recent; see code comments |
| Facts for observability | `cargo/switchyard/src/api.rs::Switchyard::prune_backups()` emits `prune.result` |

| Cons | Notes |
| --- | --- |
| Potential off-by-one in count logic if untested | Marked in Gaps; add unit tests |
| Deleting artifacts is irreversible | Dry-run mode recommended for preview; ensure policy/tag correctness |

## Behaviors

- Scans for backup artifacts matching the tag/pattern for a target.
- Selects deletion set by applying count and age limits while preserving the newest.
- Deletes chosen backups and sidecars; fsyncs parent directory best-effort.
- Emits `prune.result` facts including `backup_tag`, retention knobs, and counts.

## Implementation

- Engine: `cargo/switchyard/src/fs/backup.rs::prune_backups()` selects deletion set; fsyncs parent.
- API: `cargo/switchyard/src/api.rs::Switchyard::prune_backups()` emits `prune.result` facts with policy parameters.

## Wiring Assessment

- Public API exposed; facts emitted with `backup_tag` and retention knobs.
- Conclusion: wired; needs more tests.

## Evidence and Proof

- No explicit unit tests detected for prune; indirect validation via usage recommended.

## Feature Analytics

- Complexity: Low-Medium. Directory scan + selection + deletion.
- Risk & Blast Radius: Medium; incorrect configuration can delete desired artifacts; recommend dry-run preview path (Gap).
- Performance Budget: I/O bound on directories; manageable for typical counts.
- Observability: `prune.result` facts include tag and retention knobs.
- Test Coverage: Gap — add unit tests for selection and deletion; add golden for `prune.result` facts.
- Determinism & Redaction: Facts redacted in DryRun; selection deterministic given inputs.
- Policy Knobs: `retention_count_limit`, `retention_age_limit`, `backup_tag`.
- Exit Codes & Error Mapping: N/A (prune not part of apply-stage exit taxonomy).
- Concurrency/Locking: Independent; advisable to avoid concurrent apply during prune.
- Cross-FS/Degraded: N/A.
- Platform Notes: Works with standard filesystems; long filenames and many entries increase cost.
- DX Ergonomics: Simple API via `Switchyard::prune_backups()`.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `retention_count_limit` | `None` | Keep at most N newest backups when set |
| `retention_age_limit` | `None` | Delete backups older than duration when set |
| `backup_tag` | `DEFAULT_BACKUP_TAG` | Scope prune to a logical backup set |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| N/A | — | Not mapped; prune is a utility API |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `prune.result` | `backup_tag`, `retention_count_limit`, `retention_age_limit`, `deleted_count` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/fs/backup.rs` | prune selection tests (planned) | correct selection by count/age |
| `src/api.rs` | prune.result facts test (planned) | fields and values emitted |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze (current) | Manual prune via API; emits facts | Deletes selected artifacts; retains newest | Basic tests (planned) | None | Additive |
| Silver | Dry-run preview; better safety rails | Preview set matches actual deletion | Unit + integration + goldens | CLI/flag support | Additive |
| Gold | CI policy checks; retention SLOs documented | Compliance with retention policies | Goldens + CI | CI checks | Additive |
| Platinum | Automated retention with auditing | Continuous compliance & reporting | System tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Policy knobs documented reflect current `policy::Policy`
- [x] Error mapping and `exit_code` coverage verified (N/A)
- [x] Emitted facts fields listed and schema version up to date
- [ ] Determinism parity (DryRun vs Commit) verified in tests
- [ ] Goldens added/updated and CI gates green
- [ ] Security considerations reviewed; safe deletion preview available
- [ ] Licensing impact considered (deps changed? update licensing inventory)

## Gaps and Risks

- Potential off-by-one in count semantics and age parsing; lack of tests.

## Next Steps to Raise Maturity

- Add unit tests for pruning selection and file deletion; add golden for `prune.result`.

## Related

- Policy `retention_count_limit`, `retention_age_limit`.
