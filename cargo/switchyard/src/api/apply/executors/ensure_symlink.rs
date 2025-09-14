use std::time::Instant;

use serde_json::json;
use uuid::Uuid;

use crate::api::apply::audit_fields::{insert_hashes, maybe_warn_fsync};
use crate::api::apply::perf::PerfAgg;
use crate::api::errors::map::map_swap_error;
use crate::api::errors::{exit_code_for, id_str, ErrorId};
use crate::api::Switchyard;
use crate::constants::FSYNC_WARN_MS;
use crate::fs::meta::{kind_of, resolve_symlink_target, sha256_hex_of};
use crate::fs::swap::replace_file_with_symlink;
use crate::logging::audit::{ensure_provenance, AuditCtx};
use crate::logging::StageLogger;
use crate::logging::{AuditSink, FactsEmitter};
use crate::types::{ids::action_id, Action};

use super::ActionExecutor;

pub(crate) struct EnsureSymlinkExec;

impl<E: FactsEmitter, A: AuditSink> ActionExecutor<E, A> for EnsureSymlinkExec {
    #[allow(
        clippy::too_many_lines,
        reason = "Will be split further in PR6; executor remains verbose for parity"
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
        let Action::EnsureSymlink { source, target } = act else {
            return (
                None,
                Some("expected EnsureSymlink".to_string()),
                PerfAgg::default(),
            );
        };

        let aid = action_id(pid, act, idx);
        // Attempt fact
        {
            let slog = StageLogger::new(tctx);
            slog.apply_attempt()
                .merge(&json!({
                    "action_id": aid.to_string(),
                    "path": target.as_path().display().to_string(),
                    "safepath_validation": "success",
                    "backup_durable": api.policy.durability.backup_durability,
                }))
                .emit_success();
        }

        let degraded_used: bool;
        let mut fsync_ms: u64 = 0;
        let before_kind = kind_of(&target.as_path());
        // Compute before/after hashes (time the operation)
        let th0 = Instant::now();
        let before_hash = match resolve_symlink_target(&target.as_path()) {
            Some(p) => sha256_hex_of(&p),
            None => sha256_hex_of(&target.as_path()),
        };
        let after_hash = sha256_hex_of(&source.as_path());
        let hash_ms = u64::try_from(th0.elapsed().as_millis()).unwrap_or(u64::MAX);
        match replace_file_with_symlink(
            source,
            target,
            dry,
            matches!(
                api.policy.apply.exdev,
                crate::policy::types::ExdevPolicy::DegradedFallback
            ),
            &api.policy.backup.tag,
        ) {
            Ok((d, ms)) => {
                degraded_used = d;
                fsync_ms = ms;
            }
            Err(e) => {
                // Map to stable error id via facade
                let id = map_swap_error(&e);
                let msg = format!(
                    "symlink {} -> {} failed: {}",
                    source.as_path().display(),
                    target.as_path().display(),
                    e
                );
                // Emit result with failure now
                let mut extra = json!({
                    "action_id": aid.to_string(),
                    "path": target.as_path().display().to_string(),
                    // On failure explicitly record degraded=false and reason when EXDEV
                    "degraded": Some(false),
                    "degraded_reason": if matches!(id, ErrorId::E_EXDEV) { Some("exdev_fallback") } else { None },
                    "error_detail": if matches!(id, ErrorId::E_EXDEV) { Some("exdev_fallback_failed") } else { None },
                    "duration_ms": fsync_ms,
                    "fsync_ms": fsync_ms,
                    "lock_wait_ms": 0u64,
                    "before_kind": before_kind,
                    "after_kind": if dry { "symlink".to_string() } else { kind_of(&target.as_path()).to_string() },
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
                ensure_provenance(&mut extra);
                insert_hashes(&mut extra, before_hash.as_ref(), after_hash.as_ref());
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
                        backup: 0,
                        swap: fsync_ms,
                    },
                );
            }
        }

        // Success path: emit result
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
        ensure_provenance(&mut extra);
        insert_hashes(&mut extra, before_hash.as_ref(), after_hash.as_ref());
        maybe_warn_fsync(&mut extra, fsync_ms, FSYNC_WARN_MS);
        StageLogger::new(tctx)
            .apply_result()
            .merge(&extra)
            .emit_success();

        (
            Some(act.clone()),
            None,
            PerfAgg {
                hash: hash_ms,
                backup: 0,
                swap: fsync_ms,
            },
        )
    }
}
