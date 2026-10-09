// Copyright 2026 tess
// SPDX-License-Identifier: MIT

use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{Error, Result};
use crate::ffi::{
    invoke, BranchCreateArgs, BranchListArgs, BranchSwitchArgs, FileStageArgs, FileUnstageArgs,
    FileWriteArgs, GlobalArgs, RepositoryCreateArgs, RepositoryStatusArgs, RevisionCommitArgs,
    RevisionDiffArgs, RevisionHistoryArgs, Text, TextList, SHARED_STORE_DISABLED, VFS_NONE,
};
use crate::library::Library;
use crate::model::{
    Branch, BranchCreated, Commit, DiffEntry, HistoryEntry, RepositoryCreated, StagedPath, Status,
};

/// Options for [`Session::status_with`].
#[derive(Debug, Clone, Copy)]
pub struct StatusOptions {
    /// Walk the working tree and refresh dirty tracking.
    pub scan: bool,
    /// Include the staged changeset in the report.
    pub staged: bool,
}

/// A local Lore repository used through one loaded library.
///
/// Every operation in this session sets the C API's offline and local flags.
/// Nothing here authenticates, contacts a remote, or reads a token. Create
/// also asks for a private store (`LORE_SHARED_STORE_MODE_DISABLED`) so a
/// machine-wide shared store is not selected by inheritance.
///
/// Calls block the calling thread until Lore emits its end event, or until
/// [`Session::timeout`] elapses. A timeout does not cancel the library call.
/// Lore's C header describes no cancellation entry point.
pub struct Session {
    library: Library,
    repository: PathBuf,
    timeout: Duration,
    event_limit: usize,
}

impl Session {
    pub(crate) fn new(library: Library, repository: PathBuf, timeout: Duration) -> Result<Self> {
        if repository.as_os_str().is_empty() {
            return Err(Error::InvalidArgument {
                field: "repository",
                message: "path must not be empty".to_string(),
            });
        }
        Ok(Self {
            library,
            repository: std::path::absolute(repository)?,
            timeout,
            event_limit: crate::ffi::EVENT_LIMIT_DEFAULT,
        })
    }

    pub fn repository(&self) -> &Path {
        &self.repository
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Cap how many file, branch, history, or diff events one call retains.
    ///
    /// Completion is still recorded after the cap, and the call then fails
    /// with [`Error::EventLimit`] instead of growing without bound.
    pub fn event_limit(mut self, limit: usize) -> Self {
        self.event_limit = limit;
        self
    }

    /// Create a repository at this session's path.
    ///
    /// `name` is the repository name. An empty name lets Lore name it after
    /// the directory. The directory is created if it is missing.
    pub fn create_repository(&self, name: &str) -> Result<RepositoryCreated> {
        let path = self.prepare_directory()?;
        let name = Text::new("name", name)?;
        let empty = Text::new("description", "")?;
        let mut args = RepositoryCreateArgs::empty();
        args.repository_url = name.as_ffi();
        args.description = empty.as_ffi();
        args.id = empty.as_ffi();
        args.vfs = VFS_NONE;
        args.use_shared_store = SHARED_STORE_DISABLED;
        args.shared_store_path = empty.as_ffi();
        let outcome = self.call(
            |globals| {
                invoke(
                    "create_repository",
                    self.library.symbols().repository_create,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        outcome.created.ok_or_else(|| Error::Malformed {
            operation: "create_repository",
            message: "repository create event was missing".to_string(),
        })
    }

    pub fn status(&self) -> Result<Status> {
        self.status_with(StatusOptions {
            scan: true,
            staged: true,
        })
    }

    pub fn status_with(&self, options: StatusOptions) -> Result<Status> {
        let path = self.repository_text()?;
        let paths = TextList::new("paths", &[])?;
        let mut args = RepositoryStatusArgs::empty();
        args.staged = u8::from(options.staged);
        args.scan = u8::from(options.scan);
        args.paths = paths.as_ffi();
        let outcome = self.call(
            |globals| {
                invoke(
                    "status",
                    self.library.symbols().repository_status,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        outcome.status.ok_or_else(|| Error::Malformed {
            operation: "status",
            message: "status revision event was missing".to_string(),
        })
    }

    /// Stage repository-relative or absolute paths.
    pub fn stage(&self, paths: &[&str]) -> Result<Vec<StagedPath>> {
        if paths.is_empty() {
            return Err(Error::InvalidArgument {
                field: "paths",
                message: "stage requires at least one path".to_string(),
            });
        }
        let path = self.repository_text()?;
        let listed = TextList::new("paths", paths)?;
        let mut args = FileStageArgs::empty();
        args.paths = listed.as_ffi();
        let outcome = self.call(
            |globals| {
                invoke(
                    "stage",
                    self.library.symbols().file_stage,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        Ok(outcome.staged)
    }

    pub fn unstage(&self, paths: &[&str]) -> Result<()> {
        if paths.is_empty() {
            return Err(Error::InvalidArgument {
                field: "paths",
                message: "unstage requires at least one path".to_string(),
            });
        }
        let path = self.repository_text()?;
        let listed = TextList::new("paths", paths)?;
        let mut args = FileUnstageArgs::empty();
        args.paths = listed.as_ffi();
        self.call(
            |globals| {
                invoke(
                    "unstage",
                    self.library.symbols().file_unstage,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<Commit> {
        let path = self.repository_text()?;
        let message = Text::new("message", message)?;
        let empty = Text::new("link", "")?;
        let no_links = TextList::new("link_paths", &[])?;
        let no_link_messages = TextList::new("link_messages", &[])?;
        let no_layers = TextList::new("layer_paths", &[])?;
        let no_layer_messages = TextList::new("layer_messages", &[])?;
        let mut args = RevisionCommitArgs::empty();
        args.message = message.as_ffi();
        args.link = empty.as_ffi();
        args.link_paths = no_links.as_ffi();
        args.link_messages = no_link_messages.as_ffi();
        args.layer = empty.as_ffi();
        args.layer_paths = no_layers.as_ffi();
        args.layer_messages = no_layer_messages.as_ffi();
        let outcome = self.call(
            |globals| {
                invoke(
                    "commit",
                    self.library.symbols().revision_commit,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        outcome.commit.ok_or_else(|| Error::Malformed {
            operation: "commit",
            message: "commit revision event was missing".to_string(),
        })
    }

    /// Revision history of the current branch, newest first as Lore emits it.
    ///
    /// `limit` is passed as the C API's `length`. Zero is rejected here even
    /// though the C API treats zero as unlimited.
    pub fn history(&self, limit: u32) -> Result<Vec<HistoryEntry>> {
        if limit == 0 {
            return Err(Error::InvalidArgument {
                field: "limit",
                message: "history limit must be at least 1".to_string(),
            });
        }
        let path = self.repository_text()?;
        let empty = Text::new("revision", "")?;
        let mut args = RevisionHistoryArgs::empty();
        args.revision = empty.as_ffi();
        args.branch = empty.as_ffi();
        args.length = limit;
        args.only_branch = 1;
        let outcome = self.call(
            |globals| {
                invoke(
                    "history",
                    self.library.symbols().revision_history,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        Ok(outcome.history)
    }

    pub fn diff(&self, source: &str, target: &str) -> Result<Vec<DiffEntry>> {
        let path = self.repository_text()?;
        let source = Text::new("source", source)?;
        let target = Text::new("target", target)?;
        let paths = TextList::new("paths", &[])?;
        let mut args = RevisionDiffArgs::empty();
        args.revision_source = source.as_ffi();
        args.revision_target = target.as_ffi();
        args.paths = paths.as_ffi();
        let outcome = self.call(
            |globals| {
                invoke(
                    "diff",
                    self.library.symbols().revision_diff,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        Ok(outcome.diff)
    }

    pub fn branches(&self) -> Result<Vec<Branch>> {
        let path = self.repository_text()?;
        let args = BranchListArgs::empty();
        let outcome = self.call(
            |globals| {
                invoke(
                    "branches",
                    self.library.symbols().branch_list,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        Ok(outcome.branches)
    }

    pub fn create_branch(&self, name: &str) -> Result<BranchCreated> {
        if name.is_empty() {
            return Err(Error::InvalidArgument {
                field: "name",
                message: "branch name must not be empty".to_string(),
            });
        }
        let path = self.repository_text()?;
        let name = Text::new("name", name)?;
        let empty = Text::new("category", "")?;
        let mut args = BranchCreateArgs::empty();
        args.branch = name.as_ffi();
        args.category = empty.as_ffi();
        args.id = empty.as_ffi();
        let outcome = self.call(
            |globals| {
                invoke(
                    "create_branch",
                    self.library.symbols().branch_create,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        outcome.branch_created.ok_or_else(|| Error::Malformed {
            operation: "create_branch",
            message: "branch create event was missing".to_string(),
        })
    }

    /// Switch the working tree to `name`. Local modifications are left in place.
    pub fn switch_branch(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(Error::InvalidArgument {
                field: "name",
                message: "branch name must not be empty".to_string(),
            });
        }
        let path = self.repository_text()?;
        let name = Text::new("name", name)?;
        let revision = Text::new("revision", "")?;
        let mut args = BranchSwitchArgs::empty();
        args.branch = name.as_ffi();
        args.revision = revision.as_ffi();
        self.call(
            |globals| {
                invoke(
                    "switch_branch",
                    self.library.symbols().branch_switch,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        Ok(())
    }

    /// Materialize one committed file to `output`.
    ///
    /// Lore writes the bytes to that path. This method does not read them back,
    /// so the file is not buffered in the SDK. `revision` is a revision hash or
    /// number; an empty string asks Lore for its default revision.
    pub fn write_file(
        &self,
        repo_path: &str,
        revision: &str,
        output: impl AsRef<Path>,
    ) -> Result<()> {
        if repo_path.is_empty() {
            return Err(Error::InvalidArgument {
                field: "repo_path",
                message: "path must not be empty".to_string(),
            });
        }
        let output = std::path::absolute(output.as_ref())?;
        let output_text = path_text("output", &output)?;
        let path = self.repository_text()?;
        let repo_path = Text::new("repo_path", repo_path)?;
        let revision = Text::new("revision", revision)?;
        let address = Text::new("address", "")?;
        let written = Text::new("output", &output_text)?;
        let mut args = FileWriteArgs::empty();
        args.address = address.as_ffi();
        args.path = repo_path.as_ffi();
        args.revision = revision.as_ffi();
        args.output = written.as_ffi();
        self.call(
            |globals| {
                invoke(
                    "write_file",
                    self.library.symbols().file_write,
                    globals,
                    &args,
                    self.timeout,
                    self.event_limit,
                )
            },
            &path,
        )?;
        Ok(())
    }

    /// Read a committed file into memory, refusing content larger than `max_bytes`.
    ///
    /// The file is written into a temporary directory. At most `max_bytes + 1`
    /// bytes are then copied into the returned buffer. The directory is removed
    /// when this method returns. A timeout does not cancel Lore's write, so
    /// removal can fail while that write still has the file open.
    /// Prefer [`Session::write_file`] when the content should stay on disk.
    pub fn read_file(&self, repo_path: &str, revision: &str, max_bytes: u64) -> Result<Vec<u8>> {
        let dir = tempfile::TempDir::new()?;
        let output = dir.path().join("content");
        self.write_file(repo_path, revision, &output)?;
        let meta = std::fs::symlink_metadata(&output)?;
        if !meta.file_type().is_file() {
            return Err(Error::Malformed {
                operation: "read_file",
                message: "Lore did not write a regular file".to_string(),
            });
        }
        let file = std::fs::File::open(&output)?;
        let mut buffer = Vec::new();
        let limit = max_bytes.saturating_add(1);
        std::io::Read::take(file, limit).read_to_end(&mut buffer)?;
        if buffer.len() as u64 > max_bytes {
            return Err(Error::TooLarge {
                limit: max_bytes,
                actual: buffer.len() as u64,
            });
        }
        Ok(buffer)
    }

    fn prepare_directory(&self) -> Result<String> {
        if self.repository.as_os_str().is_empty() {
            return Err(Error::InvalidArgument {
                field: "repository",
                message: "path must not be empty".to_string(),
            });
        }
        std::fs::create_dir_all(&self.repository)?;
        self.repository_text()
    }

    fn repository_text(&self) -> Result<String> {
        if self.repository.as_os_str().is_empty() {
            return Err(Error::InvalidArgument {
                field: "repository",
                message: "path must not be empty".to_string(),
            });
        }
        path_text("repository", &std::path::absolute(&self.repository)?)
    }

    fn call<F>(&self, call: F, repository: &str) -> Result<crate::ffi::Outcome>
    where
        F: FnOnce(&GlobalArgs) -> Result<crate::ffi::Outcome>,
    {
        self.library.ensure_open()?;
        let repo = Text::new("repository", repository)?;
        let work = Text::new("working_directory", repository)?;
        // Safety: every field is a pointer or an integer. Zero is the C API's
        // empty string and the disabled value for each flag we do not set.
        let mut globals: GlobalArgs = unsafe { std::mem::zeroed() };
        globals.repository_path = repo.as_ffi();
        globals.working_directory = work.as_ffi();
        globals.offline = 1;
        globals.local = 1;
        globals.store_keep_alive = 1;
        call(&globals)
    }
}

fn path_text(field: &'static str, path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| Error::InvalidArgument {
            field,
            message: "path is not valid UTF-8".to_string(),
        })
}

/// Walk parents of `start` and return the first directory that contains a
/// `.lore` directory.
pub fn discover(start: impl AsRef<Path>) -> Result<PathBuf> {
    let start = start.as_ref();
    let mut current = if start.is_file() {
        start.parent().map(Path::to_path_buf)
    } else {
        Some(start.to_path_buf())
    };
    while let Some(dir) = current {
        if dir.join(".lore").is_dir() {
            return Ok(dir);
        }
        current = dir.parent().map(Path::to_path_buf);
    }
    Err(Error::NotARepository {
        start: start.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_stops_at_the_repository_root() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("src").join("pkg");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir(root.path().join(".lore")).unwrap();
        let found = discover(&nested).unwrap();
        assert_eq!(found, root.path());
    }

    #[test]
    fn discover_reports_a_missing_repository() {
        let root = tempfile::tempdir().unwrap();
        let error = discover(root.path()).unwrap_err();
        assert!(matches!(error, Error::NotARepository { .. }));
    }
}
