// Copyright 2026 tess
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use libloading::Library as NativeLibrary;

use crate::error::{interface_compatible, Error, Result, SUPPORTED_INTERFACE};
use crate::ffi::{
    BranchCreateAsync, BranchListAsync, BranchSwitchAsync, CommitAsync, CreateAsync, DiffAsync,
    HistoryAsync, ShutdownFn, StageAsync, StatusAsync, UnstageAsync, VersionFn, WriteAsync,
};
use crate::session::Session;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

pub(crate) struct Symbols {
    pub(crate) version: VersionFn,
    pub(crate) shutdown: ShutdownFn,
    pub(crate) repository_create: CreateAsync,
    pub(crate) repository_status: StatusAsync,
    pub(crate) file_stage: StageAsync,
    pub(crate) file_unstage: UnstageAsync,
    pub(crate) file_write: WriteAsync,
    pub(crate) revision_commit: CommitAsync,
    pub(crate) revision_history: HistoryAsync,
    pub(crate) revision_diff: DiffAsync,
    pub(crate) branch_list: BranchListAsync,
    pub(crate) branch_create: BranchCreateAsync,
    pub(crate) branch_switch: BranchSwitchAsync,
}

struct Inner {
    path: PathBuf,
    // Kept so the dynamic library is not unmapped while symbols are in use.
    _library: NativeLibrary,
    symbols: Symbols,
    /// Session calls fail once shutdown has been attempted.
    shut_down: AtomicBool,
    /// Serializes `lore_shutdown`. `Ok` means it has already succeeded.
    shutdown: Mutex<ShutdownState>,
}

struct ShutdownState {
    succeeded: bool,
}

static PROCESS_LIBRARY: Mutex<Option<Arc<Inner>>> = Mutex::new(None);

/// Process-wide handle to a loaded Lore C library.
///
/// Lore's C API keeps its worker threads and stores in process-global state.
/// The first successful [`Library::open`] loads the dynamic library. A later
/// open of the same path returns another handle to that state. A different
/// path is rejected. [`Library::shutdown`] is terminal for the process: the C
/// API cannot be reinitialized afterwards.
///
/// Dropping a handle does not shut the library down. Shutdown stops threads
/// that other handles in the same process would still need.
#[derive(Clone)]
pub struct Library {
    inner: Arc<Inner>,
}

impl Library {
    /// Load `path`, or reuse the library already loaded from that path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = std::path::absolute(path.as_ref())?;
        let mut slot = PROCESS_LIBRARY
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        if let Some(existing) = slot.as_ref() {
            if existing.shut_down.load(Ordering::Acquire) {
                return Err(Error::ShutDown);
            }
            if existing.path != path {
                return Err(Error::AlreadyLoaded {
                    loaded: existing.path.clone(),
                });
            }
            return Ok(Self {
                inner: Arc::clone(existing),
            });
        }

        // Hold the process lock across the load so two threads cannot map two
        // different libraries and then unmap the loser.
        let native = unsafe { NativeLibrary::new(&path) }.map_err(|error| Error::LibraryLoad {
            path: path.clone(),
            message: error.to_string(),
        })?;
        let symbols = load_symbols(&native, &path)?;
        let reported = read_version(symbols.version)?;
        if !interface_compatible(&reported) {
            return Err(Error::IncompatibleVersion {
                found: reported,
                expected: SUPPORTED_INTERFACE,
            });
        }

        let inner = Arc::new(Inner {
            path,
            _library: native,
            symbols,
            shut_down: AtomicBool::new(false),
            shutdown: Mutex::new(ShutdownState { succeeded: false }),
        });
        *slot = Some(Arc::clone(&inner));
        Ok(Self { inner })
    }

    /// Load the library named by `LORE_LIBRARY`, or by `LORE_BUILD_PATH`.
    ///
    /// `LORE_BUILD_PATH` may be the library file or a directory containing
    /// `lore.dll`, `liblore.so`, or `liblore.dylib`.
    pub fn from_env() -> Result<Self> {
        if let Some(path) = std::env::var_os("LORE_LIBRARY") {
            return Self::open(PathBuf::from(path));
        }
        if let Some(path) = std::env::var_os("LORE_BUILD_PATH") {
            return Self::open(library_from_build_path(PathBuf::from(path)));
        }
        Err(Error::InvalidArgument {
            field: "LORE_LIBRARY",
            message: "set LORE_LIBRARY or LORE_BUILD_PATH to the Lore C library".to_string(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.inner.path
    }

    /// Version string reported by `lore_version`.
    pub fn version(&self) -> Result<String> {
        self.ensure_open()?;
        read_version(self.inner.symbols.version)
    }

    /// Stop Lore's worker threads.
    ///
    /// The first call rejects later session operations with [`Error::ShutDown`].
    /// `lore_shutdown` itself is retried until it returns zero. After it
    /// succeeds, further `shutdown` calls return `Ok(())` and do not call it
    /// again. A call that is already waiting is not cancelled; Lore's header
    /// has no cancellation function.
    pub fn shutdown(&self) -> Result<()> {
        self.inner.shut_down.store(true, Ordering::Release);
        let mut state = self
            .inner
            .shutdown
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        if state.succeeded {
            return Ok(());
        }
        // Safety: the symbol came from the loaded Lore library. Shutdown is
        // serialized by `state`, so two threads do not enter it together.
        let code = unsafe { (self.inner.symbols.shutdown)() };
        if code != 0 {
            return Err(Error::Failed {
                operation: "shutdown",
                code,
                message: "lore_shutdown returned a non-zero status".to_string(),
            });
        }
        state.succeeded = true;
        Ok(())
    }

    /// Start a session rooted at `repository`.
    ///
    /// The path is made absolute immediately, so a later change to the process
    /// current directory does not retarget the session. The directory does not
    /// have to exist yet. [`Session::create_repository`] creates it.
    pub fn session(&self, repository: impl Into<PathBuf>) -> Result<Session> {
        Session::new(self.clone(), repository.into(), DEFAULT_TIMEOUT)
    }

    pub(crate) fn ensure_open(&self) -> Result<()> {
        if self.inner.shut_down.load(Ordering::Acquire) {
            Err(Error::ShutDown)
        } else {
            Ok(())
        }
    }

    pub(crate) fn symbols(&self) -> &Symbols {
        &self.inner.symbols
    }
}

impl std::fmt::Debug for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Library")
            .field("path", &self.inner.path)
            .field("shut_down", &self.inner.shut_down.load(Ordering::Relaxed))
            .finish()
    }
}

fn library_from_build_path(path: PathBuf) -> PathBuf {
    if path.is_dir() {
        let name = if cfg!(windows) {
            "lore.dll"
        } else if cfg!(target_os = "macos") {
            "liblore.dylib"
        } else {
            "liblore.so"
        };
        path.join(name)
    } else {
        path
    }
}

fn load_symbols(native: &NativeLibrary, path: &Path) -> Result<Symbols> {
    Ok(Symbols {
        version: copy_symbol(native, path, b"lore_version\0")?,
        shutdown: copy_symbol(native, path, b"lore_shutdown\0")?,
        repository_create: copy_symbol(native, path, b"lore_repository_create_async\0")?,
        repository_status: copy_symbol(native, path, b"lore_repository_status_async\0")?,
        file_stage: copy_symbol(native, path, b"lore_file_stage_async\0")?,
        file_unstage: copy_symbol(native, path, b"lore_file_unstage_async\0")?,
        file_write: copy_symbol(native, path, b"lore_file_write_async\0")?,
        revision_commit: copy_symbol(native, path, b"lore_revision_commit_async\0")?,
        revision_history: copy_symbol(native, path, b"lore_revision_history_async\0")?,
        revision_diff: copy_symbol(native, path, b"lore_revision_diff_async\0")?,
        branch_list: copy_symbol(native, path, b"lore_branch_list_async\0")?,
        branch_create: copy_symbol(native, path, b"lore_branch_create_async\0")?,
        branch_switch: copy_symbol(native, path, b"lore_branch_switch_async\0")?,
    })
}

fn copy_symbol<T: Copy>(native: &NativeLibrary, path: &Path, name: &[u8]) -> Result<T> {
    let symbol: libloading::Symbol<T> = unsafe { native.get(name) }.map_err(|error| {
        let label = String::from_utf8_lossy(name.strip_suffix(&[0]).unwrap_or(name));
        Error::LibraryLoad {
            path: path.to_path_buf(),
            message: format!("missing symbol {label}: {error}"),
        }
    })?;
    Ok(*symbol)
}

fn read_version(version: VersionFn) -> Result<String> {
    // Safety: `lore_version` returns a library-owned NUL-terminated string,
    // or null. Reading stops at 128 bytes so a missing NUL cannot walk memory.
    let ptr = unsafe { version() };
    if ptr.is_null() {
        return Err(Error::Malformed {
            operation: "lore_version",
            message: "returned a null pointer".to_string(),
        });
    }
    let mut len = 0usize;
    while len < 128 {
        if unsafe { *ptr.add(len) } == 0 {
            break;
        }
        len += 1;
    }
    if len == 128 {
        return Err(Error::Malformed {
            operation: "lore_version",
            message: "version string is not NUL-terminated within 128 bytes".to_string(),
        });
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_library_file_fails_without_sticking() {
        let missing = std::env::temp_dir().join("lore-sdk-missing-library.dll");
        let error = Library::open(&missing).unwrap_err();
        assert!(matches!(error, Error::LibraryLoad { .. }));
        let slot = PROCESS_LIBRARY
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        assert!(slot.is_none());
    }
}
