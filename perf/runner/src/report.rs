use crate::{artifacts::Control, execution::Sample};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn pairs(samples: &[Sample], controls: &[Control]) -> Vec<Value> {
    let baseline = &controls[0].label;
    let mut result = Vec::new();
    for before in samples.iter().filter(|s| &s.label == baseline && s.error.is_none()) {
        for after in samples.iter().filter(|s| s.block == before.block && &s.label != baseline && s.error.is_none()) {
            let old = before.api_ns.or(before.native_ns);
            let new = after.api_ns.or(after.native_ns);
            if before.api_ns.is_some() != after.api_ns.is_some() {
                continue;
            }
            if let (Some(old), Some(new)) = (old, new) {
                result.push(json!({"block":before.block,
                "baseline_sample":before.sequence,"candidate_sample":after.sequence,"candidate":after.label,
                    "scope":if before.api_ns.is_some(){"api"}else{"native-process"},
                    "baseline_ns":old,"candidate_ns":new,"relative_percent":(new as f64/old as f64-1.0)*100.0}));
            }
        }
    }
    result
}

pub fn spreads(pairs: &[Value]) -> BTreeMap<String, Value> {
    let mut grouped = BTreeMap::<String, Vec<f64>>::new();
    for pair in pairs {
        grouped
            .entry(pair["candidate"].as_str().unwrap().to_owned())
            .or_default()
            .push(pair["relative_percent"].as_f64().unwrap());
    }
    grouped
        .into_iter()
        .map(|(label, mut values)| {
            values.sort_by(f64::total_cmp);
            let n = values.len();
            let median = if n.is_multiple_of(2) {
                (values[n / 2 - 1] + values[n / 2]) / 2.0
            } else {
                values[n / 2]
            };
            (
                label,
                json!({"matched_blocks":n,"median_relative_percent":median,"min_relative_percent":values[0],"max_relative_percent":values[n-1]}),
            )
        })
        .collect()
}
