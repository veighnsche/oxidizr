use serde_json::Value;

use switchyard::logging::redact::redact_event;

// Remove volatile or run-specific fields prior to comparing
pub fn normalize_for_compare(mut v: Value) -> Value {
    if let Some(obj) = v.as_object_mut() {
        obj.remove("run_id");
        obj.remove("event_id");
        obj.remove("seq");
        obj.remove("switchyard_version");
    }
    v
}

pub fn redact_and_normalize(v: Value) -> Value {
    normalize_for_compare(redact_event(v))
}

pub fn filter_by_stage<'a, I>(events: I, stages: &[&str]) -> Vec<Value>
where
    I: IntoIterator<Item = Value>,
{
    let mut out = Vec::new();
    for e in events {
        if let Some(s) = e.get("stage").and_then(|v| v.as_str()) {
            if stages.iter().any(|t| *t == s) {
                out.push(e);
            }
        }
    }
    out
}

pub fn filter_apply_result_per_action<'a, I>(events: I) -> Vec<Value>
where
    I: IntoIterator<Item = Value>,
{
    events
        .into_iter()
        .filter(|e| e.get("stage").and_then(|v| v.as_str()) == Some("apply.result")
            && e.get("action_id").is_some())
        .collect()
}

pub fn sort_by_stage_action_path(events: &mut [Value]) {
    events.sort_by_key(|e| {
        let st = e.get("stage").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let aid = e.get("action_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let p = e.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        (st, aid, p)
    });
}

pub fn sort_by_action_id(events: &mut [Value]) {
    events.sort_by_key(|e| e.get("action_id").and_then(|v| v.as_str()).unwrap_or("").to_string());
}
