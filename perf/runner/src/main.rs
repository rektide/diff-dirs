//! Narrow clean-control driver for the preserved checked-bench executable protocol.
//! No Cargo invocation, dynamic plugins, registry, or live-root mutation in run.
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const MARKER: &str = "checked-streaming owned fixture v1\n";

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// NEW fixture only. An optional source is read-only and privately reflinked.
    Setup {
        root: PathBuf,
        #[arg(long)]
        source: Option<PathBuf>,
    },
    /// Frozen checked-bench clean-repeat protocol only. Each control has five tokens.
    Run {
        root: PathBuf,
        output: PathBuf,
        #[arg(long, default_value_t = 8)]
        blocks: usize,
        #[arg(long, default_value_t = 1701)]
        seed: u64,
        #[arg(long, default_value_t = 0)]
        threads: usize,
        /// Exact preserved fd277144 / 243be8e2 checked-bench artifacts and build records.
        #[arg(long, conflicts_with = "control")]
        frozen: Option<PathBuf>,
        /// LABEL BINARY SHA256 MODE PROVENANCE_JSON; repeat once per variant.
        #[arg(long, num_args = 5, action = clap::ArgAction::Append, required_unless_present = "frozen")]
        control: Vec<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Fixture {
    root: PathBuf,
    full: String,
    metadata: String,
    setup_ns: u128,
    verification_ns: u128,
    source_origin: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize)]
struct Control {
    label: String,
    binary: PathBuf,
    sha256: String,
    mode: String,
    provenance_path: PathBuf,
    provenance: serde_json::Value,
    provenance_sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Sample {
    sequence: usize,
    block: usize,
    position: usize,
    label: String,
    argv: Vec<String>,
    preparation_ns: u128,
    process_ns: u128,
    verification_ns: u128,
    api_ns: Option<u128>,
    exit_code: Option<i32>,
    error: Option<String>,
}

fn main() -> Result<()> {
    match Cli::parse().action {
        Action::Setup { root, source } => setup(&root, source.as_deref())?,
        Action::Run {
            root,
            output,
            blocks,
            seed,
            threads,
            frozen,
            control,
        } => {
            let controls = if let Some(frozen) = frozen {
                frozen_controls(&frozen)?
            } else {
                controls(&control)?
            };
            if !run(&root, &output, &controls, blocks, seed, threads)? {
                return Err("failed samples retained; no successful campaign claim".into());
            }
        }
    }
    Ok(())
}

fn sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}

// Hash a depth-first sorted stream, not a retained full-tree map. Never follow links.
// atime is deliberately excluded: reading fixture bytes must not look like mutation.
fn tree(root: &Path, bytes: bool, inodes: bool) -> Result<String> {
    fn visit(root: &Path, path: &Path, bytes: bool, inodes: bool, hash: &mut Sha256) -> Result<()> {
        let meta = fs::symlink_metadata(path)?;
        let name = path.strip_prefix(root)?.as_os_str().as_bytes();
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name);
        let kind = if meta.is_dir() {
            "dir"
        } else if meta.is_file() {
            "file"
        } else if meta.is_symlink() {
            "link"
        } else {
            return Err("special fixture entry refused".into());
        };
        hash.update(serde_json::to_vec(&(
            kind,
            meta.mode(),
            meta.mtime(),
            meta.mtime_nsec(),
            if meta.is_dir() { 0 } else { meta.len() },
            if inodes { Some((meta.dev(), meta.ino())) } else { None },
        ))?);
        if meta.is_symlink() {
            let target = fs::read_link(path)?;
            let target = target.as_os_str().as_bytes();
            hash.update((target.len() as u64).to_le_bytes());
            hash.update(target);
        } else if meta.is_dir() {
            let mut entries = fs::read_dir(path)?.collect::<std::io::Result<Vec<_>>>()?;
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries {
                if entry.file_name() == ".git" || entry.file_name() == ".jj" {
                    return Err("administrative fixture entry refused".into());
                }
                visit(root, &entry.path(), bytes, inodes, hash)?;
            }
        } else if bytes {
            hash.update(sha256(path)?.as_bytes());
        }
        Ok(())
    }
    let mut hash = Sha256::new();
    visit(root, root, bytes, inodes, &mut hash)?;
    Ok(format!("{:x}", hash.finalize()))
}

fn fixture_hash(root: &Path, bytes: bool) -> Result<String> {
    Ok(format!(
        "{}:{}",
        tree(&root.join("source"), bytes, true)?,
        tree(&root.join("control-target"), bytes, true)?
    ))
}

fn clone_tree(source: &Path, target: &Path) -> Result<()> {
    if target.try_exists()? || fs::symlink_metadata(target).is_ok() {
        return Err("clone destination must be new".into());
    }
    let output = Command::new("cp").args(["-a", "--reflink=always", "--"]).arg(source).arg(target).output()?;
    if !output.status.success() {
        return Err(format!("required reflink failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(())
}

fn setup(root: &Path, source: Option<&Path>) -> Result<()> {
    let start = Instant::now();
    fs::create_dir(root)?;
    let root = fs::canonicalize(root)?;
    fs::write(root.join(".owned"), MARKER)?;
    let origin = source.map(fs::canonicalize).transpose()?;
    if let Some(source) = &origin {
        if !fs::symlink_metadata(source)?.is_dir() || root.starts_with(source) {
            return Err("source must be a separate real directory".into());
        }
        clone_tree(source, &root.join("source"))?;
    } else {
        for directory in ["a", "b"] {
            let path = root.join("source").join(directory);
            fs::create_dir_all(&path)?;
            for file in ["one", "two"] {
                fs::write(path.join(file), vec![b'x'; 4096])?;
            }
            std::os::unix::fs::symlink("one", path.join("link"))?;
        }
    }
    clone_tree(&root.join("source"), &root.join("control-target"))?;
    let setup_ns = start.elapsed().as_nanos();
    let verification = Instant::now();
    if tree(&root.join("source"), true, false)? != tree(&root.join("control-target"), true, false)? {
        return Err("initial payload/metadata parity failed".into());
    }
    let fixture = Fixture {
        root: root.clone(),
        full: fixture_hash(&root, true)?,
        metadata: fixture_hash(&root, false)?,
        setup_ns,
        verification_ns: verification.elapsed().as_nanos(),
        source_origin: origin,
    };
    write_json(&root.join("fixture.json"), &fixture)?;
    println!("{}", root.join("fixture.json").display());
    Ok(())
}

fn verify_fixture(root: &Path, fixture: &Fixture, bytes: bool) -> Result<()> {
    if fs::canonicalize(root)? != fixture.root || fs::read_to_string(root.join(".owned"))? != MARKER {
        return Err("fixture identity/ownership mismatch".into());
    }
    if fixture_hash(root, bytes)? != if bytes { &fixture.full } else { &fixture.metadata }.as_str() {
        return Err("fixture changed (including inode/mtime/mode/link identity)".into());
    }
    Ok(())
}

fn controls(tokens: &[String]) -> Result<Vec<Control>> {
    let mut result = Vec::new();
    for chunk in tokens.as_chunks::<5>().0 {
        if !chunk[0].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') || chunk[0].is_empty() || result.iter().any(|c: &Control| c.label == chunk[0]) {
            return Err("control labels must be unique simple identifiers".into());
        }
        if !["none", "noop", "admin"].contains(&chunk[3].as_str()) {
            return Err("checked-bench mode must be none/noop/admin".into());
        }
        let provenance_path = fs::canonicalize(&chunk[4])?;
        result.push(Control {
            label: chunk[0].clone(),
            binary: fs::canonicalize(&chunk[1])?,
            sha256: chunk[2].clone(),
            mode: chunk[3].clone(),
            provenance: serde_json::from_slice(&fs::read(&provenance_path)?)?,
            provenance_sha256: sha256(&provenance_path)?,
            provenance_path,
        });
    }
    if !tokens.len().is_multiple_of(5) || result.len() < 2 {
        return Err("at least two five-token controls required".into());
    }
    Ok(result)
}

fn frozen_controls(directory: &Path) -> Result<Vec<Control>> {
    let directory = fs::canonicalize(directory)?;
    let identity_path = directory.join("identity.json");
    let identity: serde_json::Value = serde_json::from_slice(&fs::read(&identity_path)?)?;
    let mut result = Vec::new();
    for (label, file, mode, revision, manifest, lock) in [
        (
            "baseline",
            "checked-bench-before",
            "none",
            "fd277144c51700ca70d068fc88f3ff72a0d64293",
            "before-Cargo.toml",
            "before-Cargo.lock",
        ),
        (
            "default",
            "checked-bench-after",
            "none",
            "243be8e2a28a796c61652161c9dd30c6b35fc5d0",
            "after-Cargo.toml",
            "after-Cargo.lock",
        ),
        (
            "noop",
            "checked-bench-after",
            "noop",
            "243be8e2a28a796c61652161c9dd30c6b35fc5d0",
            "after-Cargo.toml",
            "after-Cargo.lock",
        ),
        (
            "admin",
            "checked-bench-after",
            "admin",
            "243be8e2a28a796c61652161c9dd30c6b35fc5d0",
            "after-Cargo.toml",
            "after-Cargo.lock",
        ),
    ] {
        let sha = identity["artifacts"][file]["sha256"].as_str().ok_or("missing frozen artifact identity")?;
        let expected = if file == "checked-bench-before" {
            "ab4a8dfe029892efc9a00cc92815c5cb3143dcb609c0db13eb4f6d27fd18afc9"
        } else {
            "a6906bea63e9fcaf8ccbb498a83e30ded4f0d93db28b1895962d207768fee362"
        };
        if sha != expected {
            return Err("frozen identity differs from reviewed checkpoint".into());
        }
        let provenance = json!({"frozen_identity":identity,"runtime_source_revision":revision,
            "profile":"release default; no diagnostics", "library_features":["parallel"],
            "runner_checks_feature":file=="checked-bench-after", "compiler":identity["compiler"],
            "runner_source_sha256":sha256(&directory.join("source-bench.rs"))?,
            "manifest":fs::read_to_string(directory.join(manifest))?,
            "lock_sha256":sha256(&directory.join(lock))?,
            "provenance_note":fs::read_to_string(directory.join("README.md"))?});
        result.push(Control {
            label: label.into(),
            binary: directory.join(file),
            sha256: sha.into(),
            mode: mode.into(),
            provenance_path: identity_path.clone(),
            provenance,
            provenance_sha256: sha256(&identity_path)?,
        });
    }
    Ok(result)
}

// Small specified PRNG for reproducible ordering, not for security or statistics.
fn schedule(count: usize, blocks: usize, mut seed: u64) -> Result<Vec<Vec<usize>>> {
    if count < 2 || blocks < 2 || !blocks.is_multiple_of(2) {
        return Err("need >=2 variants and positive even blocks".into());
    }
    let mut result = Vec::new();
    for _ in 0..blocks / 2 {
        let mut order: Vec<_> = (0..count).collect();
        for i in (1..count).rev() {
            seed = seed.wrapping_add(0x9e3779b97f4a7c15);
            let mut x = seed;
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
            x ^= x >> 31;
            order.swap(i, (x % (i as u64 + 1)) as usize);
        }
        result.push(order.clone());
        order.reverse();
        result.push(order);
    }
    Ok(result)
}

fn clean_ns(stdout: &str, label: &str, mode: &str, threads: usize) -> Result<u128> {
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
        || fields[7..].iter().any(|field| *field != "0")
    {
        return Err("invalid clean identity/scenario or nonzero clean operations".into());
    }
    let ns = fields[6].parse()?;
    if ns == 0 {
        return Err("zero API time".into());
    }
    Ok(ns)
}

fn verify_binary(control: &Control) -> Result<()> {
    if sha256(&control.binary)? != control.sha256 {
        return Err("control binary SHA-256 mismatch".into());
    }
    Ok(())
}

fn run(root: &Path, output: &Path, controls: &[Control], blocks: usize, seed: u64, threads: usize) -> Result<bool> {
    fs::create_dir_all("/tmp/opencode")?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open("/tmp/opencode/matched-performance.lock")?;
    lock.try_lock()?;
    let order = schedule(controls.len(), blocks, seed)?;
    let root = fs::canonicalize(root)?;
    let fixture: Fixture = serde_json::from_slice(&fs::read(root.join("fixture.json"))?)?;
    fs::create_dir(output)?;
    let output = fs::canonicalize(output)?;
    let preservation_start = Instant::now();
    let mut controls = controls.to_vec();
    fs::create_dir(output.join("artifacts"))?;
    for control in &mut controls {
        // Bad imports remain failed attempts; never preserve them under the expected hash.
        if verify_binary(control).is_ok() {
            let destination = output.join("artifacts").join(&control.label);
            fs::copy(&control.binary, &destination)?;
            fs::set_permissions(&destination, fs::Permissions::from_mode(0o555))?;
            control.binary = destination;
            verify_binary(control)?;
        }
    }
    let artifact_preservation_ns = preservation_start.elapsed().as_nanos();
    let initial = Instant::now();
    let initial_check = verify_fixture(&root, &fixture, true).map_err(|e| e.to_string());
    write_json(
        &output.join("campaign.json"),
        &json!({"schema":1,"fixture":fixture,"controls":controls,
        "seed":seed,"blocks":order,"threads":threads,"compiler":env!("BUILD_COMPILER"),
        "artifact_preservation_ns":artifact_preservation_ns,
        "profile":env!("BUILD_PROFILE"),"runner_sha256":sha256(&std::env::current_exe()?)?,
        "runner_source_sha256":format!("{:x}", Sha256::digest(include_bytes!("main.rs"))),
        "runner_manifest":include_str!("../Cargo.toml"),
        "runner_build_source":include_str!("../build.rs"),
        "initial_verification_ns":initial.elapsed().as_nanos(),"initial_error":initial_check.as_ref().err(),
        "scope":"API timer from checked-bench; process_ns includes startup, pool, untimed initialization; no cache dropping",
        "sampling":"seeded mirrored blocks; no warmup samples; each child initializes/warms outside its API timer",
        "build":"Cargo/build and fixture setup are separate; imported provenance is explicit operator evidence"}),
    )?;
    initial_check.map_err(|e| -> Box<dyn Error> { e.into() })?;
    let mut raw = OpenOptions::new().write(true).create_new(true).open(output.join("samples.jsonl"))?;
    let mut samples = Vec::new();
    for (block, indices) in order.iter().enumerate() {
        for (position, &index) in indices.iter().enumerate() {
            let control = &controls[index];
            let sequence = samples.len();
            let stem = output.join(format!("sample-{sequence:04}-{}", control.label));
            let argv = vec![
                "clean-repeat".to_owned(),
                root.display().to_string(),
                control.label.clone(),
                control.mode.clone(),
                threads.to_string(),
                "reflink".to_owned(),
                "1".to_owned(),
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
                let prepare = Instant::now();
                // All checks remain outside the child API timer.
                let prepared = verify_binary(control).and_then(|()| verify_fixture(&root, &fixture, false));
                sample.preparation_ns = prepare.elapsed().as_nanos();
                prepared?;
                let stdout = OpenOptions::new().write(true).create_new(true).open(stem.with_extension("stdout"))?;
                let stderr = OpenOptions::new().write(true).create_new(true).open(stem.with_extension("stderr"))?;
                let start = Instant::now();
                let mut child = Command::new(&control.binary)
                    .args(&sample.argv)
                    .stdin(Stdio::null())
                    .stdout(stdout)
                    .stderr(stderr)
                    .spawn()?;
                // Polling affects process_ns, never the child-owned API timer.
                let status = loop {
                    if let Some(status) = child.try_wait()? {
                        break Ok(status);
                    }
                    if start.elapsed().as_secs() >= 1800 {
                        child.kill()?;
                        child.wait()?;
                        break Err("native control timed out after 1800 seconds");
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                };
                sample.process_ns = start.elapsed().as_nanos();
                let status = status?;
                sample.exit_code = status.code();
                if !status.success() {
                    return Err(format!("child failed: {status}").into());
                }
                let verification = Instant::now();
                let checked = (|| -> Result<u128> {
                    let ns = clean_ns(&fs::read_to_string(stem.with_extension("stdout"))?, &control.label, &control.mode, threads)?;
                    verify_fixture(&root, &fixture, false)?;
                    Ok(ns)
                })();
                sample.verification_ns = verification.elapsed().as_nanos();
                sample.api_ns = Some(checked?);
                Ok(())
            })();
            if let Err(error) = attempt {
                sample.error = Some(error.to_string());
                sample.api_ns = None;
            }
            serde_json::to_writer(&mut raw, &sample)?;
            raw.write_all(b"\n")?;
            raw.flush()?;
            raw.sync_data()?;
            samples.push(sample);
        }
    }
    let final_start = Instant::now();
    let final_check = verify_fixture(&root, &fixture, true).map_err(|e| e.to_string());
    let passed = final_check.is_ok() && samples.iter().all(|s| s.error.is_none());
    write_json(
        &output.join("summary.json"),
        &json!({"passed":passed,
        "final_verification_ns":final_start.elapsed().as_nanos(),"final_error":final_check.err(),
        "pairs":pairs(&samples, &controls),"paired_spreads":spreads(&pairs(&samples, &controls)),
        "failed_samples":samples.iter().filter(|s|s.error.is_some()).count(),
        "interpretation":"plumbing evidence only; raw paired ratios, no win/significance/load correction; failures excluded"}),
    )?;
    println!("{}", output.join("summary.json").display());
    Ok(passed)
}

fn pairs(samples: &[Sample], controls: &[Control]) -> Vec<serde_json::Value> {
    let baseline = &controls[0].label;
    let mut result = Vec::new();
    for before in samples.iter().filter(|s| &s.label == baseline && s.error.is_none()) {
        for after in samples.iter().filter(|s| s.block == before.block && &s.label != baseline && s.error.is_none()) {
            if let (Some(old), Some(new)) = (before.api_ns, after.api_ns) {
                result.push(json!({"block":before.block,"baseline_sample":before.sequence,"candidate_sample":after.sequence,
                    "candidate":after.label,"baseline_api_ns":old,"candidate_api_ns":new,
                    "relative_percent":(new as f64 / old as f64 - 1.0)*100.0}));
            }
        }
    }
    result
}

fn spreads(pairs: &[serde_json::Value]) -> std::collections::BTreeMap<String, serde_json::Value> {
    let mut grouped = std::collections::BTreeMap::<String, Vec<f64>>::new();
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
                json!({"matched_blocks":n,"median_relative_percent":median,
            "min_relative_percent":values[0],"max_relative_percent":values[n-1]}),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> tempfile::TempDir {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/runner-tests");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("README.md"),
            "Tiny private Rust runner tests; required reflinks need the checkout filesystem. Session ses_eb2165cc8ffe4vOmOvDBe7aXQu.\n",
        )
        .unwrap();
        tempfile::tempdir_in(root).unwrap()
    }

    #[test]
    fn mirrored_seed_replays_and_balances_every_pair() {
        let order = schedule(4, 8, 1701).unwrap();
        assert_eq!(order, schedule(4, 8, 1701).unwrap());
        assert_ne!(order, schedule(4, 8, 1702).unwrap());
        for pair in order.as_chunks::<2>().0 {
            assert_eq!(pair[0].iter().rev().copied().collect::<Vec<_>>(), pair[1]);
            for variant in 0..4 {
                assert_eq!(
                    pair[0].iter().position(|x| *x == variant).unwrap() + pair[1].iter().position(|x| *x == variant).unwrap(),
                    3
                );
            }
        }
        assert!(schedule(1, 8, 0).is_err());
        assert!(schedule(2, 7, 0).is_err());
    }

    #[test]
    fn parser_refuses_dirty_or_extra_rows() {
        let row = "default,none,0,ReflinkRequired,0,clean,123,0,0,0,0,0,0,0,0";
        assert_eq!(clean_ns(row, "default", "none", 0).unwrap(), 123);
        assert!(clean_ns(&format!("{row}\n{row}"), "default", "none", 0).is_err());
        assert!(clean_ns(&row.replace(",123,0,", ",123,1,"), "default", "none", 0).is_err());
        assert!(clean_ns(row, "other", "none", 0).is_err());
    }

    #[test]
    fn private_reset_is_equivalent_but_not_shared_and_detects_changes() {
        let scratch = scratch();
        let root = scratch.path().join("fixture");
        setup(&root, None).unwrap();
        let fixture: Fixture = serde_json::from_slice(&fs::read(root.join("fixture.json")).unwrap()).unwrap();
        verify_fixture(&root, &fixture, true).unwrap();
        assert_ne!(
            fs::metadata(root.join("source/a/one")).unwrap().ino(),
            fs::metadata(root.join("control-target/a/one")).unwrap().ino()
        );
        fs::write(root.join("control-target/a/one"), b"changed").unwrap();
        assert!(verify_fixture(&root, &fixture, true).is_err());
        assert_eq!(fs::read(root.join("source/a/one")).unwrap(), vec![b'x'; 4096]);
        assert!(setup(&root, None).is_err());
    }

    #[test]
    fn artifact_corruption_is_retained_and_not_paired() {
        let scratch = scratch();
        let root = scratch.path().join("fixture");
        setup(&root, None).unwrap();
        let binary = scratch.path().join("control");
        // Deliberately never executed: valid hash is captured, then bytes change.
        fs::write(&binary, b"original").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        let control = Control {
            label: "baseline".into(),
            binary: binary.clone(),
            sha256: sha256(&binary).unwrap(),
            mode: "none".into(),
            provenance_path: binary.clone(),
            provenance: json!({}),
            provenance_sha256: "test".into(),
        };
        fs::write(binary, b"corrupt").unwrap();
        let mut other = control.clone();
        other.label = "candidate".into();
        let output = scratch.path().join("run");
        assert!(!run(&root, &output, &[control, other], 2, 0, 0).unwrap());
        let raw = fs::read_to_string(output.join("samples.jsonl")).unwrap();
        assert_eq!(raw.lines().count(), 4);
        assert!(raw.lines().all(|line| line.contains("SHA-256 mismatch")));
        let summary: serde_json::Value = serde_json::from_slice(&fs::read(output.join("summary.json")).unwrap()).unwrap();
        assert_eq!(summary["failed_samples"], 4);
        assert_eq!(summary["pairs"], json!([]));
    }
}
