//! Immutable native-name storage. Encoded bytes are process-local, never a codec.

use std::ffi::{OsStr, OsString};

/// Stable index within one immutable name table, not a global name identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NameId(pub usize);

/// Compact names with independently owned promotion at recursive boundaries.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NameTable {
    #[cfg(unix)]
    bytes: Vec<u8>,
    #[cfg(unix)]
    ranges: Vec<std::ops::Range<usize>>,
    #[cfg(not(unix))]
    names: Vec<OsString>,
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;
    #[test]
    fn native_names_are_exact_and_child_promotion_does_not_pin_the_table() {
        let long = OsString::from_vec([vec![b'x'; 255], vec![0xff]].concat());
        let table = NameTable::new([OsString::from("a"), long.clone()]);
        assert_eq!(table.get(NameId(1)), long);
        assert!(table.charged_capacity() >= 257);
        let child = table.get(NameId(1)).to_owned();
        drop(table);
        assert_eq!(child, long);
    }
}

impl NameTable {
    pub fn new(names: impl IntoIterator<Item = OsString>) -> Self {
        let mut table = Self::default();
        for name in names {
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                let start = table.bytes.len();
                table.bytes.extend_from_slice(name.as_bytes());
                table.ranges.push(start..table.bytes.len());
            }
            #[cfg(not(unix))]
            table.names.push(name);
        }
        table
    }

    pub fn get(&self, id: NameId) -> &OsStr {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            OsStr::from_bytes(&self.bytes[self.ranges[id.0].clone()])
        }
        #[cfg(not(unix))]
        {
            &self.names[id.0]
        }
    }

    pub fn len(&self) -> usize {
        #[cfg(unix)]
        {
            self.ranges.len()
        }
        #[cfg(not(unix))]
        {
            self.names.len()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Owned capacity, not RSS. Used by bounded runtime observation retention.
    pub fn charged_capacity(&self) -> usize {
        #[cfg(unix)]
        {
            self.bytes.capacity() + self.ranges.capacity() * size_of::<std::ops::Range<usize>>()
        }
        #[cfg(not(unix))]
        {
            self.names.capacity() * size_of::<OsString>() + self.names.iter().map(OsString::capacity).sum::<usize>()
        }
    }
}
