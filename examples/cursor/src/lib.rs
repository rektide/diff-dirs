//! Unix-only, read-only directory assessment with an explicit DFS continuation.
//! A unit is one attempted immediate-directory assessment, including validation.
//! Neither the cursor nor generated virtual files are an effects history.

#[cfg(not(unix))]
compile_error!("the cursor example supports Unix native paths only");

mod codec;
mod cursor;

pub use codec::{MAX_ARCHIVE_BYTES, decode, encode, read_cursor, write_cursor};
pub use cursor::{Block, Completion, Counts, Cursor, Descriptor, Event, FinalState, Frame, NativePath, Phase, Run, RunOptions, Tip, VirtualFile};

/// Command/codec failure; assessment failures instead leave a blocked cursor.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid cursor: {0}")]
    Invalid(String),
    #[error("{operation} {path:?}: {source}")]
    Io {
        operation: &'static str,
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Archive(#[from] kopacetic_rkyv::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl Error {
    pub(crate) fn io(path: &std::path::Path, operation: &'static str, source: std::io::Error) -> Self {
        Self::Io {
            operation,
            path: path.to_owned(),
            source,
        }
    }
}

pub(crate) fn require(condition: bool, message: &str) -> Result<(), Error> {
    if condition { Ok(()) } else { Err(Error::Invalid(message.into())) }
}

pub(crate) fn increment(value: &mut u64, amount: u64) -> Result<(), Error> {
    *value = value.checked_add(amount).ok_or_else(|| Error::Invalid("counter overflow".into()))?;
    Ok(())
}
