use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use diff_dirs::{DirectoryPlan, NoVisitor, Purpose, inspect_directory};

use crate::{Attempt, Bindings, EffectOutcome, Error, Executor, Inspector, Node, Work, Writes, disjoint, report_name, require};

#[derive(Default)]
pub struct FullInspector;

impl Inspector for FullInspector {
    fn inspect(&mut self, bindings: &Bindings, node: &Node) -> Result<DirectoryPlan, Error> {
        let [source, target, _] = bindings.roots();
        let relative = node.relative.to_path_buf();
        let source_dir = source.join(&relative);
        let target_dir = target.join(&relative);
        directory_route(&source_dir)?;
        directory_route(&target_dir)?;
        Ok(inspect_directory(&source, &target, &relative, Some(&source_dir), Some(&target_dir), Purpose::Audit, &NoVisitor)?)
    }
}

#[derive(Default)]
pub struct ByteExecutor;

impl Executor for ByteExecutor {
    fn execute(&mut self, bindings: &Bindings, work: &Work) -> Attempt<EffectOutcome> {
        let mut writes = Writes::default();
        let result = execute(bindings, work, &mut writes);
        Attempt { result, writes }
    }
}

pub(crate) fn bind_roots(bindings: &Bindings) -> Result<(), Error> {
    let roots = bindings.roots();
    for root in &roots { directory_route(root)?; }
    let resolved: Vec<_> = roots.iter().map(|root| fs::canonicalize(root).map_err(|e| Error::io(root, "resolve root", e))).collect::<Result<_, _>>()?;
    disjoint(&[resolved[0].clone(), resolved[1].clone(), resolved[2].clone()])
}

/// Structural no-follow ancestor checks, not descendant enumeration. Caller owns
/// stable nonhostile topology; these checks are not a race-resistant sandbox.
fn directory_route(path: &Path) -> Result<(), Error> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        let metadata = fs::symlink_metadata(&current).map_err(|e| Error::io(&current, "check directory route", e))?;
        require(metadata.is_dir(), "route contains a link or non-directory")?;
    }
    Ok(())
}

fn regular_or_absent(path: &Path) -> Result<bool, Error> {
    directory_route(path.parent().ok_or_else(|| Error::invalid("missing file parent"))?)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => { require(metadata.is_file(), "output must be regular or absent")?; Ok(true) }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(Error::io(path, "check output", error)),
    }
}

fn execute(bindings: &Bindings, work: &Work, writes: &mut Writes) -> Result<EffectOutcome, Error> {
    let [source, target, reports] = bindings.roots();
    match work {
        Work::EnsureDirectory(node) => {
            node.validate()?;
            let relative = node.relative.to_path_buf();
            directory_route(&source.join(&relative))?;
            let path = target.join(&relative);
            directory_route(path.parent().ok_or_else(|| Error::invalid("missing directory parent"))?)?;
            match fs::symlink_metadata(&path) {
                Ok(metadata) => { require(metadata.is_dir(), "ensure encountered a shape collision")?; Ok(EffectOutcome::NoWrite) }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir(&path).map_err(|e| Error::io(&path, "create target directory", e))?;
                    writes.directories_created = 1;
                    Ok(EffectOutcome::Created)
                }
                Err(error) => Err(Error::io(&path, "inspect target directory", error)),
            }
        }
        Work::SettleSourceFile(node) => {
            node.validate()?;
            let relative = node.relative.to_path_buf();
            let input = source.join(&relative);
            require(regular_or_absent(&input)?, "source file disappeared")?;
            let bytes = fs::read(&input).map_err(|e| Error::io(&input, "read desired source bytes", e))?;
            settle(&target.join(relative), &bytes, writes)
        }
        Work::InstallReport(recipe) => {
            let basename = report_name(&recipe.relative)?;
            settle(&reports.join(basename), &recipe.render(), writes)
        }
    }
}

fn settle(path: &Path, bytes: &[u8], writes: &mut Writes) -> Result<EffectOutcome, Error> {
    if regular_or_absent(path)? && fs::read(path).map_err(|e| Error::io(path, "read existing desired output", e))? == bytes {
        return Ok(EffectOutcome::NoWrite);
    }
    let temporary = tempfile::NamedTempFile::new_in(path.parent().expect("checked parent")).map_err(|e| Error::io(path, "create private staging", e))?;
    let (mut file, staging) = temporary.into_parts();
    let result = file.write_all(bytes).and_then(|()| file.flush()).map_err(|e| Error::io(path, "write staging bytes", e));
    drop(file);
    let result = result.and_then(|()| regular_or_absent(path).map(|_| ()));
    publish_staging(staging, path, result)?;
    writes.file_installations = 1;
    Ok(EffectOutcome::Installed)
}

/// The file descriptor is already closed. Successful same-parent persistence
/// consumes the staging name; only a failed preparation/rename needs removal.
pub(crate) fn publish_staging(staging: tempfile::TempPath, path: &Path, prepared: Result<(), Error>) -> Result<(), Error> {
    let (staging, primary) = match prepared {
        Ok(()) => match staging.persist(path) {
            Ok(()) => return Ok(()),
            Err(error) => (error.path, Error::io(path, "publish staged bytes", error.error)),
        },
        Err(error) => (staging, error),
    };
    match staging.close() {
        Ok(()) => Err(primary),
        Err(error) => Err(Error::Cleanup { primary: Box::new(primary), cleanup: Box::new(Error::io(path, "remove owned staging", error)) }),
    }
}
