//! Owned per-attempt reset and independent payload verification, shared by instruments.
use crate::{
    Result,
    fixture::{self, Fixture},
    write_json,
};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, FileTimes, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Scenario {
    Init,
    Clean,
    Sparse,
    Subtree,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Case {
    pub root: PathBuf,
    pub scenario: Scenario,
    pub destinations: usize,
    pub source_identity: String,
    pub starting_targets: Vec<Option<String>>,
    pub starting_equivalence: Vec<Option<String>>,
}

impl Case {
    pub fn source(&self) -> PathBuf {
        self.root.join("source")
    }
    pub fn target(&self, index: usize) -> PathBuf {
        self.root.join(format!("target-{index}"))
    }

    pub fn load(root: &Path) -> Result<Self> {
        let case: Self = serde_json::from_slice(&fs::read(root.join("case.json"))?)?;
        if case.root != fs::canonicalize(root)?
            || fs::read_to_string(root.join(".scenario-owned"))? != "owned scenario v1\n"
            || case.destinations == 0
            || case.starting_targets.len() != case.destinations
            || case.starting_equivalence.len() != case.destinations
        {
            return Err("invalid owned scenario case".into());
        }
        case.validate_start()?;
        Ok(case)
    }

    pub fn validate_start(&self) -> Result<()> {
        if !fs::symlink_metadata(self.source())?.is_dir() || fixture::tree(&self.source(), false, true)? != self.source_identity {
            return Err("scenario source changed".into());
        }
        for index in 0..self.destinations {
            let actual = if fs::symlink_metadata(self.target(index)).is_ok() {
                Some(fixture::tree(&self.target(index), false, true)?)
            } else {
                None
            };
            if actual != self.starting_targets[index] {
                return Err("scenario starting target changed".into());
            }
        }
        Ok(())
    }

    pub fn verify(&self) -> Result<()> {
        if fixture::tree(&self.source(), false, true)? != self.source_identity {
            return Err("operation changed its source".into());
        }
        let expected = fixture::payload(&self.source())?;
        for index in 0..self.destinations {
            if !fs::symlink_metadata(self.target(index))?.is_dir() || fixture::payload(&self.target(index))? != expected {
                return Err(format!("payload bytes/link/mode/file-mtime parity failed at target {index}").into());
            }
            if self.scenario == Scenario::Clean && Some(fixture::tree(&self.target(index), false, true)?) != self.starting_targets[index] {
                return Err("clean target inode/mtime/mode/link identity changed".into());
            }
        }
        Ok(())
    }
}

pub fn prepare(fixture: &Fixture, root: &Path, scenario: Scenario, destinations: usize) -> Result<Case> {
    prepare_scoped(fixture, root, scenario, destinations, &[PathBuf::new()])
}

pub fn prepare_scoped(fixture: &Fixture, root: &Path, scenario: Scenario, destinations: usize, scopes: &[PathBuf]) -> Result<Case> {
    if destinations == 0 {
        return Err("destinations must be positive".into());
    }
    fixture::verify(&fixture.root, fixture, false)?;
    let planned = fixture::fresh_path(root)?;
    if planned.starts_with(&fixture.root) {
        return Err("scenario case must be outside immutable fixture".into());
    }
    fs::create_dir(root)?;
    let root = fs::canonicalize(root)?;
    fs::write(root.join(".scenario-owned"), "owned scenario v1\n")?;
    fixture::clone_tree(&fixture.root.join("source"), &root.join("source"))?;
    let mut case = Case {
        root,
        scenario,
        destinations,
        source_identity: String::new(),
        starting_targets: Vec::new(),
        starting_equivalence: Vec::new(),
    };
    for index in 0..destinations {
        let target = case.target(index);
        if scenario == Scenario::Init {
            case.starting_targets.push(None);
            case.starting_equivalence.push(None);
            continue;
        }
        fixture::clone_tree(&case.source(), &target)?;
        match scenario {
            Scenario::Sparse => {
                let (scope, file) = select_scoped(&target, scopes, false)?.ok_or("sparse fixture needs a regular payload file")?;
                // Size differs; no dependency on timestamp granularity or byte comparison.
                OpenOptions::new().append(true).open(&file)?.write_all(b"sparse target delta\n")?;
                File::open(&file)?.set_times(FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(1)))?;
                let stale = scope.join("__perf_stale");
                OpenOptions::new().write(true).create_new(true).open(&stale)?.write_all(b"stale\n")?;
                File::open(stale)?.set_times(FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(1)))?;
                normalize_dirs(&target)?;
            }
            Scenario::Subtree => {
                let (scope, removed) = select_scoped(&target, scopes, true)?.ok_or("subtree fixture needs a real payload child directory")?;
                fs::rename(removed, case.root.join(format!("removed-{index}")))?;
                let obsolete = scope.join("__perf_obsolete");
                fs::create_dir(&obsolete)?;
                fs::write(obsolete.join("leaf"), b"obsolete\n")?;
                File::open(obsolete.join("leaf"))?.set_times(FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(1)))?;
                normalize_dirs(&target)?;
            }
            _ => {}
        }
        case.starting_targets.push(Some(fixture::tree(&target, false, true)?));
        case.starting_equivalence.push(Some(fixture::tree(&target, true, false)?));
    }
    case.source_identity = fixture::tree(&case.source(), false, true)?;
    write_json(&case.root.join("case.json"), &case)?;
    Ok(case)
}

fn select_scoped(target: &Path, scopes: &[PathBuf], directory: bool) -> Result<Option<(PathBuf, PathBuf)>> {
    for relative in scopes {
        if relative.is_absolute() || relative.components().any(|p| !matches!(p, std::path::Component::Normal(_))) {
            return Err("unsafe scenario scope".into());
        }
        let scope = target.join(relative);
        if !fs::symlink_metadata(&scope)?.is_dir() {
            return Err("scenario scope must be a real directory".into());
        }
        if let Some(path) = select(&scope, directory)? {
            return Ok(Some((scope, path)));
        }
    }
    Ok(None)
}

fn select(root: &Path, directory: bool) -> Result<Option<PathBuf>> {
    let mut entries = fs::read_dir(root)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let kind = entry.file_type()?;
        if (directory && kind.is_dir()) || (!directory && kind.is_file()) {
            return Ok(Some(entry.path()));
        }
        if !directory
            && kind.is_dir()
            && let Some(path) = select(&entry.path(), false)?
        {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn normalize_dirs(root: &Path) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            normalize_dirs(&entry.path())?;
        }
    }
    File::open(root)?.set_times(FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(1)))?;
    Ok(())
}

#[test]
fn every_scenario_resets_equivalent_private_n2_starts() {
    use std::os::unix::fs::MetadataExt;
    let scratch = crate::scratch();
    let root = scratch.path().join("fixture");
    fixture::setup(&root, None).unwrap();
    let fixture: Fixture = serde_json::from_slice(&fs::read(root.join("fixture.json")).unwrap()).unwrap();
    for scenario in [Scenario::Init, Scenario::Clean, Scenario::Sparse, Scenario::Subtree] {
        let a = prepare(&fixture, &scratch.path().join(format!("{scenario:?}-a")), scenario, 2).unwrap();
        let b = prepare(&fixture, &scratch.path().join(format!("{scenario:?}-b")), scenario, 2).unwrap();
        assert_eq!(a.starting_equivalence, b.starting_equivalence);
        a.validate_start().unwrap();
        b.validate_start().unwrap();
        if scenario == Scenario::Clean {
            a.verify().unwrap();
            assert_ne!(
                fs::metadata(a.target(0).join("a/one")).unwrap().ino(),
                fs::metadata(a.target(1).join("a/one")).unwrap().ino()
            );
        } else {
            assert!(a.verify().is_err());
        }
        if scenario == Scenario::Init {
            assert!(!a.target(0).exists());
        }
    }
}
