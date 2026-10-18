use crate::{Result, artifacts::sha256, write_json};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub const MARKER: &str = "checked-streaming owned fixture v1\n";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fixture {
    pub root: PathBuf,
    pub full: String,
    pub metadata: String,
    pub setup_ns: u128,
    pub verification_ns: u128,
    pub source_origin: Option<PathBuf>,
}

/// Sorted streaming fingerprint, never a retained full-tree map; no link following.
pub fn tree(root: &Path, bytes: bool, inodes: bool) -> Result<String> {
    tree_policy(root, bytes, inodes, false)
}

/// Payload contract: file mtime, bytes, mode, literal links; no directory/link mtimes.
pub fn payload(root: &Path) -> Result<String> {
    tree_policy(root, true, false, true)
}

fn tree_policy(root: &Path, bytes: bool, inodes: bool, payload: bool) -> Result<String> {
    fn visit(root: &Path, path: &Path, bytes: bool, inodes: bool, payload: bool, hash: &mut Sha256) -> Result<()> {
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
            if !payload || meta.is_file() { meta.mtime() } else { 0 },
            if !payload || meta.is_file() { meta.mtime_nsec() } else { 0 },
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
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                if entry.file_name() == ".git" || entry.file_name() == ".jj" {
                    return Err("administrative fixture entry refused".into());
                }
                visit(root, &entry.path(), bytes, inodes, payload, hash)?;
            }
        } else if bytes {
            hash.update(sha256(path)?.as_bytes());
        }
        Ok(())
    }
    let mut hash = Sha256::new();
    visit(root, root, bytes, inodes, payload, &mut hash)?;
    Ok(format!("{:x}", hash.finalize()))
}

pub fn fingerprint(root: &Path, bytes: bool) -> Result<String> {
    Ok(format!(
        "{}:{}",
        tree(&root.join("source"), bytes, true)?,
        tree(&root.join("control-target"), bytes, true)?
    ))
}

pub fn clone_tree(source: &Path, target: &Path) -> Result<()> {
    if target.try_exists()? || fs::symlink_metadata(target).is_ok() {
        return Err("clone destination must be new".into());
    }
    let output = Command::new("cp").args(["-a", "--reflink=always", "--"]).arg(source).arg(target).output()?;
    if !output.status.success() {
        return Err(format!("required reflink failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(())
}

pub fn setup(root: &Path, source: Option<&Path>) -> Result<()> {
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
    let verify = Instant::now();
    if tree(&root.join("source"), true, false)? != tree(&root.join("control-target"), true, false)? {
        return Err("initial payload/metadata parity failed".into());
    }
    let fixture = Fixture {
        root: root.clone(),
        full: fingerprint(&root, true)?,
        metadata: fingerprint(&root, false)?,
        setup_ns,
        verification_ns: verify.elapsed().as_nanos(),
        source_origin: origin,
    };
    write_json(&root.join("fixture.json"), &fixture)?;
    println!("{}", root.join("fixture.json").display());
    Ok(())
}

pub fn verify(root: &Path, fixture: &Fixture, bytes: bool) -> Result<()> {
    if fs::canonicalize(root)? != fixture.root || fs::read_to_string(root.join(".owned"))? != MARKER {
        return Err("fixture identity/ownership mismatch".into());
    }
    if fingerprint(root, bytes)? != if bytes { &fixture.full } else { &fixture.metadata }.as_str() {
        return Err("fixture changed (including inode/mtime/mode/link identity)".into());
    }
    Ok(())
}

#[test]
fn private_reset_is_equivalent_but_not_shared_and_detects_changes() {
    let scratch = crate::scratch();
    let root = scratch.path().join("fixture");
    setup(&root, None).unwrap();
    let fixture: Fixture = serde_json::from_slice(&fs::read(root.join("fixture.json")).unwrap()).unwrap();
    verify(&root, &fixture, true).unwrap();
    assert_ne!(
        fs::metadata(root.join("source/a/one")).unwrap().ino(),
        fs::metadata(root.join("control-target/a/one")).unwrap().ino()
    );
    fs::write(root.join("control-target/a/one"), b"changed").unwrap();
    assert!(verify(&root, &fixture, true).is_err());
    assert_eq!(fs::read(root.join("source/a/one")).unwrap(), vec![b'x'; 4096]);
    assert!(setup(&root, None).is_err());
}
