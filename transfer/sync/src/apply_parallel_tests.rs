use crate::CopyPolicy;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

static HOOK: Mutex<Option<Arc<Hook>>> = Mutex::new(None);
#[derive(Default)]
struct State {
    workers: Vec<String>,
    deleted: bool,
    release: bool,
}
struct Hook {
    parent: PathBuf,
    delete: Option<PathBuf>,
    state: Mutex<State>,
    changed: Condvar,
}
pub(crate) fn observe(path: &Path) {
    let hook = HOOK.lock().unwrap().clone();
    let Some(hook) = hook.filter(|h| path.parent() == Some(h.parent.as_path())) else {
        return;
    };
    let mut state = hook.state.lock().unwrap();
    state.workers.push(std::thread::current().name().unwrap().to_string());
    hook.changed.notify_all();
    let (mut state, timeout) = hook.changed.wait_timeout_while(state, Duration::from_secs(5), |s| s.workers.len() < 2).unwrap();
    assert!(!timeout.timed_out(), "no distinct concurrent apply worker");
    if let Some(delete) = &hook.delete {
        if path.file_name().unwrap() == "a-fail" {
            fs::remove_file(delete).unwrap();
            state.deleted = true;
            hook.changed.notify_all();
        } else {
            let (_state, timeout) = hook.changed.wait_timeout_while(state, Duration::from_secs(5), |s| !s.release).unwrap();
            assert!(!timeout.timed_out(), "test did not release started sibling");
        }
    }
}
#[test]
fn actual_independent_apply_and_recursive_copy_workers_join_on_failure() {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/sync-tests");
    fs::create_dir_all(&scratch).unwrap();
    let fixture = tempfile::tempdir_in(fs::canonicalize(scratch).unwrap()).unwrap();
    let source = fixture.path().join("source");
    let target = fixture.path().join("target");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .thread_name(|i| format!("sync-owned-{i}"))
        .build()
        .unwrap();
    for nested in [false, true] {
        let input = if nested { source.join("opaque") } else { source.clone() };
        fs::create_dir_all(&input).unwrap();
        for name in ["a-fail", "b-work"] {
            fs::write(input.join(name), b"actual nonempty apply work").unwrap();
        }
        let hook = Arc::new(Hook {
            parent: if nested { target.join("opaque") } else { target.clone() },
            delete: None,
            state: Mutex::default(),
            changed: Condvar::new(),
        });
        *HOOK.lock().unwrap() = Some(hook.clone());
        crate::mirror_in_pool(
            &source,
            &target,
            CopyPolicy::Copy,
            &pool,
            diff_dirs_tree_compare::ParallelLimits::for_pool(&pool),
        )
        .unwrap();
        *HOOK.lock().unwrap() = None;
        let state = hook.state.lock().unwrap();
        assert_eq!(state.workers.len(), 2);
        assert_ne!(state.workers[0], state.workers[1]);
        assert!(state.workers.iter().all(|s| s.starts_with("sync-owned-")));
    }
    for name in ["a-fail", "b-work"] {
        fs::write(source.join(name), b"changed larger source payload").unwrap();
    }
    let hook = Arc::new(Hook {
        parent: target.clone(),
        delete: Some(source.join("a-fail")),
        state: Mutex::default(),
        changed: Condvar::new(),
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&target, fs::Permissions::from_mode(0o555)).unwrap();
    }
    *HOOK.lock().unwrap() = Some(hook.clone());
    std::thread::scope(|scope| {
        let (sender, receiver) = std::sync::mpsc::channel();
        let (source, target, pool) = (&source, &target, &pool);
        scope.spawn(move || {
            sender
                .send(crate::mirror_in_pool(
                    source,
                    target,
                    CopyPolicy::Copy,
                    pool,
                    diff_dirs_tree_compare::ParallelLimits::for_pool(pool),
                ))
                .unwrap();
        });
        let state = hook.state.lock().unwrap();
        let (mut state, timeout) = hook.changed.wait_timeout_while(state, Duration::from_secs(5), |s| !s.deleted).unwrap();
        assert!(!timeout.timed_out());
        assert!(
            receiver.recv_timeout(Duration::from_millis(50)).is_err(),
            "returned before started sibling joined"
        );
        state.release = true;
        hook.changed.notify_all();
        drop(state);
        let failure = receiver.recv_timeout(Duration::from_secs(5)).unwrap().unwrap_err();
        assert!(failure.primary.path.ends_with("a-fail"));
        assert_eq!(failure.completed.files_copied, 1);
        assert_eq!(fs::read(target.join("b-work")).unwrap(), b"changed larger source payload");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(target).unwrap().permissions().mode() & 0o777, 0o555);
            assert_eq!(failure.completed.directory_permissions_set, 2);
            fs::set_permissions(target, fs::Permissions::from_mode(0o700)).unwrap();
        }
    });
    *HOOK.lock().unwrap() = None;
    crate::mirror(&source, &target, CopyPolicy::Copy).unwrap();
    assert!(diff_dirs_tree_compare::walk(&source, &target).unwrap().next().is_none());
}
