//! Optional example-owned transport. JSON output is not the continuation codec.
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use kopacetic_core::EntityIdentity;
use kopacetic_rkyv::{Schema, Snapshot};

use crate::{Error, NativePath, SavedWalk, require};

/// Archive bytes only, not a projection/reconstruction allocation bound.
pub const MAX_ARCHIVE_BYTES: usize = 8 * 1024 * 1024;

fn schema() -> Result<Schema, Error> { Ok(Schema::new("diff-dirs.walk-example", 1)?) }
fn identity() -> EntityIdentity { EntityIdentity::standalone("WalkingCursor", "example") }

pub fn encode(saved: &SavedWalk) -> Result<Vec<u8>, Error> {
    saved.validate()?;
    let bytes = Snapshot::from_value(&identity(), &schema()?, saved)?.into_bytes();
    require(bytes.len() <= MAX_ARCHIVE_BYTES, "archive exceeds 8 MiB limit")?;
    Ok(bytes)
}

pub fn decode(bytes: Vec<u8>) -> Result<SavedWalk, Error> {
    require(bytes.len() <= MAX_ARCHIVE_BYTES, "archive exceeds 8 MiB limit")?;
    let snapshot = Snapshot::from_bytes(bytes, &schema()?)?;
    require(snapshot.identity() == &identity(), "unexpected walking snapshot identity")?;
    let saved = snapshot.reconstruct::<SavedWalk>()?;
    saved.validate()?;
    Ok(saved)
}

pub fn read_cursor(path: &Path) -> Result<SavedWalk, Error> {
    let file = File::open(path).map_err(|e| Error::io(path, "open cursor", e))?;
    let mut bytes = Vec::new();
    file.take((MAX_ARCHIVE_BYTES + 1) as u64).read_to_end(&mut bytes).map_err(|e| Error::io(path, "read cursor", e))?;
    decode(bytes)
}

/// Validate the publication coordinate without consulting the tree roots, including
/// for terminal cursors whose roots may no longer exist. Resolve the output parent.
pub fn validate_cursor_path(path: &Path, saved: &SavedWalk) -> Result<(), Error> {
    NativePath::new(path).validate(true)?;
    let parent = path.parent().ok_or_else(|| Error::invalid("cursor needs a filename"))?;
    let resolved = fs::canonicalize(parent).map_err(|e| Error::io(parent, "resolve cursor parent", e))?.join(path.file_name().ok_or_else(|| Error::invalid("cursor needs a filename"))?);
    for root in saved.bindings.roots() {
        require(!path.starts_with(&root) && !resolved.starts_with(&root), "cursor must be outside source, target and reports")?;
    }
    Ok(())
}

/// Complete-file replacement, separate from tree effects and stdout. No rollback
/// guarantee if this publication fails after visible work. No directory fsync claim.
pub fn write_cursor(path: &Path, saved: &SavedWalk) -> Result<(), Error> {
    saved.validate()?;
    validate_cursor_path(path, saved)?;
    let bytes = encode(saved)?;
    let temporary = tempfile::NamedTempFile::new_in(path.parent().expect("validated parent")).map_err(|e| Error::io(path, "prepare cursor staging", e))?;
    let (mut file, staging) = temporary.into_parts();
    let result = file.write_all(&bytes).and_then(|()| file.sync_all()).map_err(|e| Error::io(path, "write cursor staging", e));
    drop(file);
    crate::publish_staging(staging, path, result)
}
