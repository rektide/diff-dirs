//! Descriptive historical CSV reading and explicitly separate trace/resource checks.
use crate::{Result, artifacts, process, write_json};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

pub fn summarize_csv(paths: &[PathBuf]) -> Result<Value> {
    let mut groups = Vec::new();
    for path in paths {
        let mut reader = csv::Reader::from_path(path)?;
        let headers = reader.headers()?.clone();
        let (column, scale) = if let Some(i) = headers.iter().position(|s| s == "elapsed_ns" || s == "ns") {
            (i, 1e-9)
        } else if let Some(i) = headers.iter().position(|s| s == "seconds") {
            (i, 1.0)
        } else {
            return Err("CSV needs elapsed_ns, ns or seconds".into());
        };
        let dimensions: Vec<_> = headers
            .iter()
            .enumerate()
            .filter(|(_, name)| {
                [
                    "sample",
                    "fixture",
                    "scenario",
                    "policy",
                    "execution",
                    "backend",
                    "check",
                    "threads",
                    "variant",
                    "label",
                    "configuration",
                ]
                .contains(name)
            })
            .map(|(i, s)| (i, s.to_string()))
            .collect();
        let mut values = BTreeMap::<Vec<String>, Vec<f64>>::new();
        for row in reader.records() {
            let row = row?;
            let value = row.get(column).ok_or("missing latency")?.parse::<f64>()? * scale;
            if !value.is_finite() || value <= 0.0 {
                return Err("latency must be positive and finite".into());
            }
            let key = dimensions.iter().map(|(i, _)| row.get(*i).unwrap_or("").to_string()).collect();
            values.entry(key).or_default().push(value);
        }
        for (key, mut times) in values {
            times.sort_by(f64::total_cmp);
            let n = times.len();
            let mean = times.iter().sum::<f64>() / n as f64;
            let median = if n.is_multiple_of(2) {
                (times[n / 2 - 1] + times[n / 2]) / 2.0
            } else {
                times[n / 2]
            };
            groups.push(
                json!({"input":path,"dimensions":dimensions.iter().map(|(_,s)|s.clone()).zip(key).collect::<BTreeMap<_,_>>(),
                "n":n,"median_seconds":median,"min_seconds":times[0],"max_seconds":times[n-1],"mean_seconds":mean,
                "population_stddev_seconds":(times.iter().map(|x|(x-mean).powi(2)).sum::<f64>()/n as f64).sqrt()}),
            );
        }
    }
    Ok(json!({"groups":groups,"limits":"descriptive per-file groups only; no cross-file pooling, pairing, significance, or inferred timing-scope equivalence"}))
}

pub fn audit(trace: &Path, fixture: &str) -> Result<Value> {
    if fixture.is_empty() {
        return Err("fixture substring must not be empty".into());
    }
    let mut opens = 0;
    let mut links = 0;
    let mut violations = Vec::new();
    for line in BufReader::new(File::open(trace)?).lines() {
        let line = line?;
        if !line.contains(fixture) {
            continue;
        }
        let Some(paren) = line.find('(') else {
            continue;
        };
        let syscall = line[..paren].split_whitespace().last().unwrap_or("");
        match syscall {
            "open" | "openat" | "openat2" => {
                opens += 1;
                if !line.contains("O_DIRECTORY") {
                    violations.push(line);
                }
            }
            "read" | "readv" | "pread64" | "preadv" | "preadv2" | "mmap" | "write" | "pwrite64" | "unlink" | "unlinkat" | "rename" | "renameat"
            | "renameat2" => violations.push(line),
            "readlink" | "readlinkat" => links += 1,
            _ => {}
        }
    }
    if opens == 0 {
        return Err("no fixture directory opens matched; use fd-annotated strace and an exact fixture substring".into());
    }
    if !violations.is_empty() {
        return Err(format!("unexpected fixture payload IO/mutation: {}", violations.join("\n")).into());
    }
    Ok(
        json!({"fixture_directory_opens":opens,"symlink_reads":links,"payload_opens_reads_mmaps_writes":0,
        "scope":"only named traced syscalls/path substring; not proof of untraced behavior"}),
    )
}

pub fn resources(binary: &Path, output: &Path, args: &[String]) -> Result<()> {
    let _lock = crate::host_lock()?;
    let binary = fs::canonicalize(binary)?;
    let mut magic = [0; 4];
    File::open(&binary)?.read_exact(&mut magic)?;
    if magic != *b"\x7fELF" {
        return Err("resource observation requires a native ELF binary".into());
    }
    fs::create_dir(output)?;
    let output = fs::canonicalize(output)?;
    let result = process::run(Command::new(&binary).args(args), &output.join("process"), Duration::from_secs(1800))?;
    write_json(
        &output.join("resources.json"),
        &json!({"binary":binary,"sha256":artifacts::sha256(&binary)?,"process":result,
        "scope":"whole native process CPU/peak RSS, including startup and everything it executes; not an API timer"}),
    )?;
    if result.code != Some(0) || result.timed_out {
        return Err("resource child failed; observation retained".into());
    }
    println!("{}", output.join("resources.json").display());
    Ok(())
}

#[test]
fn csv_scopes_stay_separate_and_pid_prefixed_audit_rejects_payload() {
    let own = crate::scratch();
    let csv = own.path().join("input.csv");
    fs::write(&csv, "fixture,scenario,ns\nx,clean,100\nx,clean,300\n").unwrap();
    let summary = summarize_csv(&[csv]).unwrap();
    assert_eq!(summary["groups"][0]["n"], 2);
    let trace = own.path().join("trace");
    fs::write(
        &trace,
        "123 openat(AT_FDCWD, \"/owned/payload\", O_RDONLY|O_DIRECTORY) = 3\n123 readlink(\"/owned/payload/link\", \"target\", 6) = 6\n",
    )
    .unwrap();
    assert_eq!(audit(&trace, "/owned/payload").unwrap()["fixture_directory_opens"], 1);
    fs::write(&trace, "123 openat(AT_FDCWD, \"/owned/payload/file\", O_RDONLY) = 3\n").unwrap();
    assert!(audit(&trace, "/owned/payload").is_err());
}
