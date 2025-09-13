use serde_json::{json, Value};

use crate::api::errors::{exit_code_for, id_str, ErrorId};
use crate::logging::StageLogger;

use super::perf::PerfAgg;

pub(crate) struct ApplySummary {
    fields: Value,
}

impl ApplySummary {
    pub(crate) fn new(lock_backend: &str, lock_wait_ms: Option<u64>) -> Self {
        let fields = json!({
            "lock_backend": lock_backend,
            "lock_wait_ms": lock_wait_ms,
        });
        Self { fields }
    }

    pub(crate) fn perf(mut self, total: PerfAgg) -> Self {
        if let Some(obj) = self.fields.as_object_mut() {
            obj.insert(
                "perf".to_string(),
                json!({
                    "hash_ms": total.hash,
                    "backup_ms": total.backup,
                    "swap_ms": total.swap,
                }),
            );
        }
        self
    }

    pub(crate) fn errors(mut self, errors: &[String]) -> Self {
        if let Some(obj) = self.fields.as_object_mut() {
            // Compute chain best-effort from collected error messages
            let chain = crate::api::errors::infer_summary_error_ids(errors);
            obj.insert("summary_error_ids".to_string(), json!(chain));
        }
        self
    }

    pub(crate) fn smoke_or_policy_mapping(mut self, errors: &[String]) -> Self {
        if errors.is_empty() {
            return self;
        }
        if let Some(obj) = self.fields.as_object_mut() {
            if errors.iter().any(|e| e.contains("smoke")) {
                obj.insert("error_id".to_string(), json!(id_str(ErrorId::E_SMOKE)));
                obj.insert(
                    "exit_code".to_string(),
                    json!(exit_code_for(ErrorId::E_SMOKE)),
                );
            } else {
                // Default summary mapping for non-smoke failures
                obj.entry("error_id")
                    .or_insert(json!(id_str(ErrorId::E_POLICY)));
                obj.entry("exit_code")
                    .or_insert(json!(exit_code_for(ErrorId::E_POLICY)));
            }
        }
        self
    }

    pub(crate) fn attestation<E: crate::logging::FactsEmitter, A: crate::logging::AuditSink>(
        mut self,
        api: &crate::api::Switchyard<E, A>,
        pid: uuid::Uuid,
        executed_len: usize,
        rolled_back: bool,
    ) -> Self {
        if let Some(att) = &api.attest {
            let bundle_json = json!({
                "plan_id": pid.to_string(),
                "executed": executed_len,
                "rolled_back": rolled_back,
            });
            let bundle: Vec<u8> = serde_json::to_vec(&bundle_json).unwrap_or_default();
            if let Some(att_json) =
                crate::adapters::attest::build_attestation_fields(&**att, &bundle)
            {
                if let Some(obj) = self.fields.as_object_mut() {
                    obj.insert("attestation".to_string(), att_json);
                }
            }
        }
        self
    }

    pub(crate) fn emit(self, slog: &StageLogger<'_>, decision: &str) {
        match decision {
            "failure" => slog.apply_result().merge(&self.fields).emit_failure(),
            _ => slog.apply_result().merge(&self.fields).emit_success(),
        }
    }
}
