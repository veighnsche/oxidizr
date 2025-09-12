# Removals Registry

Use this file to track removals/moves/replacements for files that cannot be annotated inline (e.g., JSON schemas), or when you prefer a central checklist. Each line should follow one of the formats below.

Formats
- remove: <repo-relative-path> — <reason or successor>
- move: <old-path> -> <new-path> — <reason>
- replace: <old-path> -> <new-path> — <reason>

Examples
- remove: cargo/switchyard/SPEC/audit_event.schema.json — field superseded; see SPEC.md
- move: cargo/switchyard/src/api.rs -> cargo/switchyard/src/api/mod.rs — idiomatic module layout
- replace: cargo/switchyard/src/logging/audit.rs -> cargo/switchyard/src/logging/audit.rs — StageLogger facade replaces emit_* helpers

Checklist generation
```bash
rg -n "^remove:|^move:|^replace:" cargo/switchyard/zrefactor/removals_registry.md -S
```

Planned entries
- move: cargo/switchyard/src/api.rs -> cargo/switchyard/src/api/mod.rs — idiomatic module layout; drop #[path] includes
- replace: cargo/switchyard/src/fs/backup.rs -> cargo/switchyard/src/fs/backup/{mod.rs,snapshot.rs,sidecar.rs,index.rs} — split monolith per zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md
- replace: cargo/switchyard/src/fs/restore.rs -> cargo/switchyard/src/fs/restore/{mod.rs,types.rs,selector.rs,idempotence.rs,integrity.rs,steps.rs,engine.rs} — split monolith per zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md
- replace: cargo/switchyard/src/logging/audit.rs -> cargo/switchyard/src/logging/audit.rs — StageLogger facade replaces emit_* helpers per zrefactor/logging_audit_refactor.INSTRUCTIONS.md
 - remove: cargo/switchyard/zrefactor/documantation/code_smell.md — BREAKING: superseded by zrefactor/CODE_SMELL_AND_CLEAN_CODE_AUDIT.md
