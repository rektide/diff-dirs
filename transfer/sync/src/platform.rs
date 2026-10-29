use std::fs::{self, Metadata, Permissions};
use std::io;
use std::path::Path;

#[cfg(unix)]
pub(crate) fn writable(mut permissions: Permissions, directory: bool) -> Permissions {
    use std::os::unix::fs::PermissionsExt;
    permissions.set_mode(permissions.mode() | if directory { 0o700 } else { 0o600 });
    permissions
}

#[cfg(not(unix))]
pub(crate) fn writable(mut permissions: Permissions, _directory: bool) -> Permissions {
    permissions.set_readonly(false);
    permissions
}

pub(crate) fn same_permissions(a: &Permissions, b: &Permissions) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        a.mode() == b.mode()
    }
    #[cfg(not(unix))]
    {
        a.readonly() == b.readonly()
    }
}

#[cfg(unix)]
pub(crate) fn symlink(link: &Path, to: &Path, _metadata: Option<&Metadata>) -> io::Result<()> {
    std::os::unix::fs::symlink(link, to)
}

#[cfg(windows)]
pub(crate) fn symlink(link: &Path, to: &Path, metadata: Option<&Metadata>) -> io::Result<()> {
    use std::os::windows::fs::{FileTypeExt, symlink_dir, symlink_file};
    if metadata.expect("platform execution inputs checked").file_type().is_symlink_dir() {
        symlink_dir(link, to)
    } else {
        symlink_file(link, to)
    }
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn symlink(_link: &Path, _to: &Path, _metadata: Option<&Metadata>) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "symlink creation unsupported on this platform"))
}

pub(crate) fn remove_leaf(path: &Path, kind: diff_dirs_tree_inspect::Kind, metadata: Option<&Metadata>) -> io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::FileTypeExt;
        if kind == diff_dirs_tree_inspect::Kind::Symlink && metadata.expect("platform execution inputs checked").file_type().is_symlink_dir() {
            return fs::remove_dir(path);
        }
    }
    let _ = (kind, metadata);
    // Never chmod a leaf to make this succeed: it may have hardlink aliases.
    fs::remove_file(path)
}
