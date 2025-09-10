# Audit and Logging Guide

This document explains how oxidizr-arch emits and manages logs for both
humans (operators and developers) and auditors. It covers:

- What gets logged and where the logs go
- The JSONL audit log schema and how to interpret entries
- CLI flags and environment variables that affect logging behavior
- How developers should instrument code with audit events

All referenced paths and APIs are implemented in the following modules:

- `src/logging/init.rs`
- `src/logging/audit.rs`
- `src/logging/mod.rs`
- Call sites in `src/cli/handler.rs`, `src/experiments/*.rs`, `src/system/worker/*.rs`

## Overview

oxidizr-arch emits two kinds of logs via the `tracing` ecosystem:

- Human-readable logs to stderr
- Structured audit events as one-line JSON (JSONL) to a file sink

The logging system is initialized at program start in `src/main.rs`:

- CLI is parsed first; if `--dry-run` is set, the environment variable
  `OXIDIZR_DRY_RUN=1` is exported to disable the audit file sink.
- `logging::init_logging()` installs two layers:
  - Human layer (stderr), configured by `VERBOSE` (0..3)
  - Audit JSONL layer (file), filtered to events with `target == "audit"`

## Log Locations

- Audit JSONL (preferred): `/var/log/oxidizr-arch-audit.log`
  - If that path is not writable (e.g., not root), logging falls back to
    `$HOME/.oxidizr-arch-audit.log`.
  - This announcement is printed once in human logs, e.g.,
    `[arch][v1] audit sink: /var/log/oxidizr-arch-audit.log` or the fallback path.
- Human logs: stderr of the running process.

## Human Logs (stderr)

Configured by the environment variable `VERBOSE`:

- `VERBOSE=0` → ERROR
- `VERBOSE=1` → INFO (default)
- `VERBOSE=2` → DEBUG
- `VERBOSE=3` → TRACE

Format is a single line per event:

```
[<distro>][v<level>] <message>
```

Where `<distro>` is read from `/etc/os-release` `ID=...`, and `<level>` is
mapped as described above.

## Audit JSONL

Audit logs are emitted via `audit_event_fields(...)` or the legacy-compatible
`audit_event(...)` helper in `src/logging/audit.rs`. They are routed to the
JSONL sink only when `target == "audit"` (this is set internally by the
helpers).

- File sink path: `AUDIT_LOG_PATH` = `/var/log/oxidizr-arch-audit.log`
- Sink is disabled when `OXIDIZR_DRY_RUN=1` (automatic under `--dry-run`)
- Each entry is a single line of JSON with flattened fields (no span info)

### Envelope Fields

The helpers standardize a canonical envelope so auditors can query easily:

- `ts` — RFC3339 timestamp with millisecond precision (UTC offset)
- `component` — logical component; currently always `"product"`
- `subsystem` — subsystem emitting the event (e.g., `cli`, `experiments`, `worker`, `operation`)
- `level` — standardized string: `"info"` or `"error"` based on decision
- `run_id` — optional correlation ID from environment variable `RUN_ID`
- `container_id` — best-effort container identifier from `/etc/hostname`
- `distro` — OS ID from `/etc/os-release` `ID=...`
- `event` — event name (e.g., `enabled`, `disabled`, `aur_helper_name`)
- `decision` — outcome classification (e.g., `success`, `failure`, `found`, `not_found`)

### Structured Fields (`AuditFields`)

Optional fields carried by `AuditFields` and promoted to top-level keys:

- `stage` — high-level stage (string)
- `suite` — logical suite or experiment name (e.g., `checksums`)
- `cmd` — command text (auto-masked; see Secret Masking)
- `rc` — integer return code (exit status)
- `duration_ms` — operation duration in milliseconds
- `target` — resource target (e.g., a binary name or path)
- `source` — resource source path
- `backup_path` — where a backup was written
- `artifacts` — list of artifact names; rendered as a comma-separated string

### Legacy Convenience

- `audit_event(component, event, decision, inputs, outputs, exit_code)` — accepts
  two free-form strings; values are auto-masked for secrets. New code should
  prefer `audit_event_fields` with structured fields.
- `audit_op(operation, target, success)` — small wrapper for simple operations,
  internally calls `audit_event_fields("operation", ...)`.

## Secret Masking

To reduce risk of leaks when using free-form fields, the helpers apply
best-effort masking for common credential patterns, e.g., `token=...`,
`password=...`, `Authorization=Bearer ...`.

This is not a silver bullet; developers should avoid placing secrets in
logs. Use structured fields and include only safe metadata.

## Examples

Example audit entry when enabling an experiment via CLI:

```json
{
  "ts": "2025-09-10T12:34:56.789Z",
  "component": "product",
  "subsystem": "cli",
  "level": "info",
  "run_id": "4c0c2f",
  "container_id": "a1b2c3d4e5f6",
  "distro": "arch",
  "event": "enabled",
  "decision": "success",
  "target": "coreutils"
}
```

Example AUR helper discovery in `worker`:

```json
{
  "ts": "2025-09-10T12:35:10.123Z",
  "component": "product",
  "subsystem": "worker",
  "level": "info",
  "run_id": "4c0c2f",
  "container_id": "a1b2c3d4e5f6",
  "distro": "arch",
  "event": "aur_helper_name",
  "decision": "found",
  "target": "paru"
}
```

Example no checksum applets discovered (from `experiments/checksums.rs`):

```json
{
  "ts": "2025-09-10T12:36:00.000Z",
  "component": "product",
  "subsystem": "experiments",
  "level": "info",
  "run_id": "",
  "container_id": "a1b2c3d4e5f6",
  "distro": "arch",
  "event": "nothing_to_link",
  "decision": "checksums",
  "suite": "checksums"
}
```

Note: the `level` string is derived from `decision`; only values like
`failure`/`error` produce `"error"`.

## Operator and Auditor Usage

- View human logs as the command runs (stderr). Increase detail with `VERBOSE=2` or `3`.
- Tail the audit file for structured events:

```bash
sudo tail -f /var/log/oxidizr-arch-audit.log
```

- Query audit trails with `jq`:

```bash
jq -r 'select(.subsystem=="cli" and .event=="enabled") | [.ts,.target,.decision] | @tsv' /var/log/oxidizr-arch-audit.log
```

- Correlate across a run by setting `RUN_ID` before invoking oxidizr-arch:

```bash
RUN_ID=$(date +%s) oxidizr-arch --all enable
```

- Dry runs (`--dry-run`) still emit human logs but do not write to the audit file sink.

## Developer Guidance

- Prefer `audit_event_fields` with `AuditFields` for new instrumentation.
  Re-exported in `src/logging/mod.rs`:
  
  ```rust
  use crate::logging::{audit_event_fields, AuditFields};

  let _ = audit_event_fields(
      "cli",
      "enabled",
      "success",
      &AuditFields { target: Some("coreutils".to_string()), ..Default::default() },
  );
  ```

- For simple ops, use `audit_op(op_name, target, success)`.
- Do not include secrets in free-form fields. If you must log commands, rely on the masking and keep context minimal.
- Use clear `event` names and consistent `decision` values across subsystems.
- Use `stage`, `suite`, `rc`, and `duration_ms` to make entries actionable.
- The audit layer is attached only to events with `target == "audit"`. Do not set targets manually; the helpers do it for you.

## Behavior Under Dry-Run

- `--dry-run` sets `OXIDIZR_DRY_RUN=1` before initializing logging.
- Audit file sink is disabled; human logs still print a notice like
  `audit sink: disabled (dry-run)`.
- Regular operation proceeds; no JSONL is written.

## Compatibility and Backward Compatibility

- Legacy `audit_event(...)` remains available for compatibility but should be
  phased out in favor of `audit_event_fields(...)`.
- The `audit_event_fields(...)` helper promotes optional structured fields to
  top-level JSON keys for easy querying and future schema evolution.

## Troubleshooting

- If `/var/log/oxidizr-arch-audit.log` isn’t created, ensure:
  - You’re not in `--dry-run` mode
  - The process has permissions to write to `/var/log` (try with `sudo`), or
  - Check fallback at `$HOME/.oxidizr-arch-audit.log`
- Increase verbosity via `VERBOSE=2` or `3` to see sink announcements and context.

## Security Considerations

- Audit logs are append-only at the sink; rotate and protect them via system
  tooling (e.g., `logrotate`, permissions `0600`, group root-only access).
- Even with masking, treat logs as sensitive; they may include file paths and
  system metadata.

## Related Files and Symbols

- `src/logging/audit.rs` — `AUDIT_LOG_PATH`, `AuditFields`, `audit_event_fields`, `audit_event`, `audit_op`
- `src/logging/init.rs` — `init_logging()` installs stderr and JSONL sinks
- `src/cli/handler.rs` — examples of audit events for `enabled`, `disabled`, `removed_and_restored`
- `src/system/worker/aur.rs` — audit for `aur_helper_name` discovery
- `src/experiments/checksums.rs` — emits `nothing_to_link` with `suite="checksums"`
