//! Development-only measurement primitives, shared by domain-specific instruments.
pub mod artifacts;
pub mod build_operation;
pub mod execution;
pub mod fixture;
pub mod process;
pub mod report;
pub mod scenario;
pub mod scheduling;

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}

pub fn source_identity() -> serde_json::Value {
    let mut files = std::collections::BTreeMap::new();
    for (name, source) in [
        ("main.rs", include_str!("main.rs")),
        ("lib.rs", include_str!("lib.rs")),
        ("artifacts.rs", include_str!("artifacts.rs")),
        ("fixture.rs", include_str!("fixture.rs")),
        ("scheduling.rs", include_str!("scheduling.rs")),
        ("execution.rs", include_str!("execution.rs")),
        ("report.rs", include_str!("report.rs")),
        ("process.rs", include_str!("process.rs")),
        ("scenario.rs", include_str!("scenario.rs")),
        ("build_operation.rs", include_str!("build_operation.rs")),
    ] {
        files.insert(name, format!("{:x}", Sha256::digest(source.as_bytes())));
    }
    serde_json::json!({"source_hashes":files,"compiler":env!("BUILD_COMPILER"),
        "profile":env!("BUILD_PROFILE"),"manifest":include_str!("../Cargo.toml"),
        "build_source":include_str!("../build.rs")})
}

pub fn host_lock() -> Result<std::fs::File> {
    #[cfg(not(test))]
    let path = std::path::PathBuf::from("/tmp/opencode/matched-performance.lock");
    #[cfg(test)]
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/runner-tests/unit.lock");
    std::fs::create_dir_all("/tmp/opencode")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path)?;
    lock.try_lock()?;
    Ok(lock)
}

#[cfg(test)]
pub(crate) fn scratch() -> tempfile::TempDir {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/runner-tests");
    std::fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}
