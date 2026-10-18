use crate::{artifacts::Control, execution::Sample};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn retained(root: &std::path::Path) -> crate::Result<Value> {
    let campaign: Value = serde_json::from_slice(&std::fs::read(root.join("campaign.json"))?)?;
    // A completed summary is required; a killed run's prefix is never a finished result.
    let complete: Value = serde_json::from_slice(&std::fs::read(root.join("summary.json"))?)?;
    let controls: Vec<Control> = serde_json::from_value(campaign["controls"].clone())?;
    let order: Vec<Vec<usize>> = serde_json::from_value(campaign["blocks"].clone())?;
    let mut samples = Vec::new();
    for line in std::fs::read_to_string(root.join("samples.jsonl"))?.lines() {
        samples.push(serde_json::from_str::<Sample>(line)?);
    }
    if controls.len() < 2 || samples.len() != order.iter().map(Vec::len).sum::<usize>() {
        return Err("incomplete raw chronology".into());
    }
    for (sequence, (block, position, index)) in order
        .iter()
        .enumerate()
        .flat_map(|(block, row)| row.iter().enumerate().map(move |(position, index)| (block, position, *index)))
        .enumerate()
    {
        let sample = &samples[sequence];
        let control = controls.get(index).ok_or("invalid schedule index")?;
        if sample.sequence != sequence || sample.block != block || sample.position != position || sample.label != control.label {
            return Err("sample does not match planned immutable chronology".into());
        }
        if sample.error.is_none() && (sample.api_ns.is_some() == sample.native_ns.is_some() || sample.api_ns.or(sample.native_ns) == Some(0)) {
            return Err("successful sample needs exactly one positive timing scope".into());
        }
    }
    let pairs = pairs(&samples, &controls);
    Ok(json!({"campaign_passed":complete["passed"],"pairs":pairs,"paired_spreads":spreads(&pairs),
        "failed_samples":samples.iter().filter(|s|s.error.is_some()).count(),"limits":"raw descriptive matched blocks; no win/significance/causal correction"}))
}

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
