// Copyright 2026 tess
// SPDX-License-Identifier: MIT

use std::fmt;

/// 32-byte content or revision hash, as Lore returns it.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RevisionId([u8; 32]);

impl RevisionId {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == [0; 32]
    }

    pub(crate) fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for RevisionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for RevisionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RevisionId({self})")
    }
}

/// 16-byte branch or repository identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId([u8; 16]);

impl ObjectId {
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub(crate) fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectId({self})")
    }
}

/// Change recorded against a file.
///
/// `Other` preserves a code this crate does not name yet, so a newer library
/// that still reports interface 0.10.1 cannot be misread as "keep".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAction {
    Keep,
    Add,
    Delete,
    Move,
    Copy,
    Other(i32),
}

impl FileAction {
    pub(crate) fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Keep,
            1 => Self::Add,
            2 => Self::Delete,
            3 => Self::Move,
            4 => Self::Copy,
            other => Self::Other(other),
        }
    }
}

/// Kind of node reported by status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Directory,
    File,
    Link,
    Other(i32),
}

impl NodeKind {
    pub(crate) fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Directory,
            1 => Self::File,
            2 => Self::Link,
            other => Self::Other(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchLocation {
    Local,
    Remote,
    Other(i32),
}

impl BranchLocation {
    pub(crate) fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Local,
            1 => Self::Remote,
            other => Self::Other(other),
        }
    }
}

/// Repository named by a successful create call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryCreated {
    pub id: ObjectId,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusFile {
    pub path: String,
    pub from_path: String,
    pub size: u64,
    pub action: FileAction,
    pub kind: NodeKind,
    pub staged: bool,
    pub dirty: bool,
    pub conflict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub branch: String,
    pub revision: RevisionId,
    pub revision_number: u64,
    pub files: Vec<StatusFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedPath {
    pub path: String,
    pub from_path: String,
    pub action: FileAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub revision: RevisionId,
    pub number: u64,
    pub parent: RevisionId,
    pub merge_parent: RevisionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub revision: RevisionId,
    pub number: u64,
    pub parent: RevisionId,
    pub merge_parent: RevisionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub name: String,
    pub category: String,
    pub id: ObjectId,
    pub latest: RevisionId,
    pub location: BranchLocation,
    pub current: bool,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchCreated {
    pub name: String,
    pub latest: RevisionId,
    pub committed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffEntry {
    pub path: String,
    pub from_path: String,
    pub action: FileAction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_display_is_lowercase_hex() {
        let mut bytes = [0u8; 32];
        bytes[0] = 0x0a;
        bytes[31] = 0xff;
        let id = RevisionId::from_bytes(bytes);
        let text = id.to_string();
        assert_eq!(text.len(), 64);
        assert!(text.starts_with("0a"));
        assert!(text.ends_with("ff"));
        assert!(RevisionId::from_bytes([0; 32]).is_zero());
    }

    #[test]
    fn unknown_file_action_is_preserved() {
        assert_eq!(FileAction::from_code(1), FileAction::Add);
        assert_eq!(FileAction::from_code(9), FileAction::Other(9));
    }
}
