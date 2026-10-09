// Copyright 2026 tess
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use std::time::Duration;

/// Errors returned by the SDK.
///
/// Library messages are copied from Lore's completion event. This crate does
/// not add credential material to them. Callers should still avoid logging a
/// [`Error::Failed`] message if they later point the same process at a remote
/// that echoes request data.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load the Lore library at {path}: {message}")]
    LibraryLoad { path: PathBuf, message: String },

    #[error(
        "Lore library {found:?} does not match interface {expected} (layouts are bound to that version)"
    )]
    IncompatibleVersion {
        found: String,
        expected: &'static str,
    },

    #[error("the Lore library in this process has been shut down")]
    ShutDown,

    #[error("a Lore library is already loaded from {loaded}")]
    AlreadyLoaded { loaded: PathBuf },

    #[error("invalid {field}: {message}")]
    InvalidArgument {
        field: &'static str,
        message: String,
    },

    #[error("{operation} timed out after {timeout:?}; the library call may still be running")]
    Timeout {
        operation: &'static str,
        timeout: Duration,
    },

    #[error("{operation} emitted more than {limit} events")]
    EventLimit {
        operation: &'static str,
        limit: usize,
    },

    #[error("file is {actual} bytes, which is above the {limit} byte read limit")]
    TooLarge { limit: u64, actual: u64 },

    #[error("{operation} returned a malformed event: {message}")]
    Malformed {
        operation: &'static str,
        message: String,
    },

    #[error("{operation} failed ({code}): {message}")]
    Failed {
        operation: &'static str,
        code: i32,
        message: String,
    },

    #[error("no Lore repository found at or above {start}")]
    NotARepository { start: PathBuf },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Interface version this crate's struct layouts were measured against.
pub const SUPPORTED_INTERFACE: &str = "0.10.1";

/// `lore_version` returns `LORE_INTERFACE_VERSION`, then `+` and a build name.
///
/// ```
/// assert!(lore_sdk::interface_compatible("0.10.1+840"));
/// assert!(!lore_sdk::interface_compatible("0.10.10"));
/// ```
pub fn interface_compatible(reported: &str) -> bool {
    let version = reported.split('+').next().unwrap_or(reported);
    version == SUPPORTED_INTERFACE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_match_is_exact_before_the_build_suffix() {
        assert!(interface_compatible("0.10.1"));
        assert!(interface_compatible("0.10.1+local"));
        assert!(!interface_compatible("0.10.10"));
        assert!(!interface_compatible("0.10.0"));
        assert!(!interface_compatible("0.10"));
        assert!(!interface_compatible(""));
    }
}
