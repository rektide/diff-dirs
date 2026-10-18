use crate::{
    Result,
    artifacts::{self, Control},
    fixture::{self, Fixture},
    host_lock, process, report,
    scenario::{self, Scenario},
    scheduling::schedule,
    source_identity, write_json,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
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
    #[serde(default)]
    pub native_ns: Option<u128>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    #[serde(default)]
    pub started_unix_ns: u128,
    #[serde(default)]
    pub details: Value,
}

/// Only the domain owns setup/operation/verification. The shared engine owns chronology,
/// artifacts, failure retention and pairing. This is a compile-time Rust interface, not plugins.
pub trait Workload {
    fn initialize(&mut self) -> Result<Value>;
    fn sample(&mut self, control: &Control, stem: &Path, sample: &mut Sample) -> Result<()>;
    fn finish(&mut self) -> Result<Value>;
}

pub fn balanced(output: &Path, controls: &[Control], blocks: usize, seed: u64, workload: &mut impl Workload) -> Result<bool> {
    let _lock = host_lock()?;
    let order = schedule(controls.len(), blocks, seed)?;
    fs::create_dir(output)?;
    let output = fs::canonicalize(output)?;
    let start = Instant::now();
    let controls = artifacts::preserve(controls, &output)?;
    let preservation = start.elapsed().as_nanos();
    let initial = Instant::now();
    let initialized = workload.initialize().map_err(|e| e.to_string());
    write_json(
        &output.join("campaign.json"),
        &json!({"schema":2,"controls":controls,"seed":seed,"blocks":order,
        "artifact_preservation_ns":preservation,"runner":source_identity(),"runner_sha256":artifacts::sha256(&std::env::current_exe()?)?,
        "initial_verification_ns":initial.elapsed().as_nanos(),"initial_error":initialized.as_ref().err(),"workload":initialized.as_ref().ok(),
        "sampling":"seeded mirrored blocks; setup/reset/verification excluded; no cache dropping or load correction"}),
    )?;
    initialized.map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    let mut raw = OpenOptions::new().write(true).create_new(true).open(output.join("samples.jsonl"))?;
    let mut samples = Vec::new();
    for (block, indices) in order.iter().enumerate() {
        for (position, &index) in indices.iter().enumerate() {
            let control = &controls[index];
            let sequence = samples.len();
            let stem = output.join(format!("sample-{sequence:04}-{}", control.label));
            let mut sample = Sample {
                sequence,
                block,
                position,
                label: control.label.clone(),
                argv: Vec::new(),
                preparation_ns: 0,
                process_ns: 0,
                verification_ns: 0,
                api_ns: None,
                native_ns: None,
                exit_code: None,
                error: None,
                started_unix_ns: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
                details: json!({"io_pressure_before":fs::read_to_string("/proc/pressure/io").ok()}),
            };
            let start = Instant::now();
            let checked = artifacts::verify(control);
            sample.preparation_ns = start.elapsed().as_nanos();
            let attempt = checked.and_then(|()| workload.sample(control, &stem, &mut sample));
            if let Err(e) = attempt {
                sample.error = Some(e.to_string());
                sample.api_ns = None;
                sample.native_ns = None;
            }
            sample.details["io_pressure_after"] = json!(fs::read_to_string("/proc/pressure/io").ok());
            serde_json::to_writer(&mut raw, &sample)?;
            raw.write_all(b"\n")?;
            raw.flush()?;
            raw.sync_data()?;
            samples.push(sample);
        }
    }
    let start = Instant::now();
    let final_check = workload.finish().map_err(|e| e.to_string());
    let passed = final_check.is_ok() && samples.iter().all(|s| s.error.is_none());
    let pairs = report::pairs(&samples, &controls);
    write_json(
        &output.join("summary.json"),
        &json!({"passed":passed,"final_verification_ns":start.elapsed().as_nanos(),
        "final_error":final_check.as_ref().err(),"final_checks":final_check.as_ref().ok(),"paired_spreads":report::spreads(&pairs),"pairs":pairs,
        "failed_samples":samples.iter().filter(|s|s.error.is_some()).count(),"interpretation":"descriptive paired spreads only; failures excluded; no significance or causal load correction"}),
    )?;
    println!("{}", output.join("summary.json").display());
    Ok(passed)
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum Task {
    FrozenClean,
    Operation { scenario: Scenario, destinations: usize },
}

struct LibraryWorkload {
    root: PathBuf,
    fixture: Option<Fixture>,
    task: Task,
    threads: usize,
    starting: Option<Vec<Option<String>>>,
}

pub fn run(root: &Path, output: &Path, controls: &[Control], blocks: usize, seed: u64, threads: usize) -> Result<bool> {
    run_task(root, output, controls, blocks, seed, threads, Task::FrozenClean)
}
pub fn run_task(root: &Path, output: &Path, controls: &[Control], blocks: usize, seed: u64, threads: usize, task: Task) -> Result<bool> {
    balanced(
        output,
        controls,
        blocks,
        seed,
        &mut LibraryWorkload {
            root: root.into(),
            fixture: None,
            task,
            threads,
            starting: None,
        },
    )
}

impl Workload for LibraryWorkload {
    fn initialize(&mut self) -> Result<Value> {
        self.root = fs::canonicalize(&self.root)?;
        let fixture: Fixture = serde_json::from_slice(&fs::read(self.root.join("fixture.json"))?)?;
        fixture::verify(&self.root, &fixture, true)?;
        let value = json!({"fixture":fixture,"task":self.task,"threads":self.threads,"scope":"child-owned API timer; process scope retained separately"});
        self.fixture = Some(fixture);
        Ok(value)
    }
    fn sample(&mut self, control: &Control, stem: &Path, sample: &mut Sample) -> Result<()> {
        let fixture = self.fixture.as_ref().ok_or("uninitialized workload")?;
        let start = Instant::now();
        let prepared = (|| -> Result<Option<scenario::Case>> {
            fixture::verify(&self.root, fixture, false)?;
            if let Task::Operation { scenario, destinations } = self.task {
                let case = scenario::prepare(fixture, &stem.with_extension("case"), scenario, destinations)?;
                if let Some(expected) = &self.starting {
                    if expected != &case.starting_equivalence {
                        return Err("unmatched reset starting state".into());
                    }
                } else {
                    self.starting = Some(case.starting_equivalence.clone());
                }
                sample.argv = vec!["sample".into(), case.root.display().to_string(), control.mode.clone(), self.threads.to_string()];
                sample.details["reset"] = json!(case);
                Ok(Some(case))
            } else {
                sample.argv = vec![
                    "clean-repeat".into(),
                    self.root.display().to_string(),
                    control.label.clone(),
                    control.mode.clone(),
                    self.threads.to_string(),
                    "reflink".into(),
                    "1".into(),
                ];
                Ok(None)
            }
        })();
        sample.preparation_ns += start.elapsed().as_nanos();
        let case = prepared?;
        let child = process::run(Command::new(&control.binary).args(&sample.argv), stem, Duration::from_secs(1800))?;
        sample.process_ns = child.elapsed_ns;
        sample.exit_code = child.code;
        let success = child.code == Some(0) && !child.timed_out;
        sample.details["child_process"] = json!(child);
        if !success {
            return Err("native instrument failed; see retained stderr/process record".into());
        }
        let start = Instant::now();
        let checked = (|| -> Result<u128> {
            let text = fs::read_to_string(stem.with_extension("stdout"))?;
            let ns = if let Some(case) = &case {
                let row: Value = serde_json::from_str(&text)?;
                let ns = row["api_ns"].as_u64().ok_or("missing API time")? as u128;
                if row["schema"] != 1
                    || row["mode"] != control.mode
                    || row["threads"] != self.threads
                    || row["destinations"] != case.destinations
                    || row["scenario"] != serde_json::to_value(case.scenario)?
                    || ns == 0
                    || (case.scenario == Scenario::Clean && row["zero_operations"] != true)
                {
                    return Err("invalid operation observation identity".into());
                }
                case.verify()?;
                sample.details["observation"] = row;
                ns
            } else {
                clean_ns(&text, &control.label, &control.mode, self.threads)?
            };
            fixture::verify(&self.root, fixture, false)?;
            Ok(ns)
        })();
        sample.verification_ns = start.elapsed().as_nanos();
        sample.api_ns = Some(checked?);
        Ok(())
    }
    fn finish(&mut self) -> Result<Value> {
        fixture::verify(&self.root, self.fixture.as_ref().ok_or("uninitialized workload")?, true)?;
        Ok(json!({"fixture_unchanged":true}))
    }
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
    let summary: Value = serde_json::from_slice(&fs::read(output.join("summary.json")).unwrap()).unwrap();
    assert_eq!(summary["failed_samples"], 4);
    assert_eq!(summary["pairs"], json!([]));
}
