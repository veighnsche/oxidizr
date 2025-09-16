use std::time::Instant;

use serde_json::json;
use uuid::Uuid;

use crate::api::apply::perf::PerfAgg;
use crate::api::errors::map::map_restore_error_kind;
use crate::api::errors::{exit_code_for, id_str};
use crate::api::Switchyard;
use crate::fs::meta::{kind_of, sha256_hex_of};
use crate::logging::audit::{ensure_provenance, AuditCtx};
use crate::logging::StageLogger;
use crate::logging::{AuditSink, FactsEmitter};
use crate::types::{ids::action_id, Action};

use super::ActionExecutor;

pub(crate) struct RestoreFromBackupExec;

impl<E: FactsEmitter, A: AuditSink> ActionExecutor<E, A> for RestoreFromBackupExec {
    #[allow(
        clippy::too_many_lines,
        reason = "Will be split in PR6; executor remains verbose for parity"
    )]
    fn execute(
        &self,
        api: &Switchyard<E, A>,
        tctx: &AuditCtx<'_>,
        pid: &Uuid,
        act: &Action,
        idx: usize,
        dry: bool,
    ) -> (Option<Action>, Option<String>, PerfAgg) {
        let Action::RestoreFromBackup { target } = act else {
            return (
                None,
                Some("expected RestoreFromBackup".to_string()),
                PerfAgg::default(),
            );
        };
        let aid = action_id(pid, act, idx);

        StageLogger::new(tctx)
            .apply_attempt()
            .merge(&json!({
                "action_id": aid.to_string(),
                "path": target.as_path().display().to_string(),
                "safepath_validation": "success",
                "backup_durable": api.policy.durability.backup_durability,
            }))
            .emit_success();

        let before_kind = kind_of(&target.as_path());
        let mut backup_ms = 0u64;
        let force =
            api.policy.apply.best_effort_restore || !api.policy.durability.sidecar_integrity;
        // Pre-compute sidecar integrity verification (best-effort) before restore
        let th0 = Instant::now();
        let integrity_verified = (|| {
            let pair = crate::fs::backup::find_latest_backup_and_sidecar(
                &target.as_path(),
                &api.policy.backup.tag,
            )?;
            let (backup_opt, sc_path) = pair;
            let sc = crate::fs::backup::read_sidecar(&sc_path).ok()?;
            if let (Some(backup), Some(hash)) = (backup_opt, sc.payload_hash) {
                let actual = sha256_hex_of(&backup)?;
                Some(actual == hash)
            } else {
                None
            }
        })();
        let hash_ms = u64::try_from(th0.elapsed().as_millis()).unwrap_or(u64::MAX);

        // Idempotence fast-path: only when not using the previous snapshot selector.
        // If `capture_restore_snapshot` is enabled, we intend to restore to the state prior to
        // the snapshot we are about to capture, so we must NOT short-circuit.
        let will_use_prev = !dry && api.policy.apply.capture_restore_snapshot;
        if !dry && !will_use_prev {
            if let Some((_bopt, sc_path)) = crate::fs::backup::find_latest_backup_and_sidecar(
                &target.as_path(),
                &api.policy.backup.tag,
            ) {
                if let Ok(sc) = crate::fs::backup::read_sidecar(&sc_path) {
                    if crate::fs::restore::idempotence::is_idempotent(
                        &target.as_path(),
                        sc.prior_kind.as_str(),
                        sc.prior_dest.as_deref(),
                    ) {
                        let mut extra = json!({
                            "action_id": aid.to_string(),
                            "path": target.as_path().display().to_string(),
                            "before_kind": before_kind,
                            "after_kind": before_kind,
                            "idempotent": true,
                            "backup_durable": api.policy.durability.backup_durability,
                        });
                        ensure_provenance(&mut extra);
                        StageLogger::new(tctx)
                            .apply_result()
                            .merge(&extra)
                            .emit_success();
                        return (
                            Some(act.clone()),
                            None,
                            PerfAgg {
                                hash: hash_ms,
                                backup: 0,
                                swap: 0,
                            },
                        );
                    }
                }
            }
        }

        // If configured, capture a pre-restore snapshot of the current state to enable invertibility.
        // When we capture pre-restore, we will restore from the previous snapshot (second newest)
        // so that the inverse plan can later restore the pre-restore state via the latest snapshot.
        if !dry && api.policy.apply.capture_restore_snapshot {
            let t_backup_start = Instant::now();
            let _ = crate::fs::backup::create_snapshot(&target.as_path(), &api.policy.backup.tag);
            backup_ms = backup_ms.saturating_add(
                u64::try_from(t_backup_start.elapsed().as_millis()).unwrap_or(u64::MAX),
            );
        }

        // Perform restore from backup set using appropriate selector
        let restore_res = if !dry && api.policy.apply.capture_restore_snapshot {
            crate::fs::restore::restore_file_prev(target, dry, force, &api.policy.backup.tag)
        } else {
            crate::fs::restore::restore_file(target, dry, force, &api.policy.backup.tag)
        };

        match restore_res {
            Ok(()) => {}
            Err(e) => {
                let id = map_restore_error_kind(e.kind());
                let msg = format!("restore {} failed: {}", target.as_path().display(), e);
                let mut extra = json!({
                    "action_id": aid.to_string(),
                    "path": target.as_path().display().to_string(),
                    "before_kind": before_kind,
                    "after_kind": if dry { before_kind } else { kind_of(&target.as_path()) },
                });
                // Attach ownership provenance best-effort
                if let Some(owner) = &api.owner {
                    if let Ok(info) = owner.owner_of(target) {
                        if let Some(obj) = extra.as_object_mut() {
                            let prov = obj.entry("provenance".to_string()).or_insert(json!({}));
                            if let Some(pobj) = prov.as_object_mut() {
                                pobj.insert("uid".to_string(), json!(info.uid));
                                pobj.insert("gid".to_string(), json!(info.gid));
                                pobj.insert("pkg".to_string(), json!(info.pkg));
                            }
                        }
                    }
                }
                if let Some(iv) = integrity_verified {
                    if let Some(obj) = extra.as_object_mut() {
                        obj.insert("sidecar_integrity_verified".into(), json!(iv));
                    }
                }
                ensure_provenance(&mut extra);
                if let Some(obj) = extra.as_object_mut() {
                    obj.insert("error_id".to_string(), json!(id_str(id)));
                    obj.insert("exit_code".to_string(), json!(exit_code_for(id)));
                }
                StageLogger::new(tctx)
                    .apply_result()
                    .merge(&extra)
                    .emit_failure();
                return (
                    None,
                    Some(msg),
                    PerfAgg {
                        hash: hash_ms,
                        backup: backup_ms,
                        swap: 0,
                    },
                );
            }
        }

        // Success path
        let mut extra = json!({
            "action_id": aid.to_string(),
            "path": target.as_path().display().to_string(),
            "before_kind": before_kind,
            "after_kind": if dry { before_kind } else { kind_of(&target.as_path()) },
            "backup_durable": api.policy.durability.backup_durability,
        });
        // Attach ownership provenance best-effort
        if let Some(owner) = &api.owner {
            if let Ok(info) = owner.owner_of(target) {
                if let Some(obj) = extra.as_object_mut() {
                    let prov = obj.entry("provenance".to_string()).or_insert(json!({}));
                    if let Some(pobj) = prov.as_object_mut() {
                        pobj.insert("uid".to_string(), json!(info.uid));
                        pobj.insert("gid".to_string(), json!(info.gid));
                        pobj.insert("pkg".to_string(), json!(info.pkg));
                    }
                }
            }
        }
        if let Some(iv) = integrity_verified {
            if let Some(obj) = extra.as_object_mut() {
                obj.insert("sidecar_integrity_verified".into(), json!(iv));
            }
        }
        ensure_provenance(&mut extra);
        // Note: Do not capture a post-restore snapshot here. Keeping only the pre-restore snapshot
        // ensures that a subsequent inverse restore selects the intended pre-restore state via the
        // 'previous' selector. Capturing a post-restore snapshot would shift the window such that
        // 'previous' no longer points to the pre-restore symlink snapshot, breaking invertibility.

        StageLogger::new(tctx)
            .apply_result()
            .merge(&extra)
            .emit_success();

        (
            Some(act.clone()),
            None,
            PerfAgg {
                hash: hash_ms,
                backup: backup_ms,
                swap: 0,
            },
        )
    }
}
