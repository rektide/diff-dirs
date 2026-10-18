use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Control {
    pub label: String,
    pub binary: PathBuf,
    pub sha256: String,
    pub mode: String,
    pub provenance_path: PathBuf,
    pub provenance: serde_json::Value,
    pub provenance_sha256: String,
}

pub fn sha256(path: &Path) -> Result<String> {
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

pub fn verify(control: &Control) -> Result<()> {
    if sha256(&control.binary)? != control.sha256 {
        return Err("control binary SHA-256 mismatch".into());
    }
    Ok(())
}

pub fn preserve(controls: &[Control], output: &Path) -> Result<Vec<Control>> {
    fs::create_dir(output.join("artifacts"))?;
    let mut controls = controls.to_vec();
    for control in &mut controls {
        if verify(control).is_ok() {
            let destination = output.join("artifacts").join(&control.label);
            fs::copy(&control.binary, &destination)?;
            fs::set_permissions(&destination, fs::Permissions::from_mode(0o555))?;
            control.binary = destination;
            verify(control)?;
        }
    }
    Ok(controls)
}

pub fn controls(tokens: &[String]) -> Result<Vec<Control>> {
    let mut result = Vec::new();
    for chunk in tokens.as_chunks::<5>().0 {
        if chunk[0].is_empty() || !chunk[0].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') || result.iter().any(|c: &Control| c.label == chunk[0]) {
            return Err("control labels must be unique simple identifiers".into());
        }
        if !["none", "noop", "admin"].contains(&chunk[3].as_str()) {
            return Err("mode must be none/noop/admin".into());
        }
        let path = fs::canonicalize(&chunk[4])?;
        result.push(Control {
            label: chunk[0].clone(),
            binary: fs::canonicalize(&chunk[1])?,
            sha256: chunk[2].clone(),
            mode: chunk[3].clone(),
            provenance: serde_json::from_slice(&fs::read(&path)?)?,
            provenance_sha256: sha256(&path)?,
            provenance_path: path,
        });
    }
    if !tokens.len().is_multiple_of(5) || result.len() < 2 {
        return Err("at least two five-token controls required".into());
    }
    Ok(result)
}

pub fn frozen_controls(directory: &Path) -> Result<Vec<Control>> {
    let directory = fs::canonicalize(directory)?;
    let path = directory.join("identity.json");
    let identity: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
    let mut result = Vec::new();
    for (label, mode, before) in [
        ("baseline", "none", true),
        ("default", "none", false),
        ("noop", "noop", false),
        ("admin", "admin", false),
    ] {
        let (file, prefix, revision, expected) = if before {
            (
                "checked-bench-before",
                "before",
                "fd277144c51700ca70d068fc88f3ff72a0d64293",
                "ab4a8dfe029892efc9a00cc92815c5cb3143dcb609c0db13eb4f6d27fd18afc9",
            )
        } else {
            (
                "checked-bench-after",
                "after",
                "243be8e2a28a796c61652161c9dd30c6b35fc5d0",
                "a6906bea63e9fcaf8ccbb498a83e30ded4f0d93db28b1895962d207768fee362",
            )
        };
        if identity["artifacts"][file]["sha256"] != expected {
            return Err("frozen identity differs from reviewed checkpoint".into());
        }
        let provenance = json!({"frozen_identity":identity,"runtime_source_revision":revision,
            "profile":"release default; no diagnostics","library_features":["parallel"],"runner_checks_feature":!before,
            "compiler":identity["compiler"],"runner_source_sha256":sha256(&directory.join("source-bench.rs"))?,
            "manifest":fs::read_to_string(directory.join(format!("{prefix}-Cargo.toml")))?,
            "lock_sha256":sha256(&directory.join(format!("{prefix}-Cargo.lock")))?,
            "provenance_note":fs::read_to_string(directory.join("README.md"))?});
        result.push(Control {
            label: label.into(),
            binary: directory.join(file),
            sha256: expected.into(),
            mode: mode.into(),
            provenance_path: path.clone(),
            provenance,
            provenance_sha256: sha256(&path)?,
        });
    }
    Ok(result)
}
