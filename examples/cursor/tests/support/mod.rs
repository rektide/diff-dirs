#![allow(dead_code)]

use std::fs::{self, File};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

pub struct Fixture {
    pub directory: tempfile::TempDir,
    pub source: PathBuf,
    pub target: PathBuf,
}

impl Fixture {
    pub fn empty() -> Self {
        let directory = tempfile::Builder::new().prefix("cursor-test-").tempdir_in("/tmp/opencode").unwrap();
        let source = directory.path().join("source");
        let target = directory.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();
        Self { directory, source, target }
    }

    pub fn mixed() -> Self {
        let fixture = Self::empty();
        for name in ["a/deep", "b"] {
            fixture.both_dir(name);
        }
        fs::create_dir_all(fixture.source.join("source/sub")).unwrap();
        fs::create_dir_all(fixture.target.join("target/sub")).unwrap();
        fs::create_dir(fixture.source.join("collision")).unwrap();
        file(&fixture.target.join("collision"), b"old leaf");
        fixture.both_file("equal", b"same");
        fixture.both_file("a/file", b"same");
        file(&fixture.source.join("changed"), b"new-longer");
        file(&fixture.target.join("changed"), b"old");
        file(&fixture.source.join("only-source"), b"source");
        file(&fixture.target.join("only-target"), b"target");
        file(&fixture.source.join("source/file"), b"source");
        file(&fixture.source.join("source/sub/file"), b"source");
        file(&fixture.target.join("target/file"), b"target");
        file(&fixture.target.join("target/sub/file"), b"target");
        file(&fixture.source.join("collision/file"), b"source");
        std::os::unix::fs::symlink("equal", fixture.source.join("link")).unwrap();
        std::os::unix::fs::symlink("different", fixture.target.join("link")).unwrap();
        fixture.both_dir(non_utf8());
        fixture.both_file(non_utf8().join("file"), b"same");
        fixture
    }

    pub fn both_dir(&self, path: impl AsRef<Path>) {
        fs::create_dir_all(self.source.join(&path)).unwrap();
        fs::create_dir_all(self.target.join(path)).unwrap();
    }

    pub fn both_file(&self, path: impl AsRef<Path>, contents: &[u8]) {
        file(&self.source.join(&path), contents);
        file(&self.target.join(path), contents);
    }
}

pub fn non_utf8() -> PathBuf {
    std::ffi::OsString::from_vec(vec![b'n', 0xff]).into()
}

pub fn file(path: &Path, contents: &[u8]) {
    fs::write(path, contents).unwrap();
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(UNIX_EPOCH + Duration::new(1_700_000_000, 123))
        .unwrap();
}
