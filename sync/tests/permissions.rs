#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Set a process-global umask only in an isolated child process, not in the
/// shared test runner while other copy work can execute.
#[test]
fn restrictive_umask_listing_failure_retains_permission_progress() {
    const CHILD: &str = "DIFF_DIRS_SYNC_UMASK_CHILD";
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.test-agent/sync-tests");
    if std::env::var_os(CHILD).is_none() {
        fs::create_dir_all(&scratch).unwrap();
        let output = std::process::Command::new("sh")
            .args([
                "-c",
                "umask 0777; exec \"$1\" --exact restrictive_umask_listing_failure_retains_permission_progress --nocapture",
                "sync-permission-test",
            ])
            .arg(std::env::current_exe().unwrap())
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    fs::create_dir_all(&scratch).unwrap();
    let owned = tempfile::tempdir_in(scratch).unwrap();
    fs::set_permissions(owned.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let source = owned.path().join("source");
    let target = owned.path().join("target");
    fs::create_dir(&source).unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o111)).unwrap();
    if fs::read_dir(&source).is_ok() {
        // A privileged account can bypass this permission fault; ordinary
        // unprivileged Linux verification takes the asserted failure branch.
        fs::set_permissions(&source, fs::Permissions::from_mode(0o700)).unwrap();
        eprintln!("privileged account bypasses directory listing restrictions");
        return;
    }
    let failure = diff_dirs_sync::mirror(&source, &target, diff_dirs_sync::CopyPolicy::Copy).unwrap_err();
    assert_eq!(failure.primary.operation, "read directory");
    assert_eq!(failure.completed.directories_created, 1);
    assert_eq!(failure.completed.directory_permissions_set, 2); // relax + finalize
    assert_eq!(failure.completed.files_copied, 0);
    assert_eq!(fs::metadata(&target).unwrap().permissions().mode() & 0o777, 0o111);
    assert_eq!(fs::metadata(&source).unwrap().permissions().mode() & 0o777, 0o111);
    fs::set_permissions(&source, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn restrictive_umask_does_not_block_private_staging_or_final_mode() {
    const CHILD: &str = "DIFF_DIRS_SYNC_UMASK_SUCCESS_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new("sh")
            .args([
                "-c",
                "umask 0777; exec \"$1\" --exact restrictive_umask_does_not_block_private_staging_or_final_mode --nocapture",
                "sync-permission-test",
            ])
            .arg(std::env::current_exe().unwrap())
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.test-agent/sync-tests");
    fs::create_dir_all(&scratch).unwrap();
    let owned = tempfile::tempdir_in(scratch).unwrap();
    fs::set_permissions(owned.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let source = owned.path().join("source");
    fs::create_dir(&source).unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(source.join("file"), vec![b'x'; 4097]).unwrap();
    fs::set_permissions(source.join("file"), fs::Permissions::from_mode(0o444)).unwrap();
    let available = match reflink_copy::reflink(source.join("file"), owned.path().join("clone-probe")) {
        Ok(()) => true,
        Err(error) => {
            assert!(std::env::var_os("DIFF_DIRS_SYNC_REQUIRE_REFLINK").is_none(), "real reflink required: {error}");
            false
        }
    };
    for (index, policy) in [
        diff_dirs_sync::CopyPolicy::Copy,
        diff_dirs_sync::CopyPolicy::ReflinkRequired,
        diff_dirs_sync::CopyPolicy::ReflinkOrCopy,
    ]
    .into_iter()
    .enumerate()
    {
        if policy != diff_dirs_sync::CopyPolicy::Copy && !available {
            continue;
        }
        let target = owned.path().join(format!("target-{index}"));
        let report = diff_dirs_sync::mirror(&source, &target, policy).unwrap();
        assert_eq!(report.completed.files_copied + report.completed.files_reflinked, 1);
        assert_eq!(fs::metadata(target.join("file")).unwrap().permissions().mode() & 0o777, 0o444);
        assert_eq!(fs::read(target.join("file")).unwrap(), vec![b'x'; 4097]);
        assert_eq!(fs::read_dir(target).unwrap().count(), 1);
    }
}
