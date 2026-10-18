use crate::{
    Result,
    artifacts::{self, Control},
    fixture::{self, Fixture},
    host_lock, report,
    scheduling::schedule,
    source_identity, write_json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sample {
    pub sequence: usize,
    pub block: usize,
    pub position: usize,
    pub label: String,
    pub argv: Vec<String>,
    pub preparation_ns: u128,
    pub process_ns: u128,
    pub verification_ns: u128,
    pub api_ns: Option<u128>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

pub fn clean_ns(stdout: &str, label: &str, mode: &str, threads: usize) -> Result<u128> {
    let lines: Vec<_> = stdout.lines().collect();
    if lines.len() != 1 {
        return Err("expected exactly one clean observation".into());
    }
    let fields: Vec<_> = lines[0].split(',').collect();
    if fields.len() != 15
        || fields[0] != label
        || fields[1] != mode
        || fields[2] != threads.to_string()
        || fields[3] != "ReflinkRequired"
        || fields[4] != "0"
        || fields[5] != "clean"
        || fields[7..].iter().any(|f| *f != "0")
    {
        return Err("invalid clean identity/scenario or nonzero clean operations".into());
    }
    let ns = fields[6].parse()?;
    if ns == 0 {
        return Err("zero API time".into());
    }
    Ok(ns)
}

pub fn invoke(binary: &Path, args: &[String], stem: &Path) -> Result<(Option<i32>, u128)> {
    let out = OpenOptions::new().write(true).create_new(true).open(stem.with_extension("stdout"))?;
    let err = OpenOptions::new().write(true).create_new(true).open(stem.with_extension("stderr"))?;
    let start = Instant::now();
    let mut child = Command::new(binary).args(args).stdin(Stdio::null()).stdout(out).stderr(err).spawn()?;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok((status.code(), start.elapsed().as_nanos()));
        }
        if start.elapsed().as_secs() >= 1800 {
            child.kill()?;
            child.wait()?;
            return Err("native control timed out after 1800 seconds".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

pub fn run(root: &Path, output: &Path, controls: &[Control], blocks: usize, seed: u64, threads: usize) -> Result<bool> {
    let _lock = host_lock()?;
    let order = schedule(controls.len(), blocks, seed)?;
    let root = fs::canonicalize(root)?;
    let fixture: Fixture = serde_json::from_slice(&fs::read(root.join("fixture.json"))?)?;
    fs::create_dir(output)?;
    let output = fs::canonicalize(output)?;
    let start = Instant::now();
    let controls = artifacts::preserve(controls, &output)?;
    let preservation = start.elapsed().as_nanos();
    let initial = Instant::now();
    let check = fixture::verify(&root, &fixture, true).map_err(|e| e.to_string());
    write_json(
        &output.join("campaign.json"),
        &json!({"schema":1,"fixture":fixture,"controls":controls,
        "seed":seed,"blocks":order,"threads":threads,"artifact_preservation_ns":preservation,
        "runner":source_identity(),"runner_sha256":artifacts::sha256(&std::env::current_exe()?)?,
        "initial_verification_ns":initial.elapsed().as_nanos(),"initial_error":check.as_ref().err(),
        "scope":"API from checked-bench; process includes startup/pool/untimed initialization and 2ms polling",
        "sampling":"mirrored blocks; child warms outside API timer; no cache dropping"}),
    )?;
    check.map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    let mut raw = OpenOptions::new().write(true).create_new(true).open(output.join("samples.jsonl"))?;
    let mut samples = Vec::new();
    for (block, indices) in order.iter().enumerate() {
        for (position, &index) in indices.iter().enumerate() {
            let control = &controls[index];
            let sequence = samples.len();
            let stem = output.join(format!("sample-{sequence:04}-{}", control.label));
            let argv = vec![
                "clean-repeat".into(),
                root.display().to_string(),
                control.label.clone(),
                control.mode.clone(),
                threads.to_string(),
                "reflink".into(),
                "1".into(),
            ];
            let mut sample = Sample {
                sequence,
                block,
                position,
                label: control.label.clone(),
                argv,
                preparation_ns: 0,
                process_ns: 0,
                verification_ns: 0,
                api_ns: None,
                exit_code: None,
                error: None,
            };
            let attempt = (|| -> Result<()> {
                let start = Instant::now();
                let prep = artifacts::verify(control).and_then(|()| fixture::verify(&root, &fixture, false));
                sample.preparation_ns = start.elapsed().as_nanos();
                prep?;
                let (exit, ns) = invoke(&control.binary, &sample.argv, &stem)?;
                sample.process_ns = ns;
                sample.exit_code = exit;
                if exit != Some(0) {
                    return Err(format!("child failed: {exit:?}").into());
                }
                let start = Instant::now();
                let verify = (|| -> Result<u128> {
                    let ns = clean_ns(&fs::read_to_string(stem.with_extension("stdout"))?, &control.label, &control.mode, threads)?;
                    fixture::verify(&root, &fixture, false)?;
                    Ok(ns)
                })();
                sample.verification_ns = start.elapsed().as_nanos();
                sample.api_ns = Some(verify?);
                Ok(())
            })();
            if let Err(e) = attempt {
                sample.error = Some(e.to_string());
                sample.api_ns = None;
            }
            serde_json::to_writer(&mut raw, &sample)?;
            raw.write_all(b"\n")?;
            raw.flush()?;
            raw.sync_data()?;
            samples.push(sample);
        }
    }
    let start = Instant::now();
    let check = fixture::verify(&root, &fixture, true).map_err(|e| e.to_string());
    let passed = check.is_ok() && samples.iter().all(|s| s.error.is_none());
    let pairs = report::pairs(&samples, &controls);
    write_json(
        &output.join("summary.json"),
        &json!({"passed":passed,"final_verification_ns":start.elapsed().as_nanos(),"final_error":check.err(),
        "paired_spreads":report::spreads(&pairs),"pairs":pairs,"failed_samples":samples.iter().filter(|s|s.error.is_some()).count(),
        "interpretation":"plumbing evidence only; descriptive pairs, no significance/load correction; failed samples excluded"}),
    )?;
    println!("{}", output.join("summary.json").display());
    Ok(passed)
}

#[test]
fn parser_refuses_dirty_or_extra_rows() {
    let row = "default,none,0,ReflinkRequired,0,clean,123,0,0,0,0,0,0,0,0";
    assert_eq!(clean_ns(row, "default", "none", 0).unwrap(), 123);
    assert!(clean_ns(&format!("{row}\n{row}"), "default", "none", 0).is_err());
    assert!(clean_ns(&row.replace(",123,0,", ",123,1,"), "default", "none", 0).is_err());
}

#[test]
fn artifact_corruption_is_retained_and_not_paired() {
    let scratch = crate::scratch();
    let root = scratch.path().join("fixture");
    fixture::setup(&root, None).unwrap();
    let binary = scratch.path().join("control");
    fs::write(&binary, b"original").unwrap();
    let c = Control {
        label: "baseline".into(),
        binary: binary.clone(),
        sha256: artifacts::sha256(&binary).unwrap(),
        mode: "none".into(),
        provenance_path: binary.clone(),
        provenance: json!({}),
        provenance_sha256: "test".into(),
    };
    fs::write(binary, b"corrupt").unwrap();
    let mut other = c.clone();
    other.label = "candidate".into();
    let output = scratch.path().join("run");
    assert!(!run(&root, &output, &[c, other], 2, 0, 0).unwrap());
    let raw = fs::read_to_string(output.join("samples.jsonl")).unwrap();
    assert_eq!(raw.lines().count(), 4);
    assert!(raw.lines().all(|line| line.contains("SHA-256 mismatch")));
    let summary: serde_json::Value = serde_json::from_slice(&fs::read(output.join("summary.json")).unwrap()).unwrap();
    assert_eq!(summary["failed_samples"], 4);
    assert_eq!(summary["pairs"], json!([]));
}
