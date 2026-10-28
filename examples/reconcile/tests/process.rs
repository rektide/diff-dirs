use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use diff_dirs_examples_reconcile::{native::NativeString, transport};
use diff_dirs_transfer_sync::coordinated::{ApplicationState, End};
use serde_json::Value;

struct Fixture {
    _owned: tempfile::TempDir,
    source: PathBuf,
    a: PathBuf,
    b: PathBuf,
    cursor: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let owned = tempfile::tempdir_in("/tmp/opencode").unwrap();
        let source = owned.path().join("source");
        let a = owned.path().join("a");
        let b = owned.path().join("b");
        for p in [&source, &a, &b] {
            fs::create_dir(p).unwrap();
        }
        let cursor = owned.path().join("cursor.json");
        Self {
            _owned: owned,
            source,
            a,
            b,
            cursor,
        }
    }
    fn paired(&self, path: &Path) {
        for p in [&self.source, &self.a, &self.b] {
            fs::create_dir_all(p.join(path)).unwrap();
        }
    }
    fn start_args(&self, groups: u64, applications: u64) -> Vec<OsString> {
        vec![
            "start".into(),
            "--source".into(),
            self.source.clone().into(),
            "--target".into(),
            assignment("A", &self.a),
            "--target".into(),
            assignment("B", &self.b),
            "--copy-policy".into(),
            "copy".into(),
            "--cursor".into(),
            self.cursor.clone().into(),
            "--groups".into(),
            groups.to_string().into(),
            "--applications".into(),
            applications.to_string().into(),
        ]
    }
    fn resume_args(&self, groups: u64, applications: u64) -> Vec<OsString> {
        vec![
            "resume".into(),
            "--cursor".into(),
            self.cursor.clone().into(),
            "--groups".into(),
            groups.to_string().into(),
            "--applications".into(),
            applications.to_string().into(),
        ]
    }
    fn invoke(&self, args: Vec<OsString>, code: i32) -> Value {
        let out = invoke(args);
        assert_eq!(
            out.status.code(),
            Some(code),
            "stdout={} stderr={}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    }
    fn parts(&self) -> diff_dirs_transfer_sync::coordinated::SyncParts {
        transport::read(&self.cursor).unwrap().0
    }
}
#[cfg(unix)]
impl Drop for Fixture {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        fn writable(path: &Path) {
            if fs::symlink_metadata(path).is_ok_and(|m| m.is_dir()) {
                fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
                for e in fs::read_dir(path).unwrap() {
                    writable(&e.unwrap().path());
                }
            }
        }
        writable(self._owned.path());
    }
}
fn assignment(id: &str, path: &Path) -> OsString {
    let mut value = OsString::from(format!("{id}="));
    value.push(path);
    value
}
fn invoke(args: Vec<OsString>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_reconcile")).args(args).output().unwrap()
}
fn file(path: impl AsRef<Path>, bytes: &[u8]) {
    let p = path.as_ref();
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, bytes).unwrap();
}
fn no_staging(path: &Path) {
    for e in fs::read_dir(path).unwrap() {
        let e = e.unwrap();
        assert!(!e.file_name().as_encoded_bytes().starts_with(b".diff-dirs-sync-"));
        if e.file_type().unwrap().is_dir() {
            no_staging(&e.path());
        }
    }
}
fn raw_parts(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn fresh_processes_preserve_barrier_different_heads_returns_and_root_closure() {
    let f = Fixture::new();
    f.paired(Path::new("a/deep"));
    f.paired(Path::new("b"));
    file(f.source.join("root"), b"root");
    file(f.source.join("a/deep/leaf"), b"A only");
    #[cfg(unix)]
    let native_leaf = {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(b"leaf\xff".to_vec())
    };
    #[cfg(not(unix))]
    let native_leaf = OsString::from("leaf");
    file(f.source.join("b").join(&native_leaf), b"shared");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&f.a, fs::Permissions::from_mode(0o500)).unwrap();
    }
    let mut args = f.start_args(1, 1);
    args.extend(["--preserve".into(), assignment("B", Path::new("a"))]);
    let first = f.invoke(args, 0);
    assert_eq!(first["root_hooks"], 2);
    assert_eq!(first["stop"], "Budget");
    let p = f.parts();
    assert_eq!(p.comparison.frames.len(), 1);
    let apps = &p.current.unwrap().applications;
    assert_eq!(apps[0].state, ApplicationState::Accepted);
    assert_eq!(apps[1].state, ApplicationState::Unstarted);
    assert!(f.a.join("root").exists());
    assert!(!f.b.join("root").exists());
    #[cfg(unix)]
    let inode = {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        assert_eq!(fs::metadata(&f.a).unwrap().permissions().mode() & 0o777, 0o500);
        fs::metadata(f.a.join("root")).unwrap().ino()
    };
    no_staging(&f.a);
    no_staging(&f.b);

    // Settle B's root, then A's clean a/ batch; inspect a/deep but leave its
    // application pending. B is still waiting at root for shared b/.
    let second = f.invoke(f.resume_args(2, 2), 0);
    assert_eq!(second["root_hooks"], 0);
    let p = f.parts();
    assert_eq!(p.comparison.frames.len(), 3);
    assert_eq!(p.comparison.frames[2].relative, Path::new("a/deep"));
    assert_eq!(p.comparison.frames[0].remaining[1].name, "b");
    assert_eq!(p.current.unwrap().applications[0].state, ApplicationState::Unstarted);
    assert!(!f.a.join("a/deep/leaf").exists());
    assert!(f.b.join("root").exists());

    let mut args = f.resume_args(0, 1);
    args.extend(["--apply-only".into(), "A".into()]);
    let third = f.invoke(args, 0);
    assert_eq!(third["root_hooks"], 0);
    let p = f.parts();
    assert_eq!(p.comparison.frames.len(), 3);
    assert_eq!(p.current.unwrap().applications[0].state, ApplicationState::Accepted);
    assert_eq!(fs::read(f.a.join("a/deep/leaf")).unwrap(), b"A only");
    assert!(!f.b.join("a/deep/leaf").exists());

    // Pure returns, no next observation: now only root remains, with b pending.
    let fourth = f.invoke(f.resume_args(0, 0), 0);
    assert_eq!(fourth["groups"], 0);
    assert_eq!(fourth["root_hooks"], 0);
    let p = f.parts();
    assert_eq!(p.comparison.frames.len(), 1);
    assert!(p.current.is_none());
    assert_eq!(p.comparison.frames[0].remaining[0].name, "b");

    let fifth = f.invoke(f.resume_args(1, 1), 0);
    assert_eq!(fifth["root_hooks"], 0);
    let p = f.parts();
    assert_eq!(p.comparison.frames.len(), 2);
    assert_eq!(p.comparison.frames[1].relative, Path::new("b"));
    let current = p.current.unwrap();
    assert_eq!(current.applications[0].state, ApplicationState::Accepted);
    assert_eq!(current.applications[1].state, ApplicationState::Unstarted);
    assert_eq!(current.applications[1].decision.entries[0].name, native_leaf);
    assert!(!f.b.join("b").join(&native_leaf).exists());

    // Accept B but deliberately save before child/root closure.
    let mut args = f.resume_args(0, 1);
    args.extend(["--apply-only".into(), "B".into()]);
    let sixth = f.invoke(args, 0);
    assert_eq!(sixth["root_hooks"], 0);
    let p = f.parts();
    assert_eq!(p.comparison.frames.len(), 2);
    assert!(p.current.unwrap().applications.iter().all(|a| a.state == ApplicationState::Accepted));
    no_staging(&f.a);
    no_staging(&f.b);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        assert_eq!(fs::metadata(&f.a).unwrap().permissions().mode() & 0o777, 0o500);
        assert_eq!(fs::metadata(f.a.join("root")).unwrap().ino(), inode);
    }

    let seventh = f.invoke(f.resume_args(0, 0), 0);
    assert_eq!(seventh["stop"], "Done");
    assert_eq!(seventh["root_hooks"], 0);
    assert_eq!(seventh["reports"][0]["completed"]["files_copied"], 3);
    assert_eq!(seventh["reports"][1]["completed"]["files_copied"], 2);
    assert_eq!(seventh["reports"][0]["satisfied"], true);
    assert_eq!(seventh["reports"][1]["satisfied"], true);
    let terminal = f.parts();
    assert!(terminal.comparison.frames.is_empty());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&f.a, fs::Permissions::from_mode(0o700)).unwrap();
    }
    for root in [&f.source, &f.a, &f.b] {
        fs::remove_dir_all(root).unwrap();
    }
    let eighth = f.invoke(f.resume_args(0, 0), 0);
    assert_eq!(eighth["stop"], "Done");
    assert_eq!(eighth["root_hooks"], 0);
    assert_eq!(f.parts(), terminal);
}

#[test]
fn returned_opaque_failure_is_saved_once_and_failed_root_is_not_rebound() {
    let f = Fixture::new();
    file(f.source.join("opaque/bad"), b"refused");
    f.paired(Path::new("paired"));
    file(f.source.join("paired/leaf"), b"healthy");
    let mut args = f.start_args(1, 1);
    args.extend(["--refuse".into(), assignment("A", Path::new("opaque/bad"))]);
    let first = f.invoke(args, 2);
    assert_eq!(first["root_hooks"], 2);
    assert_eq!(first["failures"].as_array().unwrap().len(), 1);
    let p = f.parts();
    assert_eq!(p.lanes[0].end, Some(End::ApplicationFailed));
    assert_eq!(p.lanes[0].applications, 1);
    assert_eq!(p.lanes[0].completed.directories_created, 1);
    assert!(f.a.join("opaque").is_dir());
    assert!(!f.a.join("opaque/bad").exists());
    assert!(!f.b.join("opaque").exists());
    no_staging(&f.a);
    fs::remove_dir_all(&f.a).unwrap();
    let second = f.invoke(f.resume_args(100, 100), 2);
    assert_eq!(second["stop"], "Done");
    assert_eq!(second["root_hooks"], 0);
    assert_eq!(second["reports"][0]["satisfied"], false);
    assert_eq!(second["reports"][1]["satisfied"], true);
    assert_eq!(f.parts().lanes[0].completed.directories_created, 1);
    assert_eq!(f.parts().lanes[0].applications, 1);
    assert_eq!(fs::read(f.b.join("opaque/bad")).unwrap(), b"refused");
    assert_eq!(fs::read(f.b.join("paired/leaf")).unwrap(), b"healthy");
}

#[cfg(unix)]
#[test]
fn changed_kind_with_same_diff_refuses_before_any_resumed_effect() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    file(f.source.join("x"), b"source new payload");
    file(f.a.join("x"), b"old");
    f.invoke(f.start_args(1, 0), 0);
    fs::remove_file(f.a.join("x")).unwrap();
    symlink("old-text", f.a.join("x")).unwrap();
    let out = f.invoke(f.resume_args(0, 1), 2);
    assert_eq!(out["root_hooks"], 0);
    assert!(fs::symlink_metadata(f.a.join("x")).unwrap().is_symlink());
    assert!(!f.b.join("x").exists());
    let p = f.parts();
    assert_eq!(p.lanes[0].end, Some(End::DecisionChanged));
    assert_eq!(p.lanes[0].applications, 0);
    assert_eq!(p.lanes[1].applications, 0);
    f.invoke(f.resume_args(100, 100), 2);
    assert_eq!(fs::read(f.b.join("x")).unwrap(), b"source new payload");
}

#[test]
fn changed_early_excluded_name_set_is_not_hidden_by_equal_counts() {
    let f = Fixture::new();
    file(f.source.join("x"), b"new");
    file(f.a.join("x"), b"old long payload");
    file(f.source.join("hidden-old"), b"hidden");
    let mut args = f.start_args(1, 0);
    for name in ["hidden-old", "hidden-new"] {
        args.extend(["--preserve".into(), assignment("A", Path::new(name))]);
    }
    f.invoke(args, 0);
    fs::rename(f.source.join("hidden-old"), f.source.join("hidden-new")).unwrap();
    f.invoke(f.resume_args(0, 1), 2);
    assert_eq!(f.parts().lanes[0].end, Some(End::DecisionChanged));
    assert_eq!(fs::read(f.a.join("x")).unwrap(), b"old long payload");
    assert!(!f.b.join("x").exists());
}

#[test]
fn restored_controls_can_prune_accepted_a_and_abandon_unstarted_b() {
    let f = Fixture::new();
    f.paired(Path::new("d"));
    file(f.source.join("d/leaf"), b"not traversed");
    file(f.source.join("root"), b"A only");
    f.invoke(f.start_args(1, 1), 0);
    let mut args = f.resume_args(0, 0);
    args.extend(["--prune".into(), "A".into(), "--stop".into(), "B".into()]);
    let out = f.invoke(args, 2);
    assert_eq!(out["stop"], "Done");
    assert_eq!(out["root_hooks"], 0);
    let p = f.parts();
    assert!(p.lanes[0].pruned);
    assert_eq!(p.lanes[1].end, Some(End::Abandoned));
    assert!(!f.b.join("root").exists());
    assert!(!f.a.join("d/leaf").exists());
}

#[test]
fn malformed_transport_state_binding_and_descriptor_fail_before_effects() {
    let f = Fixture::new();
    file(f.source.join("file"), b"data");
    f.invoke(f.start_args(1, 0), 0);
    let original = raw_parts(&f.cursor);
    for case in 0..7 {
        let mut bad = original.clone();
        match case {
            0 => bad["version"] = 99.into(),
            1 => {
                bad["state"]["current"]["applications"].as_array_mut().unwrap().pop();
            }
            2 => bad["state"]["comparison"]["targets"][1]["root"] = bad["state"]["comparison"]["targets"][0]["root"].clone(),
            3 => bad["state"]["comparison"]["targets"][0]["policy"] = "foreign/v2".into(),
            4 => bad["state"]["current"]["applications"][0]["decision"]["entries"][0]["source"] = "imaginary-kind".into(),
            5 => bad["state"]["lanes"][0]["completed"]["files_copied"] = 1.into(),
            _ => bad["unknown_semantics"] = true.into(),
        }
        fs::write(&f.cursor, serde_json::to_vec(&bad).unwrap()).unwrap();
        let bytes = fs::read(&f.cursor).unwrap();
        let out = invoke(f.resume_args(100, 100));
        assert_eq!(out.status.code(), Some(1), "case {case}: {}", String::from_utf8_lossy(&out.stderr));
        assert!(out.stdout.is_empty());
        assert_eq!(fs::read(&f.cursor).unwrap(), bytes);
        assert!(!f.a.join("file").exists());
        assert!(!f.b.join("file").exists());
    }
}

#[test]
fn cursor_publication_inside_a_tree_is_refused_before_application() {
    let mut f = Fixture::new();
    file(f.source.join("file"), b"data");
    f.cursor = f.source.join("state.json");
    let out = invoke(f.start_args(100, 100));
    assert_eq!(out.status.code(), Some(1));
    assert!(!f.a.join("file").exists());
    assert!(!f.b.join("file").exists());
    assert!(!f.cursor.exists());
}

#[test]
fn malformed_explicit_application_subset_refuses_before_its_first_effect() {
    let f = Fixture::new();
    file(f.source.join("file"), b"data");
    f.invoke(f.start_args(1, 0), 0);
    let saved = fs::read(&f.cursor).unwrap();
    for second in ["A", "unknown"] {
        let mut args = f.resume_args(0, 2);
        args.extend(["--apply-only".into(), "A".into(), "--apply-only".into(), second.into()]);
        let output = invoke(args);
        assert_eq!(output.status.code(), Some(1));
        assert!(!f.a.join("file").exists());
        assert!(!f.b.join("file").exists());
        assert_eq!(fs::read(&f.cursor).unwrap(), saved);
    }
}

#[cfg(unix)]
#[test]
fn native_non_utf8_roots_rules_and_names_roundtrip_without_loss() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let mut f = Fixture::new();
    let moved = f.source.with_file_name(OsString::from_vec(b"source\xff".to_vec()));
    fs::rename(&f.source, &moved).unwrap();
    f.source = moved;
    let leaf = OsString::from_vec(b"leaf\xfe".to_vec());
    file(f.source.join(&leaf), b"native");
    let mut args = f.start_args(1, 0);
    args.extend(["--preserve".into(), assignment("B", Path::new(&leaf))]);
    f.invoke(args, 0);
    let p = f.parts();
    assert_eq!(p.comparison.bindings.source.as_os_str().as_bytes(), f.source.as_os_str().as_bytes());
    let b = &p.current.as_ref().unwrap().applications[1].decision;
    assert_eq!(b.excluded_names[0], leaf);
    f.invoke(f.resume_args(100, 100), 0);
    assert_eq!(fs::read(f.a.join(&leaf)).unwrap(), b"native");
    assert!(!f.b.join(&leaf).exists());
    // Reject foreign native units explicitly, rather than constructing an OS
    // string from Rust's unrelated private encoded-byte representation.
    let mut raw = raw_parts(&f.cursor);
    raw["state"]["comparison"]["source"] = serde_json::to_value(NativeString::Windows(vec![67, 58, 92])).unwrap();
    fs::write(&f.cursor, serde_json::to_vec(&raw).unwrap()).unwrap();
    let out = invoke(f.resume_args(0, 0));
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn wide_cumulative_counters_have_an_explicit_lossless_json_representation() {
    let f = Fixture::new();
    f.paired(Path::new("d"));
    file(f.source.join("d/leaf"), b"data");
    f.invoke(f.start_args(100, 100), 0);
    let (mut p, config) = transport::read(&f.cursor).unwrap();
    assert_eq!(p.lanes[0].applications, 2);
    p.lanes[0].completed.bytes_copied = u128::from(u64::MAX) + 7;
    transport::write(&f.cursor, &p, config).unwrap();
    assert_eq!(transport::read(&f.cursor).unwrap().0, p);
    let output = f.invoke(f.resume_args(0, 0), 0);
    assert_eq!(
        output["reports"][0]["completed"]["bytes_copied"].as_number().unwrap().as_u128(),
        Some(u128::from(u64::MAX) + 7)
    );
    assert_eq!(f.parts(), p);
}

#[cfg(unix)]
#[test]
fn explicit_subset_saves_shared_refresh_failures_without_redispatching_failed_peers() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    file(f.source.join("x"), b"source payload");
    file(f.b.join("x"), b"old");
    f.invoke(f.start_args(1, 0), 0);
    fs::remove_file(f.b.join("x")).unwrap();
    symlink("old", f.b.join("x")).unwrap();
    let mut args = f.resume_args(0, 2);
    args.extend(["--apply-only".into(), "A".into(), "--apply-only".into(), "B".into()]);
    let out = f.invoke(args, 2);
    assert_eq!(out["applications"], 1, "only A's refused shared-refresh request was attempted");
    assert_eq!(out["failures"].as_array().unwrap().len(), 1);
    assert!(!f.a.join("x").exists());
    assert!(fs::symlink_metadata(f.b.join("x")).unwrap().is_symlink());
    let parts = f.parts();
    assert_eq!(parts.lanes[1].end, Some(End::DecisionChanged));
    let applications = &parts.current.as_ref().unwrap().applications;
    assert_eq!(applications[0].state, ApplicationState::Unstarted);
    assert_eq!(applications[1].state, ApplicationState::Failed);
    f.invoke(f.resume_args(100, 100), 2);
    assert_eq!(fs::read(f.a.join("x")).unwrap(), b"source payload");
    assert_eq!(f.parts().lanes[1].applications, 0);
}

#[test]
fn alias_and_nested_write_destinations_refuse_before_any_application() {
    let f = Fixture::new();
    file(f.source.join("file"), b"data");
    for root in [&f.a, &f.a.join("nested")] {
        fs::create_dir_all(root).unwrap();
        let mut args = f.start_args(100, 100);
        // Replace the second --target's value with an alias/nested root.
        let second = args.iter().enumerate().filter(|(_, a)| a == &&OsString::from("--target")).nth(1).unwrap().0 + 1;
        args[second] = assignment("B", root);
        let out = invoke(args);
        assert_eq!(out.status.code(), Some(1));
        assert!(!f.a.join("file").exists());
        assert!(!f.cursor.exists());
    }
}

#[test]
fn assignment_paths_can_contain_equals_without_changing_target_identity() {
    let parsed = diff_dirs_examples_reconcile::native::assignment(OsStr::new("A=/path/with=equals")).unwrap();
    assert_eq!(parsed.0, "A");
    assert_eq!(parsed.1, Path::new("/path/with=equals"));
}
