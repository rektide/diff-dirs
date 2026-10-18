//! Explicit same-instrument builds from exact git archives; no generic build DSL.
use crate::{
    Result,
    artifacts::{self, Control},
    fixture, host_lock, write_json,
};
use serde_json::json;
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn command(command: &mut Command, stem: &Path) -> Result<String> {
    let output = command.output()?;
    fs::write(stem.with_extension("stdout"), &output.stdout)?;
    fs::write(stem.with_extension("stderr"), &output.stderr)?;
    if !output.status.success() {
        return Err(format!("build command failed; see {}.stderr", stem.display()).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

pub fn build(repository: &Path, revision: &str, output: &Path, checks: bool) -> Result<()> {
    let _lock = host_lock()?;
    if ![40, 64].contains(&revision.len()) || !revision.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("revision must be a full immutable git commit ID".into());
    }
    fs::create_dir(output)?;
    let output = fs::canonicalize(output)?;
    let repository = fs::canonicalize(repository)?;
    let actual = command(
        Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args(["rev-parse", "--verify", &format!("{revision}^{{commit}}")]),
        &output.join("revision"),
    )?;
    if actual.trim() != revision {
        return Err("source revision mismatch".into());
    }
    command(
        Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args(["archive", "--format=tar", "--output"])
            .arg(output.join("source.tar"))
            .arg(revision),
        &output.join("archive"),
    )?;
    fs::create_dir(output.join("source"))?;
    command(
        Command::new("tar")
            .arg("-xf")
            .arg(output.join("source.tar"))
            .arg("-C")
            .arg(output.join("source")),
        &output.join("extract"),
    )?;
    // Copy the development helper/instrument too: later editing the checkout cannot alter this build input.
    let tool = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::create_dir(output.join("helper"))?;
    fs::create_dir(output.join("helper/src"))?;
    for file in ["Cargo.toml", "build.rs"] {
        fs::copy(tool.join(file), output.join("helper").join(file))?;
    }
    for entry in fs::read_dir(tool.join("src"))? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            fs::copy(entry.path(), output.join("helper/src").join(entry.file_name()))?;
        }
    }
    fs::copy(tool.join("../operation/src/main.rs"), output.join("operation.rs"))?;
    let helper_before = fixture::tree(&output.join("helper"), true, false)?;
    let operation_sha = artifacts::sha256(&output.join("operation.rs"))?;
    let source_before = source_hash(&output.join("source"))?;
    let manifest = r#"[package]
name = "diff-dirs-perf-operation"
version = "0.0.0"
edition = "2024"
publish = false
[workspace]
members = ["helper"]
[workspace.package]
edition = "2024"
[dependencies]
diff-dirs = { path = "source", features = ["parallel"] }
diff-dirs-sync = { path = "source/sync", features = ["parallel"] }
diff-dirs-perf-runner = { path = "helper" }
rayon = "1.12"
serde_json = "1"
[features]
checks = []
[[bin]]
name = "diff-dirs-perf-operation"
path = "operation.rs"
"#;
    fs::write(output.join("Cargo.toml"), manifest)?;
    // Use one common lock seed for baseline/candidate, then retain the actual resolved lock.
    fs::copy(tool.join("../../Cargo.lock"), output.join("Cargo.lock"))?;
    let compiler = command(
        Command::new("rustc").args(["--version", "--verbose"]).current_dir(&output),
        &output.join("compiler"),
    )?;
    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(&output)
        .args(["build", "--release", "--bin", "diff-dirs-perf-operation", "--target-dir", "target"]);
    if checks {
        cargo.args(["--features", "checks"]);
    }
    let argv = format!("{cargo:?}");
    command(&mut cargo, &output.join("cargo-build"))?;
    if fixture::tree(&output.join("helper"), true, false)? != helper_before
        || source_hash(&output.join("source"))? != source_before
        || artifacts::sha256(&output.join("operation.rs"))? != operation_sha
    {
        return Err("build inputs changed during build".into());
    }
    let binary = output.join("operation");
    fs::copy(output.join("target/release/diff-dirs-perf-operation"), &binary)?;
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o555))?;
    let provenance_path = output.join("provenance.json");
    let provenance = json!({"source_revision":revision,"repository":repository,"source_archive_sha256":artifacts::sha256(&output.join("source.tar"))?,
        "source_inputs":source_before,"helper_inputs":helper_before,"instrument_source_sha256":operation_sha,
        "compiler":compiler,"profile":"release default","library_features":["parallel"],"checks":checks,"build_argv":argv,
        "manifest":manifest,"resolved_lock_sha256":artifacts::sha256(&output.join("Cargo.lock"))?,"binary_sha256":artifacts::sha256(&binary)?});
    write_json(&provenance_path, &provenance)?;
    let control = Control {
        label: "artifact".into(),
        sha256: artifacts::sha256(&binary)?,
        binary,
        mode: "none".into(),
        provenance_path: provenance_path.clone(),
        provenance,
        provenance_sha256: artifacts::sha256(&provenance_path)?,
    };
    write_json(&output.join("control.json"), &control)?;
    println!("{}", output.join("control.json").display());
    Ok(())
}

fn source_hash(root: &Path) -> Result<serde_json::Value> {
    let mut hashes = serde_json::Map::new();
    for path in ["Cargo.toml", "Cargo.lock", "src", "sync/Cargo.toml", "sync/src"] {
        let full = root.join(path);
        hashes.insert(
            path.into(),
            json!(if full.is_dir() {
                fixture::tree(&full, true, false)?
            } else {
                artifacts::sha256(&full)?
            }),
        );
    }
    Ok(hashes.into())
}

pub fn controls(builds: &[PathBuf]) -> Result<Vec<Control>> {
    if builds.len() != 2 {
        return Err("exactly two build directories required".into());
    }
    let before: Control = serde_json::from_slice(&fs::read(builds[0].join("control.json"))?)?;
    let after: Control = serde_json::from_slice(&fs::read(builds[1].join("control.json"))?)?;
    if before.provenance["checks"] != false
        || after.provenance["checks"] != true
        || before.provenance["compiler"] != after.provenance["compiler"]
        || before.provenance["instrument_source_sha256"] != after.provenance["instrument_source_sha256"]
    {
        return Err("builds need unchecked baseline, checked candidate and matching compiler/instrument".into());
    }
    let mut result = Vec::new();
    for (label, mode, base) in [
        ("baseline", "none", &before),
        ("default", "none", &after),
        ("noop", "noop", &after),
        ("admin", "admin", &after),
    ] {
        let mut c = base.clone();
        c.label = label.into();
        c.mode = mode.into();
        result.push(c);
    }
    Ok(result)
}
