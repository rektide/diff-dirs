use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Result;

/// Application wire spelling. Neither UTF-8 substitution nor Rust's private
/// encoded-byte representation is used. Foreign platforms refuse reconstruction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "platform", content = "units", rename_all = "snake_case")]
pub enum NativeString {
    Unix(Vec<u8>),
    Windows(Vec<u16>),
}

impl NativeString {
    pub fn new(value: &OsStr) -> Self {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            Self::Unix(value.as_bytes().to_vec())
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            Self::Windows(value.encode_wide().collect())
        }
        #[cfg(not(any(unix, windows)))]
        compile_error!("reconcile example transport supports Unix and Windows native strings");
    }
    pub fn into_os(self) -> Result<OsString> {
        match self {
            #[cfg(unix)]
            Self::Unix(bytes) => {
                use std::os::unix::ffi::OsStringExt;
                Ok(OsString::from_vec(bytes))
            }
            #[cfg(windows)]
            Self::Windows(units) => {
                use std::os::windows::ffi::OsStringExt;
                Ok(OsString::from_wide(&units))
            }
            _ => Err("foreign native string platform".into()),
        }
    }
    pub fn path(&self) -> Result<PathBuf> {
        Ok(self.clone().into_os()?.into())
    }
}

/// IDs are UTF-8; everything after the first '=' remains a lossless native path.
pub fn assignment(value: &OsStr) -> Result<(String, PathBuf)> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let bytes = value.as_bytes();
        let split = bytes.iter().position(|&b| b == b'=').ok_or("expected ID=PATH")?;
        let id = std::str::from_utf8(&bytes[..split])?.to_owned();
        if id.is_empty() {
            return Err("empty target ID".into());
        }
        Ok((id, OsString::from_vec(bytes[split + 1..].to_vec()).into()))
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        let units: Vec<_> = value.encode_wide().collect();
        let split = units.iter().position(|&b| b == u16::from(b'=')).ok_or("expected ID=PATH")?;
        let id = String::from_utf16(&units[..split])?;
        if id.is_empty() {
            return Err("empty target ID".into());
        }
        Ok((id, OsString::from_wide(&units[split + 1..]).into()))
    }
}
