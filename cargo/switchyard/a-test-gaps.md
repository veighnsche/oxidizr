# Missing Test Coverage (Gaps) — Switchyard

Date: 2025-09-12

This file lists concrete scenarios that are not currently exercised by the test suite, with suggested assertions and pointers to implementation sites.

## API / Apply

- __Attestation in apply.result on success__
  - Behavior: `src/api/apply.rs` emits an `attestation` object (sig_alg, signature, bundle_hash, public_key_id) when an `Attestor` is configured and apply succeeds in Commit.
  - Gap: No test verifies presence and redaction masking of `attestation.signature`, `bundle_hash`, and `public_key_id`.
  - Test idea: Inject a mock `Attestor` returning a fixed Signature and key id, run a single successful `EnsureSymlink`, assert `apply.result` includes `attestation` fields (unredacted in raw; masked after `redact_event`).

- __apply.result summary default mapping for non-smoke failures__
  - Behavior: When final summary is failure and not smoke-related, apply summary defaults `error_id/exit_code` to `E_POLICY/10` unless already set.
  - Gap: No test that forces a non-smoke, non-gating per-action failure and verifies summary `apply.result` picks `E_POLICY/10` via `.or_insert(...)` path.
  - Test idea: Cause `handle_ensure_symlink` to fail with `E_ATOMIC_SWAP` while ensuring at least one action executed, and verify the summary `apply.result` includes `E_POLICY/10` (since smoke didn’t fail). Alternatively, validate explicit mapping rules.

## Policy / Preflight

- __require_preservation gating__
  - Behavior: `src/api/preflight.rs` adds STOP when `require_preservation=true` and `preservation_supported=false` for target.
  - Gap: No focused test that toggles `require_preservation=true` and asserts a preflight STOP with summary `E_POLICY/10`.
  - Test idea: On a target path where `detect_preservation_capabilities` returns `supported=false` (e.g., non-existent or constrained), assert `pf.ok=false` and summary includes `E_POLICY/10` (already covered indirectly in summary mapping tests, but not specifically for preservation).

- __extra_mount_checks propagation__
  - Behavior: `policy.extra_mount_checks` is checked in both preflight and apply gating.
  - Gap: No test populates `extra_mount_checks` and asserts row `notes` and STOP reasons include the custom path, and that apply gating produces `E_POLICY/10` when the check fails.
  - Test idea: Set `extra_mount_checks=[<temp root>]` and use a mock/override to force `ensure_mount_rw_exec` to fail; assert preflight notes and apply gating results.

## Filesystem / Backup & Restore

- __Sidecar field completeness and schema__
  - Behavior: `BackupSidecar` carries `schema`, `prior_kind`, optional `prior_dest`, and `mode`.
  - Gap: No test asserts that `mode` is serialized for file topology and preserved on restore (there is mode-set code in `restore.rs`).
  - Test idea: Create a file with a distinct mode, snapshot via `replace_file_with_symlink` path, run `restore_file`, assert resulting file’s mode matches sidecar’s `mode`.

- __Legacy rename fallback when sidecar missing__
  - Behavior: If sidecar missing but payload exists, restore falls back to `renameat`.
  - Gap: We only test E_RESTORE_FAILED; no explicit test deletes sidecar then restores successfully via payload-only fallback.
  - Test idea: Create payload and then remove `.meta.json`, run `restore_file`, assert success and post-state.

- __Previous snapshot path (`restore_file_prev`) success path__
  - Behavior: When `capture_restore_snapshot=true`, handler tries previous snapshot first.
  - Gap: We don’t have a direct FS test that ensures `restore_file_prev` path restores to the second-newest.
  - Test idea: Create two snapshots by two mutating cycles, call `restore_file_prev`, assert restored state is from the older snapshot.

## Logging / Redaction

- __Redaction masks provenance.helper and attestation fields__
  - Behavior: `src/logging/redact.rs` masks `provenance.helper` and attestation keys.
  - Gap: No test constructs an event with attestation fields to ensure masking.
  - Test idea: Covered alongside the Attestation test above.

- __FSYNC warn severity__
  - Behavior: `apply/audit_fields.rs::maybe_warn_fsync` sets `severity=warn` if `fsync_ms` > `FSYNC_WARN_MS`.
  - Gap: No test forces a high `fsync_ms` to assert the warn annotation exists pre-redaction and is removed by redaction.
  - Test idea: Inject a fake high duration (needs minor injection point or feature flag) and assert behavior.

## Adapters

- __Ownership strict policy path__
  - Behavior: With `strict_ownership=true` and no `OwnershipOracle`, preflight STOPs; with oracle, uses `owner_of` and may STOP on error.
  - Gap: No test explicitly toggles `strict_ownership` and asserts preflight STOPs with appropriate note.
  - Test idea: Enable `strict_ownership=true` without oracle; assert STOP mentions missing oracle. With oracle that errors for target, assert STOP mentions strict ownership failure.

- __Attestation adapter integration__
  - Behavior: `Attestor` used by apply summary on success.
  - Gap: No test injects a mock `Attestor` and verifies attestation fields present and masked after redaction.
  - Test idea: See Attestation in API / Apply above.

## Misc / Determinism & IDs

- __UUIDv5 determinism for `plan_id`/`action_id`__
  - Behavior: Deterministic IDs for the same `Plan`.
  - Gap: No unit test that builds the same `Plan` twice and asserts identical `plan_id` and per-action `action_id`s.
  - Test idea: Build twice with same inputs; compare.

## Suggested file structure for new tests

- New unit tests in `cargo/switchyard/tests/`:
  - `attestation_apply_success.rs` (covers attestation emission and redaction).
  - `preflight_preservation_required.rs`.
  - `preflight_extra_mount_checks.rs` (may need injectability or environment control for failure).
  - `restore_payload_only_fallback.rs`.
  - `restore_prev_snapshot.rs`.
  - `ids_determinism.rs`.
  - `fsync_warn_annotation.rs` (consider feature gate or injection hook to simulate high fsync_ms).
  - `strict_ownership_gate.rs`.

Each proposed test should:

- Use a temp root via `tempfile::tempdir()`.
- Construct `SafePath` via `SafePath::from_rooted(root, ...)`.
- Prefer policy flags to guide paths (e.g., `allow_unlocked_commit`, `force_untrusted_source`, etc.).
- Use `FactsEmitter` test doubles for capturing facts where needed.
- Assert redacted or raw events appropriately based on the behavior under test.
