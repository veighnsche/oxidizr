# Public API (rustdoc)

Build with:

```bash
cargo doc -p switchyard --no-deps
```

Start here:
- `switchyard::api` — builder, `Switchyard`, `plan`, `preflight`, `apply`
- `switchyard::types` — `Plan`, `Action`, `ApplyMode`, IDs, reports
- `switchyard::fs` — atomic swap, backup/restore engine
- `switchyard::policy` — configuration presets
- `switchyard::adapters` — integration traits and defaults
- `switchyard::logging` — facts, StageLogger, redaction
