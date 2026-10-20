use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use kopacetic_core::EntityIdentity;
use kopacetic_rkyv::{Schema, Snapshot};

use crate::{Cursor, Error, require};

/// Archive-byte bound, not a bound on projection/reconstruction allocations.
pub const MAX_ARCHIVE_BYTES: usize = 8 * 1024 * 1024;

fn schema() -> Result<Schema, Error> {
    Ok(Schema::new("diff-dirs.cursor-example", 1)?)
}

fn identity() -> EntityIdentity {
    EntityIdentity::standalone("DirectoryCursor", "example")
}

pub fn encode(cursor: &Cursor) -> Result<Vec<u8>, Error> {
    cursor.validate()?;
    let bytes = Snapshot::from_value(&identity(), &schema()?, cursor)?.into_bytes();
    require(bytes.len() <= MAX_ARCHIVE_BYTES, "archive exceeds 8 MiB limit")?;
    Ok(bytes)
}

pub fn decode(bytes: Vec<u8>) -> Result<Cursor, Error> {
    require(bytes.len() <= MAX_ARCHIVE_BYTES, "archive exceeds 8 MiB limit")?;
    let snapshot = Snapshot::from_bytes(bytes, &schema()?)?;
    require(snapshot.identity() == &identity(), "unexpected snapshot identity")?;
    let cursor = snapshot.reconstruct::<Cursor>()?;
    cursor.validate()?;
    Ok(cursor)
}

pub fn read_cursor(path: &Path) -> Result<Cursor, Error> {
    let file = File::open(path).map_err(|e| Error::io(path, "open cursor", e))?;
    let mut bytes = Vec::new();
    file.take((MAX_ARCHIVE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| Error::io(path, "read cursor", e))?;
    decode(bytes)
}

/// Complete-file publication only; no coupling to stdout or filesystem effects.
pub fn write_cursor(path: &Path, cursor: &Cursor) -> Result<(), Error> {
    let bytes = encode(cursor)?;
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| Error::io(path, "prepare cursor output", e))?;
    temporary.write_all(&bytes).map_err(|e| Error::io(path, "write cursor", e))?;
    temporary.as_file().sync_all().map_err(|e| Error::io(path, "sync cursor file", e))?;
    temporary.persist(path).map_err(|e| Error::io(path, "publish cursor", e.error))?;
    Ok(())
}
