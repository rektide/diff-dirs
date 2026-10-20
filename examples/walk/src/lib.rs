//! A bounded Unix walking specimen, not a production synchronizer or recovery log.
//! Saved data owns future obligations; supplied components own executable code.

#[cfg(not(unix))]
compile_error!("the walking example uses Unix native byte paths");

#[cfg(feature = "codec")]
pub mod codec;
mod driver;
mod io;
mod program;
mod state;

pub use driver::*;
pub use io::*;
pub use program::*;
pub use state::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid walking state or operation: {0}")]
    Invalid(String),
    #[error("{operation} {path:?}: {source}")]
    Io {
        operation: &'static str,
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Inspection(#[from] diff_dirs::Error),
    #[error("{primary}; staging cleanup also failed: {cleanup}")]
    Cleanup { primary: Box<Error>, cleanup: Box<Error> },
    #[cfg(feature = "codec")]
    #[error(transparent)]
    Archive(#[from] kopacetic_rkyv::Error),
}

impl Error {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }

    pub(crate) fn io(path: &std::path::Path, operation: &'static str, source: std::io::Error) -> Self {
        Self::Io { operation, path: path.to_owned(), source }
    }
}

pub(crate) fn require(condition: bool, message: &str) -> Result<(), Error> {
    if condition { Ok(()) } else { Err(Error::invalid(message)) }
}

pub(crate) fn add(a: u64, b: u64) -> Result<u64, Error> {
    a.checked_add(b).ok_or_else(|| Error::invalid("counter overflow"))
}
