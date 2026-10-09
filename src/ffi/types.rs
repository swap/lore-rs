// Copyright 2026 tess
// SPDX-License-Identifier: MIT

//! Layouts measured from `lore.h` in the Lore 0.10.1 Windows x64 library.
//!
//! Sizes were printed by an MSVC compilation of that header (`sizeof` /
//! `offsetof`). Each struct has a unit test that checks `size_of` against
//! those numbers. A mismatch means the Rust declaration drifted from the ABI
//! this crate claims to speak.

use std::os::raw::c_void;

pub const PAYLOAD_OFFSET: usize = 8;
pub const EVENT_LIMIT_DEFAULT: usize = 100_000;
pub const MAX_COPIED_STRING: usize = 1024 * 1024;

pub const TAG_COMPLETE: u32 = 2;
pub const TAG_END: u32 = 5;
pub const TAG_BRANCH_CREATE: u32 = 11;
pub const TAG_BRANCH_LIST_ENTRY: u32 = 15;
pub const TAG_FILE_STAGE_FILE: u32 = 107;
pub const TAG_REPOSITORY_CREATE: u32 = 133;
pub const TAG_REPOSITORY_STATUS_REVISION: u32 = 153;
pub const TAG_REPOSITORY_STATUS_FILE: u32 = 154;
pub const TAG_REVISION_COMMIT_REVISION: u32 = 161;
pub const TAG_REVISION_DIFF_FILE: u32 = 164;
pub const TAG_REVISION_HISTORY_ENTRY: u32 = 167;

pub const VFS_NONE: i32 = 0;
pub const SHARED_STORE_DISABLED: i32 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LoreString {
    pub ptr: *const u8,
    pub length: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LoreStringArray {
    pub ptr: *const LoreString,
    pub count: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CallbackConfig {
    pub user_context: u64,
    pub func: Option<extern "C" fn(*const u8, u64)>,
}

#[repr(C)]
pub struct GlobalArgs {
    pub repository_path: LoreString,
    pub working_directory: LoreString,
    pub correlation_id: LoreString,
    pub identity: LoreString,
    pub force: u8,
    pub offline: u8,
    pub local: u8,
    pub remote: u8,
    pub dry_run: u8,
    pub max_connections: u32,
    pub search_limit: u32,
    pub search_nearest: u8,
    pub no_gc: u8,
    pub in_memory: u8,
    pub file_count_limit: u64,
    pub file_size_limit: u64,
    pub compress_task_limit: u64,
    pub store_keep_alive: u8,
    pub store_keep_alive_seconds: u64,
    pub sync_data: u8,
    pub cache: u8,
    pub identity_token: LoreString,
    pub access_token: LoreString,
    pub stats: u32,
    pub event_interval_ms: u64,
}

#[repr(C)]
pub struct RepositoryCreateArgs {
    pub repository_url: LoreString,
    pub description: LoreString,
    pub id: LoreString,
    pub vfs: i32,
    pub use_shared_store: i32,
    pub shared_store_path: LoreString,
}

#[repr(C)]
pub struct RepositoryStatusArgs {
    pub staged: u8,
    pub scan: u8,
    pub check_dirty: u8,
    pub reset: u8,
    pub sync_point: u8,
    pub revision_only: u8,
    pub count: u8,
    pub paths: LoreStringArray,
}

#[repr(C)]
pub struct FileStageArgs {
    pub paths: LoreStringArray,
    pub case_change: u32,
    pub scan: u8,
}

#[repr(C)]
pub struct FileUnstageArgs {
    pub paths: LoreStringArray,
}

#[repr(C)]
pub struct FileWriteArgs {
    pub address: LoreString,
    pub path: LoreString,
    pub revision: LoreString,
    pub output: LoreString,
}

#[repr(C)]
pub struct RevisionCommitArgs {
    pub message: LoreString,
    pub link: LoreString,
    pub link_paths: LoreStringArray,
    pub link_messages: LoreStringArray,
    pub layer: LoreString,
    pub layer_paths: LoreStringArray,
    pub layer_messages: LoreStringArray,
}

#[repr(C)]
pub struct RevisionHistoryArgs {
    pub revision: LoreString,
    pub branch: LoreString,
    pub date: u64,
    pub length: u32,
    pub only_branch: u8,
}

#[repr(C)]
pub struct RevisionDiffArgs {
    pub revision_source: LoreString,
    pub revision_target: LoreString,
    pub paths: LoreStringArray,
}

#[repr(C)]
pub struct BranchListArgs {
    pub archived: u8,
}

#[repr(C)]
pub struct BranchCreateArgs {
    pub branch: LoreString,
    pub category: LoreString,
    pub id: LoreString,
}

#[repr(C)]
pub struct BranchSwitchArgs {
    pub branch: LoreString,
    pub revision: LoreString,
    pub reset: u8,
    pub bare: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CompleteEvent {
    pub status: i32,
    _pad_status: u32,
    pub error_code: i32,
    _pad_error: u32,
    pub message: LoreString,
    _trace_ptr: *const c_void,
    _trace_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RepositoryCreateEvent {
    pub id: [u8; 16],
    pub name: LoreString,
    pub path: LoreString,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BranchListEntryEvent {
    pub location: i32,
    pub id: [u8; 16],
    pub name: LoreString,
    pub category: LoreString,
    pub latest: [u8; 32],
    _stack_ptr: *const c_void,
    _stack_count: usize,
    pub creator: LoreString,
    pub created: u64,
    pub is_current: u8,
    pub archived: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BranchCreateEvent {
    pub name: LoreString,
    pub latest: [u8; 32],
    pub is_commit: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FileStageFileEvent {
    pub from_path: LoreString,
    pub path: LoreString,
    pub action: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StatusRevisionEvent {
    pub repository: [u8; 16],
    pub branch: [u8; 16],
    pub branch_name: LoreString,
    pub revision: [u8; 32],
    pub revision_number: u64,
    pub revision_staged: [u8; 32],
    pub revision_merged: [u8; 32],
    pub revision_merged_parent_branch: [u8; 32],
    pub revision_local: [u8; 32],
    pub revision_local_number: u64,
    pub revision_remote: [u8; 32],
    pub revision_remote_number: u64,
    pub is_local_ahead: u8,
    pub is_remote_ahead: u8,
    pub remote_available: u8,
    pub remote_authorized: u8,
    pub remote_branch_exist: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StatusFileEvent {
    pub path: LoreString,
    pub size: u64,
    pub action: i32,
    pub kind: i32,
    pub flag_staged: u8,
    pub flag_merged: u8,
    pub flag_conflict: u8,
    pub flag_conflict_unresolved: u8,
    pub flag_conflict_automerged: u8,
    pub flag_conflict_mine: u8,
    pub flag_conflict_theirs: u8,
    pub flag_dirty: u8,
    pub from_path: LoreString,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CommitRevisionEvent {
    pub repository: [u8; 16],
    pub branch: [u8; 16],
    pub revision: [u8; 32],
    pub revision_number: u64,
    pub parent: [u8; 32],
    pub parent_other: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HistoryEntryEvent {
    pub revision: [u8; 32],
    pub revision_number: u64,
    pub parent: [[u8; 32]; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DiffFileEvent {
    pub path: LoreString,
    pub action: i32,
    pub old_is_file: u8,
    pub new_is_file: u8,
    pub old_address: [u8; 48],
    pub new_address: [u8; 48],
    pub from_path: LoreString,
}

macro_rules! zero_default {
    ($name:ident) => {
        impl $name {
            pub(crate) fn empty() -> Self {
                // Safety: every field is a pointer or an integer. Zero is the
                // empty string and the disabled flag. Padding is zero, which
                // matters because Lore copies the whole struct.
                unsafe { std::mem::zeroed() }
            }
        }
    };
}

zero_default!(RepositoryCreateArgs);
zero_default!(RepositoryStatusArgs);
zero_default!(FileStageArgs);
zero_default!(FileUnstageArgs);
zero_default!(FileWriteArgs);
zero_default!(RevisionCommitArgs);
zero_default!(RevisionHistoryArgs);
zero_default!(RevisionDiffArgs);
zero_default!(BranchListArgs);
zero_default!(BranchCreateArgs);
zero_default!(BranchSwitchArgs);

pub type CreateAsync =
    unsafe extern "C" fn(*const GlobalArgs, *const RepositoryCreateArgs, CallbackConfig);
pub type StatusAsync =
    unsafe extern "C" fn(*const GlobalArgs, *const RepositoryStatusArgs, CallbackConfig);
pub type StageAsync = unsafe extern "C" fn(*const GlobalArgs, *const FileStageArgs, CallbackConfig);
pub type UnstageAsync =
    unsafe extern "C" fn(*const GlobalArgs, *const FileUnstageArgs, CallbackConfig);
pub type WriteAsync = unsafe extern "C" fn(*const GlobalArgs, *const FileWriteArgs, CallbackConfig);
pub type CommitAsync =
    unsafe extern "C" fn(*const GlobalArgs, *const RevisionCommitArgs, CallbackConfig);
pub type HistoryAsync =
    unsafe extern "C" fn(*const GlobalArgs, *const RevisionHistoryArgs, CallbackConfig);
pub type DiffAsync =
    unsafe extern "C" fn(*const GlobalArgs, *const RevisionDiffArgs, CallbackConfig);
pub type BranchListAsync =
    unsafe extern "C" fn(*const GlobalArgs, *const BranchListArgs, CallbackConfig);
pub type BranchCreateAsync =
    unsafe extern "C" fn(*const GlobalArgs, *const BranchCreateArgs, CallbackConfig);
pub type BranchSwitchAsync =
    unsafe extern "C" fn(*const GlobalArgs, *const BranchSwitchArgs, CallbackConfig);
pub type VersionFn = unsafe extern "C" fn() -> *const u8;
pub type ShutdownFn = unsafe extern "C" fn() -> i32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_offsets_match_lore_0_10_1_header() {
        assert_eq!(std::mem::offset_of!(GlobalArgs, offline), 65);
        assert_eq!(std::mem::offset_of!(GlobalArgs, max_connections), 72);
        assert_eq!(std::mem::offset_of!(GlobalArgs, file_count_limit), 88);
        assert_eq!(std::mem::offset_of!(GlobalArgs, identity_token), 136);
        assert_eq!(std::mem::offset_of!(GlobalArgs, event_interval_ms), 176);
        assert_eq!(std::mem::offset_of!(RepositoryStatusArgs, paths), 8);
        assert_eq!(std::mem::offset_of!(FileStageArgs, case_change), 16);
        assert_eq!(std::mem::offset_of!(RevisionHistoryArgs, date), 32);
        assert_eq!(std::mem::offset_of!(RevisionHistoryArgs, length), 40);
        assert_eq!(std::mem::offset_of!(CompleteEvent, message), 16);
        assert_eq!(std::mem::offset_of!(BranchListEntryEvent, id), 4);
        assert_eq!(std::mem::offset_of!(BranchListEntryEvent, name), 24);
        assert_eq!(std::mem::offset_of!(BranchListEntryEvent, is_current), 128);
        assert_eq!(std::mem::offset_of!(StatusFileEvent, from_path), 40);
        assert_eq!(std::mem::offset_of!(DiffFileEvent, old_address), 22);
        assert_eq!(std::mem::offset_of!(DiffFileEvent, from_path), 120);
    }

    #[test]
    fn sizes_match_lore_0_10_1_header() {
        assert_eq!(std::mem::size_of::<LoreString>(), 16);
        assert_eq!(std::mem::size_of::<LoreStringArray>(), 16);
        assert_eq!(std::mem::size_of::<CallbackConfig>(), 16);
        assert_eq!(std::mem::size_of::<GlobalArgs>(), 184);
        assert_eq!(std::mem::size_of::<RepositoryCreateArgs>(), 72);
        assert_eq!(std::mem::size_of::<RepositoryStatusArgs>(), 24);
        assert_eq!(std::mem::size_of::<FileStageArgs>(), 24);
        assert_eq!(std::mem::size_of::<FileUnstageArgs>(), 16);
        assert_eq!(std::mem::size_of::<FileWriteArgs>(), 64);
        assert_eq!(std::mem::size_of::<RevisionCommitArgs>(), 112);
        assert_eq!(std::mem::size_of::<RevisionHistoryArgs>(), 48);
        assert_eq!(std::mem::size_of::<RevisionDiffArgs>(), 48);
        assert_eq!(std::mem::size_of::<BranchListArgs>(), 1);
        assert_eq!(std::mem::size_of::<BranchCreateArgs>(), 48);
        assert_eq!(std::mem::size_of::<BranchSwitchArgs>(), 40);
        assert_eq!(std::mem::size_of::<CompleteEvent>(), 48);
        assert_eq!(std::mem::size_of::<RepositoryCreateEvent>(), 48);
        assert_eq!(std::mem::size_of::<BranchListEntryEvent>(), 136);
        assert_eq!(std::mem::size_of::<BranchCreateEvent>(), 56);
        assert_eq!(std::mem::size_of::<FileStageFileEvent>(), 40);
        assert_eq!(std::mem::size_of::<StatusRevisionEvent>(), 272);
        assert_eq!(std::mem::size_of::<StatusFileEvent>(), 56);
        assert_eq!(std::mem::size_of::<CommitRevisionEvent>(), 136);
        assert_eq!(std::mem::size_of::<HistoryEntryEvent>(), 104);
        assert_eq!(std::mem::size_of::<DiffFileEvent>(), 136);
    }
}
