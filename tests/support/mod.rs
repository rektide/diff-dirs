#![allow(dead_code)]

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static NEXT: AtomicU64 = AtomicU64::new(0);

pub struct Fixture {
    pub root: PathBuf,
    pub original: PathBuf,
    pub target: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let tmp = Path::new("/tmp/opencode");
        let tmp = if tmp.is_dir() { tmp.to_path_buf() } else { std::env::temp_dir() };
        let root = tmp.join(format!("diff-dirs-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&root).unwrap();
        let original = root.join("original");
        let target = root.join("target");
        fs::create_dir(&original).unwrap();
        fs::create_dir(&target).unwrap();
        Self { root, original, target }
    }

    pub fn both_dir(&self, path: impl AsRef<Path>) {
        fs::create_dir_all(self.original.join(&path)).unwrap();
        fs::create_dir_all(self.target.join(path)).unwrap();
    }

    pub fn both_file(&self, path: impl AsRef<Path>, contents: &[u8]) {
        file(&self.original.join(&path), contents);
        file(&self.target.join(path), contents);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

pub fn timestamp() -> SystemTime {
    UNIX_EPOCH + Duration::new(1_700_000_000, 123_456_789)
}

pub fn set_time(path: &Path, time: SystemTime) {
    File::options().write(true).open(path).unwrap().set_modified(time).unwrap();
}

pub fn file(path: &Path, contents: &[u8]) {
    fs::write(path, contents).unwrap();
    set_time(path, timestamp());
}

pub fn gather(f: &Fixture) -> (Vec<diff_dirs::DirDiff>, diff_dirs::Summary) {
    let mut walk = diff_dirs::walk(&f.original, &f.target).unwrap();
    let diffs = walk.by_ref().collect::<Result<Vec<_>, _>>().unwrap();
    (diffs, walk.into_summary())
}
