// Copyright 2026 tess
// SPDX-License-Identifier: MIT

//! Community Rust SDK for [Lore](https://github.com/EpicGames/lore).
//!
//! Lore ships its supported external interface as a C library. This crate
//! loads that library and exposes a smaller, synchronous API for local
//! repository work: create, status, stage, unstage, commit, history, diff,
//! branches, and reading committed file bytes.
//!
//! It is not an Epic Games project, and it does not wrap the unpublished
//! `lore` crate inside the Lore repository. Struct layouts match Lore C API
//! **0.10.1** only.
//!
//! # Library lifetime
//!
//! Load the dynamic library once with [`Library::open`] or [`Library::from_env`].
//! [`Library::shutdown`] stops Lore's threads and is irreversible in the
//! process. Dropping a [`Library`] does not shut it down.
//!
//! # Example
//!
//! ```no_run
//! use lore_sdk::Library;
//!
//! # fn example() -> lore_sdk::Result<()> {
//! let library = Library::from_env()?;
//! let session = library.session("demo")?;
//! session.create_repository("demo")?;
//! std::fs::write("demo/notes.txt", "hello")?;
//! session.stage(&["notes.txt"])?;
//! let commit = session.commit("add notes")?;
//! println!("{commit:?}");
//! library.shutdown()?;
//! # Ok(())
//! # }
//! ```

mod error;
mod ffi;
mod library;
mod model;
mod session;

pub use error::{interface_compatible, Error, Result, SUPPORTED_INTERFACE};
pub use library::Library;
pub use model::{
    Branch, BranchCreated, BranchLocation, Commit, DiffEntry, FileAction, HistoryEntry, NodeKind,
    ObjectId, RepositoryCreated, RevisionId, StagedPath, Status, StatusFile,
};
pub use session::{discover, Session, StatusOptions};
