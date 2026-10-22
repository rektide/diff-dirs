#![cfg(unix)]

use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use diff_dirs::{CheckError, FactOrigin, FileKind, Names, NoCheck, Operations, Purpose, Visitor};
use diff_dirs_sync::{Applier, CopyPolicy, Progress};
use diff_dirs_transfer_sync as diff_dirs_sync;
use diff_dirs_tree_compare as diff_dirs;

struct Fixture {
    _owned: tempfile::TempDir,
    source: PathBuf,
    target: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/early");
        fs::create_dir_all(&scratch).unwrap();
        let owned = tempfile::tempdir_in(fs::canonicalize(scratch).unwrap()).unwrap();
        let source = owned.path().join("source");
        let target = owned.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();
        Self { _owned: owned, source, target }
    }
}
fn file(path: PathBuf) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"tiny required-reflink fixture").unwrap();
}

#[derive(Debug)]
struct Event {
    path: PathBuf,
    source: Option<FileKind>,
    target: Option<FileKind>,
    root: bool,
}
struct Inspect<'a> {
    target: &'a Path,
    early: &'a Mutex<Vec<Event>>,
    late: &'a Mutex<Vec<PathBuf>>,
}
impl Visitor for Inspect<'_> {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        let root = names.is_root();
        for name in names.entries_mut() {
            assert!(!name.relative_path().to_string_lossy().contains(".diff-dirs-sync-"));
            for facts in [name.source(), name.target()].into_iter().flatten() {
                assert_eq!(facts.origin(), if root { FactOrigin::RootMetadata } else { FactOrigin::Listing });
                assert!(facts.inode().is_some());
                if root {
                    assert_ne!(facts.kind(), FileKind::Unknown);
                }
            }
            self.early.lock().unwrap().push(Event {
                path: name.relative_path().to_path_buf(),
                source: name.source().map(|f| f.kind()),
                target: name.target().map(|f| f.kind()),
                root,
            });
            // No child observation is needed for .jj, regardless of d_type.
            // The type-dependent rule explicitly defers Unknown to late.
            if !root && (name.name() == ".jj" || (name.name() == "skip-type" && name.source().is_some_and(|f| f.kind() == FileKind::Symlink))) {
                name.preserve_tree();
            }
        }
        Ok(())
    }
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        if batch.purpose() != Purpose::Audit {
            assert_eq!(batch.target_root(), self.target);
        }
        for entry in batch.entries_mut() {
            assert!(!entry.relative_path().to_string_lossy().contains(".diff-dirs-sync-"));
            assert_ne!(entry.name(), ".jj");
            self.late.lock().unwrap().push(entry.relative_path().to_path_buf());
            if entry.name() == "link" {
                let facts = entry.source().or_else(|| entry.target()).unwrap();
                assert!(facts.metadata().is_symlink());
            }
            if entry.name() == "skip-type" {
                assert!(entry.source().unwrap().metadata().is_symlink());
                entry.preserve_tree();
            }
        }
        Ok(())
    }
}

#[test]
fn early_facts_share_the_direct_recursive_removal_staging_audit_and_preview_seam() {
    let modes: &[bool] = if cfg!(feature = "parallel") { &[false, true] } else { &[false] };
    for &parallel in modes {
        let f = Fixture::new();
        for path in ["direct", "added/deep/file", "paired/new", "stage/child", "flatten"] {
            file(f.source.join(path));
        }
        for path in ["paired/old", "stage", "flatten/deep/old", "removed/deep/old"] {
            file(f.target.join(path));
        }
        std::os::unix::fs::symlink("missing//./literal", f.source.join("link")).unwrap();
        std::os::unix::fs::symlink("unobserved", f.source.join("skip-type")).unwrap();
        let _sockets = [
            std::os::unix::net::UnixListener::bind(f.source.join(".jj")).unwrap(),
            std::os::unix::net::UnixListener::bind(f.target.join(".jj")).unwrap(),
            std::os::unix::net::UnixListener::bind(f.source.join("added/deep/.jj")).unwrap(),
        ];
        let early = Mutex::new(Vec::new());
        let late = Mutex::new(Vec::new());
        let binding = Applier::new(&f.source, &f.target, CopyPolicy::ReflinkRequired).unwrap().visitor(Inspect {
            target: &f.target,
            early: &early,
            late: &late,
        });
        let audit = binding.audit().unwrap();
        assert_eq!(
            audit.original.excluded,
            if early
                .lock()
                .unwrap()
                .iter()
                .any(|e| e.path == Path::new("skip-type") && e.source == Some(FileKind::Symlink))
            {
                3
            } else {
                2
            }
        );
        let preview = binding.preview().unwrap();
        assert_eq!(preview.completed, Progress::default());
        assert!(!f.target.join("direct").exists());
        assert!(f.target.join("stage").is_file());
        #[cfg(feature = "parallel")]
        let report = if parallel {
            let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
            binding.run_in_pool(&pool, diff_dirs::ParallelLimits::for_pool(&pool)).unwrap()
        } else {
            binding.run().unwrap()
        };
        #[cfg(not(feature = "parallel"))]
        let report = {
            let _ = parallel;
            binding.run().unwrap()
        };
        assert_eq!(report.planned, preview.planned);
        assert_eq!(report.completed.files_reflinked, 5);
        assert_eq!(report.completed.files_copied, 0);
        for path in ["direct", "added/deep/file", "paired/new", "stage/child", "flatten"] {
            assert!(f.target.join(path).is_file());
        }
        for path in ["paired/old", "removed", "added/deep/.jj", "skip-type"] {
            assert!(!f.target.join(path).exists());
        }
        assert_eq!(fs::read_link(f.target.join("link")).unwrap().as_os_str(), "missing//./literal");
        let events = early.lock().unwrap();
        assert!(
            events
                .iter()
                .any(|e| e.path == Path::new("stage/child") && e.source.is_some() && e.target.is_none())
        );
        assert!(
            events
                .iter()
                .any(|e| e.path == Path::new("removed/deep/old") && e.source.is_none() && e.target.is_some())
        );
        let stage = events
            .iter()
            .find(|e| e.path == Path::new("stage") && e.source.is_some() && e.target.is_some())
            .unwrap();
        assert!(matches!(stage.source, Some(FileKind::Directory | FileKind::Unknown)));
        assert!(matches!(stage.target, Some(FileKind::RegularFile | FileKind::Unknown)));
        drop(events);
        early.lock().unwrap().clear();
        diff_dirs_sync::clear_with_visitor(
            &f.target,
            Inspect {
                target: &f.target,
                early: &early,
                late: &late,
            },
        )
        .unwrap();
        assert!(fs::symlink_metadata(f.target.join(".jj")).unwrap().file_type().is_socket());
        let events = early.lock().unwrap();
        assert!(events.iter().any(|e| e.root));
        assert!(events.iter().all(|e| e.source.is_none() && e.target.is_some()));
    }
}

#[test]
fn mandatory_root_facts_and_source_only_hints_survive_initialization_and_root_staging() {
    for replace in [false, true] {
        let f = Fixture::new();
        fs::remove_dir(&f.target).unwrap();
        file(f.source.join("deep/file"));
        std::os::unix::fs::symlink("missing", f.source.join("deep/link")).unwrap();
        if replace {
            std::os::unix::fs::symlink("never-follow", &f.target).unwrap();
        }
        let early = Mutex::new(Vec::new());
        let late = Mutex::new(Vec::new());
        let binding = Applier::new_replacing_checked(&f.source, &f.target, CopyPolicy::ReflinkRequired, NoCheck)
            .unwrap()
            .visitor(Inspect {
                target: &f.target,
                early: &early,
                late: &late,
            });
        let preview = binding.preview().unwrap();
        let report = binding.run().unwrap();
        assert_eq!(report.planned, preview.planned);
        assert_eq!(report.completed.files_reflinked, 1);
        assert!(f.target.join("deep/file").is_file());
        let events = early.lock().unwrap();
        let root = events.iter().find(|e| e.root).unwrap();
        assert_eq!(root.source, Some(FileKind::Directory));
        assert_eq!(root.target, replace.then_some(FileKind::Symlink));
        assert!(events.iter().filter(|e| !e.root).all(|e| e.source.is_some() && e.target.is_none()));
    }
}
