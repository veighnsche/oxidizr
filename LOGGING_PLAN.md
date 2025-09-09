# Logging Plan (aligned with VERBOSITY & anti-masking)

## Goals

* **One source of truth per layer:** Product = mutation facts; Runner = orchestration & assertions; Host = container lifecycle.&#x20;
* **Portable & machine-readable:** JSONL everywhere with a tiny shared envelope.&#x20;
* **Correlatable:** shared `run_id`, `container_id`, `suite`, RFC3339 timestamps.&#x20;
* **Non-masking:** capture **raw** product stdout/stderr verbatim; never downgrade severities.&#x20;

---

## Verbosity (strict)

Use the project’s four levels **intrinsically**—the message decides its level; flags only filter visibility.
**Flag → visibility:** default: v0+v1; `-v`: +v2; `-vv`: +v3; `-q`: v0 only.&#x20;

* **v0 (Critical/Summary):** fatal errors; run and suite end summaries.
* **v1 (Default):** major lifecycle steps and important outcomes.
* **v2 (Verbose):** command echoing, expanded details.
* **v3 (Trace):** step-by-step traces, raw context for debugging.&#x20;

**Prefix for human logs:** `[<distro>][v<level>][<scope>] …` where scope is `HOST` or `RUNNER` (blank for product/raw).&#x20;

---

## Canonical JSONL envelope (all components)

Each line is one event:

```json
{
  "ts": "2025-09-09T10:30:45.949Z",
  "component": "product|runner|host",
  "level": "trace|debug|info|warn|error",

  "run_id": "r-20250909-1030-abc123",
  "container_id": "ctr_96be2899",
  "distro": "arch|manjaro|...",

  "suite": "75-flip-checksums",
  "stage": "preflight|deps|build|run_suites|restore|collect",

  "event": "enabled|removed_and_restored|cmd_exec|assert_fail|...",

  "cmd": "string",
  "rc": 0,
  "duration_ms": 123,

  "target": "/usr/bin/sha256sum",
  "source": "/usr/bin/uu-sha256sum",
  "backup_path": "/.oxidizr/backups/sha256sum.XXXX",

  "artifacts": [".../logs/..."],
  "message": "short human text"
}
```

This envelope matches the one you already defined; we’re keeping it as the contract.&#x20;

---

## Responsibilities (who logs what)

### Product (Rust) — **mutation facts only**

* **Events & levels**

  * `enabled`, `removed_and_restored` → **v1 info** (+ **v0 summary** at run/suite end).
  * `link_started`, `restore_started` → **v2 debug**.
  * `link_done`, `restore_done`, `backup_created` → **v2 debug** (include `duration_ms`).
  * `skip_applet` (presence-aware) → **v1 warn** with `target`, reason.
  * `package_install`, `package_remove` → **v1 info**; failures → **v0 error**.
* **Fields:** always include `target`/`source`/`backup_path` when mutating; include `applet_total`, `linked_count`, `skipped_count`, `elapsed_ms`, `distro`.&#x20;
* **Must not:** shape/normalize runner/host logs; no fallbacks that hide failures.

### Container Runner (Python+Bash) — **orchestration & assertions**

* **Events & levels**

  * Stage boundaries: `stage_start`/`stage_end` → **v1 info** with `stage`, timings.
  * Suite boundaries: `suite_start`/`suite_end` → **v1 info**; `suite_end` also **v0 summary**.
  * Command wrapper: `cmd_exec` → **v2 debug** (show `cmd`, `rc`, `duration_ms`).
  * Assertions: `assert_pass` → **v1 info**, `assert_fail` → **v0 error** with evidence paths.
  * Policy violations (e.g., missing xfail reason, attempt to normalize repos): **v0 error** and abort.
* **Raw capture:** always write **verbatim** `product.stdout.log` & `product.stderr.log` per suite.&#x20;
* **Must not:** force `RUST_LOG`, force locales by default, normalize repos/mirrors, or touch `/usr/bin/*` except via product calls (all previously agreed).
* **Invariants enforced in logs:**

  * `restore` failure → emit `assert_fail` and **suite FAIL** (no warnings).
  * Presence-aware assertions: compute expected applets from installed package, never hardcode counts.&#x20;

### Host Orchestrator (Go) — **container lifecycle**

* **Events & levels**

  * `container_start`, `container_ready` → **v1 info**.
  * `container_exit` → **v1 info** (+ **v0** if non-zero exit).
  * `artifact_mirror` → **v1 info** with destinations.
  * High-verbosity live tail: `stderr_tail` (last N lines) → **v2/v3**; never alter severity/content.&#x20;
* **Must not:** mutate container FS; rewrite severities; hang on channel closure (ensure lifecycle events always conclude).&#x20;

---

## File layout & mirroring (no change)

**In container (runner writes):**

```
/workspace/.proof/
  logs/
    product.stdout.log
    product.stderr.log
    runner.jsonl
  results/
    summary.json
  snapshots/
    <suite>/<phase>/*.meta.json
```

**On host (orchestrator mirrors 1:1):**

```
.artifacts/<run_id>/<distro>_<container_id>/
  product.stdout.log
  product.stderr.log
  runner.jsonl
  host.jsonl
  summary.json
```

(As in your current plan.)&#x20;

---

## Event → Level matrix (quick reference)

| Component | Event                                        | Level            | Why                                   |
| --------- | -------------------------------------------- | ---------------- | ------------------------------------- |
| product   | enabled / removed\_and\_restored             | v1               | major lifecycle outcome               |
| product   | skip\_applet                                 | v1/warn          | presence-aware skip should be visible |
| product   | link\_started / restore\_started             | v2               | detail                                |
| product   | link\_done / restore\_done / backup\_created | v2               | detail + timings                      |
| runner    | stage\_start / stage\_end                    | v1               | user-visible progress                 |
| runner    | suite\_start / suite\_end                    | v1 (+v0 summary) | suite lifecycle + final status        |
| runner    | cmd\_exec                                    | v2               | debug echo w/ rc & duration           |
| runner    | assert\_pass                                 | v1               | visible outcome                       |
| runner    | assert\_fail                                 | v0               | must not be filterable                |
| host      | container\_start / ready                     | v1               | lifecycle                             |
| host      | container\_exit (rc≠0)                       | v0               | critical container failure            |
| host      | stderr\_tail (high verb.)                    | v2/v3            | triage aid, not default noise         |

Verbosity meanings and filtering follow `VERBOSITY.md`.&#x20;

---

## Summary JSON (per container)

Shape remains as you defined, with the policy affirmation preserved:
`"harness_policy": "No harness mutation of product-owned artifacts; fail-on-skip enforced"`.&#x20;

---

## IDs, clocks, and durations

* **IDs:** `run_id` generated by host and injected; `container_id` from host; runner echoes both in every event.&#x20;
* **Time:** `ts` is RFC3339; `duration_ms` computed on a monotonic clock. Host times are authoritative for cross-container aggregation.&#x20;

---

## Redaction & safety

* No secrets in logs. If any command may emit secrets, runner’s subprocess wrapper can redact known tokens; default list is empty.
* Prefer checksums/metadata over dumping sensitive file contents. (Unchanged from your plan.)&#x20;

---

## Guardrails (CI checks)

* Missing required envelope fields (`ts`, `component`, `run_id`) in **runner.jsonl** or **host.jsonl** → **run FAIL**.&#x20;
* Suites that invoked the product but lack `product.stdout/stderr.log` → **suite INCONCLUSIVE** (fail the run).&#x20;
* Any `restore` failure → **suite FAIL** (not warning).
* Presence-aware assertions only; if runner hardcodes applet counts, fail lint.&#x20;

---

## “Definition of done” (per component)

**Product**

* Emits domain events at the right levels; respects verbosity mapping; documents event names & meanings in CLI help/README.&#x20;

**Runner**

* Writes `runner.jsonl`, `product.stdout.log`, `product.stderr.log`, `summary.json`, with envelope fields present on every line.
* No env/locale/repo normalization unless a suite explicitly asks (and logs it).

**Host**

* Streams stdout & stderr; writes `host.jsonl`; mirrors artifacts; never blocks on channel close; always writes run summary with container indexes.&#x20;