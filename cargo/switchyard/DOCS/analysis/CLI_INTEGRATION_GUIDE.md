# CLI Integration Best Practices

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Guidance for downstream CLIs to compose policies, handle exit-codes and facts, choose tags/retention, and interoperate with package managers.  
**Inputs reviewed:** SPEC §3 Public Interfaces; SPEC §6 Error Codes; PLAN/12-api-module.md; CODE: `src/api.rs`, `src/policy/config.rs`, `src/logging/*`  
**Affected modules:** `api`, `policy`, `logging`

## Summary

- Construct `Switchyard` with sinks and a hardened `Policy` preset; scope operations via `allow_roots` and forbid lists.
- Use `SafePath::from_rooted` for all user-provided inputs and pass `&SafePath` directly to mutating APIs. All low-level filesystem mutators in `src/fs/` now accept `&SafePath` to enforce TOCTOU safety per SPEC v1.1; no conversion to raw `&Path` is required.
- Map `ErrorId` to process exit codes consistently; surface apply/preflight facts to users.

## Integration Skeleton

- Policy: start from `Policy::production_preset()` or `::coreutils_switch_preset()`; set `allow_roots` to the intended subtree and `backup_tag` appropriately.
- Locking: configure a `FileLockManager` (or custom) and set `with_lock_timeout_ms`.
- Facts: implement a `FactsEmitter` (e.g., file JSONL). Consider also emitting human summaries.
- Attestation: optional `Attestor` adapter to sign success bundles.

## Exit Codes

- Map `ErrorId` to `SPEC/error_codes.toml` via `api::errors::exit_code_for`. On success exit 0; on `E_LOCKING` exit 30, etc.

## Retention

- Expose a `prune` subcommand in your CLI that calls the library API to prune backups deterministically under policy.
- Use `Switchyard::prune_backups(&SafePath)` to apply count/age limits set in `Policy` (see `retention_count_limit` and `retention_age_limit`).
- Facts will include a `prune.result` event with `pruned_count` and `retained_count` for auditability.

## Package Manager Interop

- Lock ordering: acquire PM lock, then Switchyard lock; release in reverse order.
- Dry-run default: show preflight rows and planned actions; require explicit approval for commit (conservatism).

## References

- CODE: `src/api.rs`, `src/policy/config.rs`, `src/logging/facts.rs`

## Round 1 Peer Review (AI 4, 2025-09-12 15:16 CET)

- **Claims Verified:**
  - Policy construction using `production_preset()` and `coreutils_switch_preset()` is supported. Cited `src/policy/config.rs` for policy presets.
  - Locking configuration with `FileLockManager` is implemented. Cited `src/adapters/lock/file.rs` for `FileLockManager` implementation.
  - Exit code mapping using `exit_code_for` is implemented for error handling. Cited `src/api/errors.rs` for exit code mapping.
- **Key Citations:**
  - `src/policy/config.rs`: Policy presets for production and coreutils switching.
  - `src/adapters/lock/file.rs`: Implementation of `FileLockManager` for process locking.
  - `src/api/errors.rs`: Exit code mapping for error handling.
- **Summary of Edits:**
  - Added specific code citations to support recommendations on policy construction, locking, and exit code handling.
  - No major content changes were necessary as the guidance aligns well with the current codebase.

Reviewed and updated in Round 1 by AI 4 on 2025-09-12 15:16 CET

## Round 2 Gap Analysis (AI 3, 2025-09-12 15:33+02:00)

- Invariant: API guidance is accurate and actionable.
- Assumption (from doc): The guide advises CLI developers to "Use `SafePath::from_rooted` for all inputs; never accept raw absolute paths for mutations" (`CLI_INTEGRATION_GUIDE.md:12`).
- Reality (evidence): The core mutating functions in `src/fs/` (e.g., `swap`, `restore`) do not accept a `SafePath` argument. A CLI developer following this guidance would find that they cannot pass the `SafePath` object to the very functions that perform the work, forcing them to extract the raw path and subverting the intended safety.
- Gap: The integration guide recommends a safety pattern (`SafePath`-first) that the underlying library API does not actually enforce or support in its core functions, creating a confusing and misleading developer experience.
- Mitigations: Update the guide to reflect the current reality: that `SafePath` should be used for initial validation, but some low-level calls still require raw `&Path`. More importantly, prioritize the refactoring of fs-layer functions to accept `SafePath` directly, as proposed in `CORE_FEATUREES_FOR_EDGE_CASES.md`.
- Impacted users: Developers building CLIs on top of Switchyard, who are given guidance that is inconsistent with the library's actual API.
- Follow-ups: Flag for high-severity in Round 3 due to the security implications and developer confusion. Prioritize the `SafePath` refactoring in Round 4.

- Invariant Update: The library now provides `Switchyard::prune_backups(&SafePath)`; this guide reflects the current API. Retention can be implemented using this API and `Policy` knobs.

## SafePath Example

```rust
use switchyard::{Switchyard, logging::JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{PlanInput, LinkRequest};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

let facts = JsonlSink::default();
let audit = JsonlSink::default();
let mut policy = Policy::production_preset();
policy.allow_roots.push(std::path::PathBuf::from("/tmp/root/usr/bin"));

let api = Switchyard::new(facts, audit, policy);
let root = std::path::Path::new("/tmp/root");
let src = SafePath::from_rooted(root, &root.join("opt/new/bin/ls")).unwrap();
let tgt = SafePath::from_rooted(root, &root.join("usr/bin/ls")).unwrap();
let plan = api.plan(PlanInput { link: vec![LinkRequest { source: src, target: tgt }], restore: vec![] });
let _pf = api.preflight(&plan).unwrap();
let _ar = api.apply(&plan, ApplyMode::Commit).unwrap();
```

Gap analysis in Round 2 by AI 3 on 2025-09-12 15:33+02:00

## Round 3 Severity Assessment (AI 2, 2025-09-12 15:45+02:00)

- **Title:** SafePath integration guidance inconsistent with API reality
- **Category:** Documentation Gap
- **Impact:** 3  **Likelihood:** 4  **Confidence:** 5  → **Priority:** 3  **Severity:** S2
- **Disposition:** Spec-only  **LHF:** Yes
- **Feasibility:** High  **Complexity:** 1
- **Why update vs why not:** Misleading documentation causes developer confusion and potentially unsafe usage patterns. Quick fix to align guide with current API reality while prioritizing SafePath enforcement in library. Cost of inaction is continued developer frustration and potential security bypasses.
- **Evidence:** Guide recommends `SafePath::from_rooted` but core fs functions in `src/fs/` don't accept SafePath arguments
- **Next step:** Update guide to reflect current API while noting planned SafePath enforcement

- **Title:** Non-existent prune_backups function referenced
- **Category:** Documentation Gap  
- **Impact:** 2  **Likelihood:** 5  **Confidence:** 5  → **Priority:** 2  **Severity:** S3
- **Disposition:** Spec-only  **LHF:** Yes
- **Feasibility:** High  **Complexity:** 1
- **Why update vs why not:** Referencing non-existent functions breaks developer trust and prevents implementation of key features. Simple documentation fix until feature implemented. Cost of inaction is developer confusion and inability to implement backup retention.
- **Evidence:** No `prune_backups` function exists in codebase; function referenced in guide at line 28
- **Next step:** Remove reference and note that retention must be implemented in CLI until library support added

Severity assessed in Round 3 by AI 2 on 2025-09-12 15:45+02:00
