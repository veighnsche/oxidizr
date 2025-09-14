# RELEASE_BLOCKERS_4.md — Facts/Schema v2 Compliance, Rendered as a Dialogue

Last updated: 2025-09-14

This document treats the codebase as a set of arguing voices. Instead of proving, it interprets: where do facts tell one story and tests another? When do we prize “schema as law” versus “operator pragmatism”? The goal is to decide the release‑worthiness of Blocker 4 by staging a debate among AIs, bugs, and code.

---

## Prelude — What schema v2 is “demanding” and why

Schema v2 is the social contract of observability. It demands two things at once:

- A stable envelope on every fact (schema_version=2, ts, plan_id, run_id, event_id, decision, stage, redaction metadata).
- Stage‑specific required fields (e.g., preflight rows must carry path/current_kind/planned_kind; apply.attempt must carry lock fields; apply.result should carry hashes and degraded flags when applicable).

The deeper invariant: operators must be able to reconstruct what happened from facts alone, without reading our code. Therefore, omission of required fields is not a mere lint; it violates intelligibility.

---

## Voices in Conflict

- AI #1 (the Purist): “Schema is a brittle glass vase — drop one required field and the vase shatters. The gap is global; prune.result is missing; some preflight fields are absent.”
- AI #2 (the Pragmatist): “Schema is a checklist — the envelope is centrally enforced. Preflight rows include path/current_kind/planned_kind via RowEmitter. We should validate in CI and move on.”
- BUGS.md (the Cassandra): “In sprint acceptance, schema validation fails for preflight: missing path, current_kind, planned_kind.” It names files and tests and points to SPEC/audit_event.v2.schema.json.

These are not mutually exclusive. The Purist warns of systemic obligations; the Pragmatist points to concrete implementations; Cassandra reports empirical failure. Together they ask: is the failure a local slip, or evidence of a deeper contradiction?

---

## What the Code Says (Voices from the Text)

Code voice (envelope; `cargo/switchyard/src/logging/audit.rs:256–276, 323–327`):

```rust
fn redact_and_emit(
    ctx: &AuditCtx<'_>,
    subsystem: &str,
    event: &str,
    decision: &str,
    mut fields: Value,
) {
    if let Some(obj) = fields.as_object_mut() {
        obj.entry("schema_version").or_insert(json!(SCHEMA_VERSION));
        obj.entry("ts").or_insert(json!(ctx.ts));
        obj.entry("plan_id").or_insert(json!(ctx.plan_id));
        obj.entry("run_id").or_insert(json!(ctx.run_id));
        obj.entry("event_id").or_insert(json!(new_event_id()));
        obj.entry("switchyard_version")
            .or_insert(json!(env!("CARGO_PKG_VERSION")));
        obj.entry("redacted").or_insert(json!(ctx.mode.redact));
        obj.entry("redaction").or_insert(json!({"applied": ctx.mode.redact}));
        // …
        let cur = ctx.seq.get();
        obj.entry("seq").or_insert(json!(cur));
        ctx.seq.set(cur.saturating_add(1));
        obj.entry("dry_run").or_insert(json!(ctx.mode.dry_run));
    }
    // …
}
```

Interpretation: The envelope is not a rumor; it is enforced centrally, every time. The schema’s spine is intact.

Code voice (preflight rows; `cargo/switchyard/src/api/preflight/row_emitter.rs:63–71`):

```rust
let slog = StageLogger::new(ctx);
let mut evt = slog
    .preflight()
    .action_id(aid.to_string())
    .path(args.path)
    .field("current_kind", json!(args.current_kind))
    .field("planned_kind", json!(args.planned_kind));
```

Interpretation: The accused fields are explicitly spoken. If a validator says they’re missing, either a different code path speaks another dialect, or the test is listening to the wrong event variant.

Code voice (apply attempt lock telemetry; `cargo/switchyard/src/api/apply/mod.rs:83–89`):

```rust
slog.apply_attempt()
    .merge(&json!({
        "lock_backend": linfo.lock_backend,
        "lock_wait_ms": linfo.lock_wait_ms,
        "lock_attempts": approx_attempts,
    }))
    .emit_success();
```

Interpretation: Blocker 2’s moral — “warn when no lock manager” — has a schema echo. Even in the pragmatic path, the attempt event includes the lock fields demanded by v2.

Code voice (apply result, success path; `cargo/switchyard/src/api/apply/executors/ensure_symlink.rs:142–154, 168–174`):

```rust
let mut extra = json!({
    "action_id": aid.to_string(),
    "path": target.as_path().display().to_string(),
    "degraded": if degraded_used { Some(true) } else { None },
    "degraded_reason": if degraded_used { Some("exdev_fallback") } else { None },
    "duration_ms": fsync_ms,
    "fsync_ms": fsync_ms,
    "lock_wait_ms": 0u64,
    "before_kind": before_kind,
    "after_kind": if dry { "symlink".to_string() } else { kind_of(&target.as_path()).to_string() },
    "backup_durable": api.policy.durability.backup_durability,
});
ensure_provenance(&mut extra);
insert_hashes(&mut extra, before_hash.as_ref(), after_hash.as_ref());
StageLogger::new(tctx).apply_result().merge(&extra).emit_success();
```

Interpretation: Blocker 1’s lesson (EXDEV degraded) is recorded in facts (`degraded`, `degraded_reason`) and bounded by the schema’s vocabulary. The before/after hashes make the operation legible to auditors.

Code voice (apply summary exposes fsync bound; `cargo/switchyard/src/api/apply/summary.rs:21–33`):

```rust
obj.insert(
    "perf".to_string(),
    json!({ "hash_ms": total.hash, "backup_ms": total.backup, "swap_ms": total.swap })
);
obj.insert("fsync_ms".to_string(), json!(total.swap));
```

Interpretation: The summary speaks a top‑level `fsync_ms` so tests can assert bounds, even if the deeper truth is “swap_ms ≈ rename+fsync.”

---

## The Cross‑Examination (Socratic Method)

- What assumption underlies Cassandra’s claim? That the preflight variant being validated is the same one emitted by `RowEmitter`.
- Is the opposite true anywhere? That some preflight code path “forgets” `path` or kinds? Possibly if a legacy emitter bypasses `RowEmitter` or if tests validate `preflight.summary` events against per‑action requirements.
- What principle is at stake? Not just “fill the fields,” but “single, typed emission path per stage variant.” Duplication breeds dialects; dialects break schemas.

Evidence from BUGS.md (`cargo/switchyard/BUGS.md:91–105`):

> JSON schema validation fails for preflight events — missing required properties (path, current_kind, planned_kind)… Error messages show missing "path", "current_kind", and "planned_kind" properties.

This does not contradict the code quotes; it suggests either:

- The validator looked at a different event (e.g., `preflight.summary`).
- Some tests construct preflight facts without going through `RowEmitter`.
- A timing/redaction variant stripped fields (unlikely, given `.field(...)` inserts plain JSON and redaction preserves structure).

---

## Resolution — Synthesis, not Verdict

Both AIs are right, but at different layers.

- The Purist is right about obligation: schema is law. We must validate every emitted fact against v2 in CI and ensure there’s a single, typed path per stage that meets the law.
- The Pragmatist is right about implementation: the code already centralizes the envelope and, for the primary paths, includes the required fields.
- Cassandra’s failure is thus diagnostic: it reveals a dialect problem — a second voice emitting preflight facts that the schema does not recognize, or a test listening to summaries with the wrong expectations.

Therefore, Blocker 4 stands — but reframed: not “we don’t speak schema” but “too many tongues are speaking; unify the dialects and validate.”

---

## Blocker Statement (Reframed)

Blocker 4 — Facts/schema v2 compliance — is a unification task. Ensure that every stage emits through a single typed builder (the `StageLogger` façade and per‑stage helpers) and that the CI harness validates each emitted JSON against `/SPEC/audit_event.v2.schema.json` for the correct variant (per‑action vs summary). Address known missing emissions (e.g., `prune.result`) to close the contract.

- Status: 🔶 In Progress
- Justification: Primary paths for preflight rows and apply attempt/result appear compliant by code inspection, but an acceptance test flags a missing‑fields failure. Additionally, `prune.result` is not yet emitted by the prune mechanism, which is an explicit schema v2 gap.

---

## Next Action — Two Ways of Believing

- If we believe the schema is law, we must:
  - Add a test helper that validates every emitted fact against `SPEC/audit_event.v2.schema.json` stage branches (per‑action vs summary) across plan, preflight, apply, rollback, and prune.
  - Ensure all preflight emissions flow through `RowEmitter`; remove or adapt any legacy/alternate emitters.
  - Plumb and emit `prune.result` via `StageLogger::prune_result()` and add goldens.
  - Gate CI on zero schema violations and byte‑identical goldens after redaction.

- If we believe operators need pragmatism, we should:
  - Keep `additionalProperties: true` posture in the schema and focus on Requireds only; document that some fields (e.g., `degraded`, attestation) are best‑effort.
  - Preserve dual emission in locking (WARN attempt then SUCCESS attempt) but document consumer guidance to filter on `decision`.
  - Prioritize readability of facts (explicit `fsync_ms`, clear `degraded_reason`) even if they are derivable from nested objects.

---

## Appendix — Evidence Index

- Envelope enforcement: `cargo/switchyard/src/logging/audit.rs:256–276, 323–327`
- Preflight per‑action required fields: `cargo/switchyard/src/api/preflight/row_emitter.rs:63–71`
- Apply attempt lock fields: `cargo/switchyard/src/api/apply/mod.rs:83–89`
- Apply result degraded+hash fields: `cargo/switchyard/src/api/apply/executors/ensure_symlink.rs:142–154, 168–174`
- Apply summary fsync bound: `cargo/switchyard/src/api/apply/summary.rs:21–33`
- Schema reference: `cargo/switchyard/SPEC/audit_event.v2.schema.json` (§ stage requirements); SPEC overview: `SPEC/SPEC.md §5`
- Cassandra’s claim: `cargo/switchyard/BUGS.md:91–106`

— End of dialogue. The law is affirmed; the dialect must be unified.
