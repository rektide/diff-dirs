use std::fs::Metadata;

/// No-follow kind hint. Unknown means unavailable/unknown, never an absent side.
/// A listing hint is not full stat data, equality proof, or protection against
/// concurrent replacement. Consumers decide which late facts they require.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Unknown,
    Directory,
    RegularFile,
    Symlink,
    Fifo,
    Socket,
    BlockDevice,
    CharacterDevice,
    /// A known kind not represented by another variant.
    Other,
}

impl FileKind {
    pub(crate) fn from_metadata(metadata: &Metadata) -> Self {
        let kind = metadata.file_type();
        if kind.is_dir() {
            return Self::Directory;
        }
        if kind.is_file() {
            return Self::RegularFile;
        }
        if kind.is_symlink() {
            return Self::Symlink;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            if kind.is_fifo() {
                return Self::Fifo;
            }
            if kind.is_socket() {
                return Self::Socket;
            }
            if kind.is_block_device() {
                return Self::BlockDevice;
            }
            if kind.is_char_device() {
                return Self::CharacterDevice;
            }
        }
        Self::Other
    }
}

/// Where early facts came from; root binding is not a child directory listing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactOrigin {
    /// Already returned by enumeration, without extra child observation I/O.
    Listing,
    /// Reused mandatory no-follow structural-root metadata.
    RootMetadata,
    /// Explicit no-follow acquisition, also used to resolve unknown listing kinds.
    AcquiredMetadata,
}

/// Immutable already-available facts for a present filesystem entry.
/// Accessors never perform I/O. Linux `native-dirent` exposes cached d_type and
/// d_ino; DT_UNKNOWN remains Unknown. The default std Unix backend exposes inode
/// but leaves kind Unknown, avoiding std's hidden file_type metadata fallback.
/// Windows uses std's cached enumeration metadata; other std platforms may have
/// neither field. Root bindings can instead reuse mandatory root metadata.
///
/// ```no_run
/// use std::path::Path;
/// use diff_dirs_tree_observe::{FileKind, read_directory};
/// # fn main() -> Result<(), diff_dirs_tree_observe::Error> {
/// for entry in read_directory(Path::new("tree"))? {
///     let hints = entry.hints(); // cached only, including Unknown
///     if hints.kind() == FileKind::Unknown {
///         let _metadata = entry.metadata()?; // explicit no-follow acquisition
///     }
/// }
/// # Ok(())
/// # }
/// ```
/// A hint alone is not a portable final type policy or a comparison result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarlyFacts {
    pub(crate) kind: FileKind,
    pub(crate) inode: Option<u64>,
    pub(crate) origin: FactOrigin,
}

impl EarlyFacts {
    /// Cached no-follow kind, or Unknown. Does not trigger a stat fallback.
    pub fn kind(&self) -> FileKind {
        self.kind
    }
    /// Enumeration inode on Unix, or root metadata inode; None if unavailable.
    /// This is not a device/inode identity or a guarantee against replacement.
    pub fn inode(&self) -> Option<u64> {
        self.inode
    }
    /// Listing facts versus the mandatory structural-root exception.
    pub fn origin(&self) -> FactOrigin {
        self.origin
    }
    /// Project metadata already obtained for a structural root, without I/O.
    /// The caller supplies actual no-follow binding metadata; this does not
    /// validate that root or create authority to construct a comparison plan.
    pub fn from_root_metadata(metadata: &Metadata) -> Self {
        #[cfg(unix)]
        let inode = Some(std::os::unix::fs::MetadataExt::ino(metadata));
        #[cfg(not(unix))]
        let inode = None;
        Self {
            kind: FileKind::from_metadata(metadata),
            inode,
            origin: FactOrigin::RootMetadata,
        }
    }

    /// Project an actual no-follow child lookup, distinct from root binding and
    /// the immutable hints of a directory enumeration.
    pub fn from_acquired_metadata(metadata: &Metadata) -> Self {
        let mut facts = Self::from_root_metadata(metadata);
        facts.origin = FactOrigin::AcquiredMetadata;
        facts
    }
}
