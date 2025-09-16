# Validate Facts Against Schema v2

Validate emitted JSONL facts to prevent drift and keep goldens stable.

## Option A: Using ajv-cli (Node.js)

1) Install ajv-cli (optional in CI):

```bash
npm install -g ajv-cli
```

2) Validate each JSON line against the schema:

```bash
schema="$(pwd)/SPEC/audit_event.v2.schema.json"
file="/var/log/switchyard/facts.jsonl"
cat "$file" | while IFS= read -r line; do
  printf "%s" "$line" | ajv validate -s "$schema" -d - || exit 1
done
```

## Option B: Python jsonschema

```python
import json, sys
from jsonschema import validate
from pathlib import Path

schema = json.loads(Path('SPEC/audit_event.v2.schema.json').read_text())

ok = True
for line in Path('/var/log/switchyard/facts.jsonl').read_text().splitlines():
    try:
        validate(instance=json.loads(line), schema=schema)
    except Exception as e:
        print('Validation failed:', e)
        ok = False
        break
sys.exit(0 if ok else 1)
```

## Tips

- Use the `StageLogger` facade to ensure consistent envelope fields and redaction.
- Run validation in CI against a curated subset of scenarios for fast feedback.

Citations:
- `SPEC/audit_event.v2.schema.json`
- `src/logging/audit.rs`
- Inventory: `INVENTORY/70_Observability_Facts_Schema_Validation.md`
