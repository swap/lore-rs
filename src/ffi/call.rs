// Copyright 2026 tess
// SPDX-License-Identifier: MIT

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use super::text::copy_text;
use super::types::{
    BranchCreateEvent, BranchListEntryEvent, CallbackConfig, CommitRevisionEvent, CompleteEvent,
    DiffFileEvent, FileStageFileEvent, HistoryEntryEvent, RepositoryCreateEvent, StatusFileEvent,
    StatusRevisionEvent, PAYLOAD_OFFSET, TAG_BRANCH_CREATE, TAG_BRANCH_LIST_ENTRY, TAG_COMPLETE,
    TAG_END, TAG_FILE_STAGE_FILE, TAG_REPOSITORY_CREATE, TAG_REPOSITORY_STATUS_FILE,
    TAG_REPOSITORY_STATUS_REVISION, TAG_REVISION_COMMIT_REVISION, TAG_REVISION_DIFF_FILE,
    TAG_REVISION_HISTORY_ENTRY,
};
use crate::error::{Error, Result};
use crate::model::{
    Branch, BranchCreated, BranchLocation, Commit, DiffEntry, FileAction, HistoryEntry, NodeKind,
    ObjectId, RepositoryCreated, RevisionId, StagedPath, Status, StatusFile,
};

#[derive(Default)]
struct SlotState {
    created: Option<RepositoryCreated>,
    status_revision: Option<StatusRevision>,
    status_files: Vec<StatusFile>,
    staged: Vec<StagedPath>,
    commit: Option<Commit>,
    history: Vec<HistoryEntry>,
    branches: Vec<Branch>,
    branch_created: Option<BranchCreated>,
    diff: Vec<DiffEntry>,
    completion: Option<Completion>,
    ended: bool,
    truncated: bool,
    malformed: Option<String>,
}

struct StatusRevision {
    branch: String,
    revision: RevisionId,
    revision_number: u64,
}

struct Completion {
    status: i32,
    message: String,
}

struct CallSlot {
    state: Mutex<SlotState>,
    done: Condvar,
    limit: usize,
}

pub struct Outcome {
    pub created: Option<RepositoryCreated>,
    pub status: Option<Status>,
    pub staged: Vec<StagedPath>,
    pub commit: Option<Commit>,
    pub history: Vec<HistoryEntry>,
    pub branches: Vec<Branch>,
    pub branch_created: Option<BranchCreated>,
    pub diff: Vec<DiffEntry>,
}

impl CallSlot {
    fn record(&self, event: *const u8) {
        if event.is_null() {
            return;
        }
        // Safety: `lore_event_t.tag` is a u32 at offset 0. The payload union
        // starts at offset 8 for interface 0.10.1. The pointer is valid until
        // this callback returns, and events for one call are not concurrent.
        let tag = unsafe { event.cast::<u32>().read() };
        let payload = unsafe { event.add(PAYLOAD_OFFSET) };
        let mut state = self.state.lock().unwrap_or_else(|err| err.into_inner());
        if state.truncated && tag != TAG_COMPLETE && tag != TAG_END {
            return;
        }
        if tag != TAG_COMPLETE && tag != TAG_END && self.over_limit(&state) {
            state.truncated = true;
            return;
        }
        if let Err(error) = self.store(&mut state, tag, payload) {
            if state.malformed.is_none() {
                state.malformed = Some(error.to_string());
            }
        }
        if tag == TAG_END {
            state.ended = true;
            self.done.notify_all();
        }
    }

    fn over_limit(&self, state: &SlotState) -> bool {
        let used = state.status_files.len()
            + state.staged.len()
            + state.history.len()
            + state.branches.len()
            + state.diff.len();
        used >= self.limit
    }

    fn store(&self, state: &mut SlotState, tag: u32, payload: *const u8) -> Result<()> {
        match tag {
            TAG_COMPLETE => {
                let event = unsafe { read_pod::<CompleteEvent>(payload) };
                let message = if event.status == 0 {
                    String::new()
                } else {
                    unsafe { copy_text(event.message) }
                        .unwrap_or_else(|_| format!("error {}", event.error_code))
                };
                state.completion = Some(Completion {
                    status: event.status,
                    message,
                });
            }
            TAG_REPOSITORY_CREATE => {
                let event = unsafe { read_pod::<RepositoryCreateEvent>(payload) };
                state.created = Some(RepositoryCreated {
                    id: ObjectId::from_bytes(event.id),
                    name: unsafe { copy_text(event.name) }?,
                    path: unsafe { copy_text(event.path) }?,
                });
            }
            TAG_REPOSITORY_STATUS_REVISION => {
                let event = unsafe { read_pod::<StatusRevisionEvent>(payload) };
                state.status_revision = Some(StatusRevision {
                    branch: unsafe { copy_text(event.branch_name) }?,
                    revision: RevisionId::from_bytes(event.revision),
                    revision_number: event.revision_number,
                });
            }
            TAG_REPOSITORY_STATUS_FILE => {
                let event = unsafe { read_pod::<StatusFileEvent>(payload) };
                state.status_files.push(StatusFile {
                    path: unsafe { copy_text(event.path) }?,
                    from_path: unsafe { copy_text(event.from_path) }?,
                    size: event.size,
                    action: FileAction::from_code(event.action),
                    kind: NodeKind::from_code(event.kind),
                    staged: event.flag_staged != 0,
                    dirty: event.flag_dirty != 0,
                    conflict: event.flag_conflict != 0,
                });
            }
            TAG_FILE_STAGE_FILE => {
                let event = unsafe { read_pod::<FileStageFileEvent>(payload) };
                state.staged.push(StagedPath {
                    path: unsafe { copy_text(event.path) }?,
                    from_path: unsafe { copy_text(event.from_path) }?,
                    action: FileAction::from_code(event.action),
                });
            }
            TAG_REVISION_COMMIT_REVISION => {
                let event = unsafe { read_pod::<CommitRevisionEvent>(payload) };
                state.commit = Some(Commit {
                    revision: RevisionId::from_bytes(event.revision),
                    number: event.revision_number,
                    parent: RevisionId::from_bytes(event.parent),
                    merge_parent: RevisionId::from_bytes(event.parent_other),
                });
            }
            TAG_REVISION_HISTORY_ENTRY => {
                let event = unsafe { read_pod::<HistoryEntryEvent>(payload) };
                state.history.push(HistoryEntry {
                    revision: RevisionId::from_bytes(event.revision),
                    number: event.revision_number,
                    parent: RevisionId::from_bytes(event.parent[0]),
                    merge_parent: RevisionId::from_bytes(event.parent[1]),
                });
            }
            TAG_BRANCH_LIST_ENTRY => {
                let event = unsafe { read_pod::<BranchListEntryEvent>(payload) };
                state.branches.push(Branch {
                    name: unsafe { copy_text(event.name) }?,
                    category: unsafe { copy_text(event.category) }?,
                    id: ObjectId::from_bytes(event.id),
                    latest: RevisionId::from_bytes(event.latest),
                    location: BranchLocation::from_code(event.location),
                    current: event.is_current != 0,
                    archived: event.archived != 0,
                });
            }
            TAG_BRANCH_CREATE => {
                let event = unsafe { read_pod::<BranchCreateEvent>(payload) };
                state.branch_created = Some(BranchCreated {
                    name: unsafe { copy_text(event.name) }?,
                    latest: RevisionId::from_bytes(event.latest),
                    committed: event.is_commit != 0,
                });
            }
            TAG_REVISION_DIFF_FILE => {
                let event = unsafe { read_pod::<DiffFileEvent>(payload) };
                state.diff.push(DiffEntry {
                    path: unsafe { copy_text(event.path) }?,
                    from_path: unsafe { copy_text(event.from_path) }?,
                    action: FileAction::from_code(event.action),
                });
            }
            _ => {}
        }
        Ok(())
    }
}

/// # Safety
///
/// `payload` must point at a live `T` for interface 0.10.1, aligned for `T`.
unsafe fn read_pod<T: Copy>(payload: *const u8) -> T {
    unsafe { payload.cast::<T>().read() }
}

extern "C" fn on_event(event: *const u8, user: u64) {
    let ptr = user as *const CallSlot;
    if ptr.is_null() {
        return;
    }
    // The waiter holds one `Arc`. A second reference stays alive until `END`,
    // which is the last event for this call. Taking it on every event and
    // putting it back unless this is `END` keeps the slot allocated if the
    // waiter times out first.
    let library_ref = unsafe { Arc::from_raw(ptr) };
    let slot = Arc::clone(&library_ref);
    let is_end = !event.is_null() && unsafe { event.cast::<u32>().read() } == TAG_END;
    if !is_end {
        std::mem::forget(library_ref);
    }
    let _ = catch_unwind(AssertUnwindSafe(|| slot.record(event)));
}

pub fn invoke<T>(
    operation: &'static str,
    func: unsafe extern "C" fn(*const super::types::GlobalArgs, *const T, CallbackConfig),
    globals: &super::types::GlobalArgs,
    args: &T,
    timeout: Duration,
    limit: usize,
) -> Result<Outcome> {
    if timeout.is_zero() {
        return Err(Error::InvalidArgument {
            field: "timeout",
            message: "must be greater than zero".to_string(),
        });
    }
    let slot = Arc::new(CallSlot {
        state: Mutex::new(SlotState::default()),
        done: Condvar::new(),
        limit,
    });
    let library_ref = Arc::into_raw(Arc::clone(&slot));
    let callback = CallbackConfig {
        user_context: library_ref as u64,
        func: Some(on_event),
    };
    // Safety: `globals` and `args` outlive this function, which does not
    // return until the async entry point has copied them (it returns before
    // the work finishes) and we have waited. The function pointer is a symbol
    // from the loaded Lore library.
    unsafe { func(globals, args, callback) };

    let state = slot.state.lock().unwrap_or_else(|err| err.into_inner());
    let (state, wait) = slot
        .done
        .wait_timeout_while(state, timeout, |state| !state.ended)
        .unwrap_or_else(|err| err.into_inner());
    if wait.timed_out() && !state.ended {
        return Err(Error::Timeout { operation, timeout });
    }
    if state.truncated {
        return Err(Error::EventLimit { operation, limit });
    }
    if let Some(message) = &state.malformed {
        return Err(Error::Malformed {
            operation,
            message: message.clone(),
        });
    }
    let completion = state.completion.as_ref().ok_or_else(|| Error::Malformed {
        operation,
        message: "completion event was missing".to_string(),
    })?;
    if completion.status != 0 {
        return Err(Error::Failed {
            operation,
            code: completion.status,
            message: completion.message.clone(),
        });
    }
    let status = state.status_revision.as_ref().map(|revision| Status {
        branch: revision.branch.clone(),
        revision: revision.revision,
        revision_number: revision.revision_number,
        files: state.status_files.clone(),
    });
    Ok(Outcome {
        created: state.created.clone(),
        status,
        staged: state.staged.clone(),
        commit: state.commit.clone(),
        history: state.history.clone(),
        branches: state.branches.clone(),
        branch_created: state.branch_created.clone(),
        diff: state.diff.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_does_not_free_the_slot_before_end() {
        let slot = Arc::new(CallSlot {
            state: Mutex::new(SlotState::default()),
            done: Condvar::new(),
            limit: 10,
        });
        let library_ref = Arc::into_raw(Arc::clone(&slot));
        let state = slot.state.lock().unwrap();
        let (state, wait) = slot
            .done
            .wait_timeout_while(state, Duration::from_millis(20), |state| !state.ended)
            .unwrap();
        assert!(wait.timed_out());
        assert!(!state.ended);
        drop(state);
        // The library reference is still outstanding. Delivering END must be
        // able to use it, and must drop that reference.
        let mut raw = [0u8; 8];
        raw[..4].copy_from_slice(&TAG_END.to_ne_bytes());
        on_event(raw.as_ptr(), library_ref as u64);
        assert!(slot.state.lock().unwrap().ended);
        assert_eq!(Arc::strong_count(&slot), 1);
    }

    unsafe extern "C" fn never_called(
        _globals: *const crate::ffi::types::GlobalArgs,
        _args: *const u8,
        _callback: CallbackConfig,
    ) {
        panic!("the library function ran");
    }

    unsafe extern "C" fn finish_after_timeout(
        _globals: *const crate::ffi::types::GlobalArgs,
        _args: *const u8,
        callback: CallbackConfig,
    ) {
        let user = callback.user_context;
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(60));
            let ignored = [0u8; 16];
            on_event(ignored.as_ptr(), user);
            let mut end = [0u8; 16];
            end[..4].copy_from_slice(&TAG_END.to_ne_bytes());
            on_event(end.as_ptr(), user);
        });
    }

    #[test]
    fn zero_timeout_does_not_call_the_library() {
        let globals = unsafe { std::mem::zeroed() };
        let args = 0u8;
        let error = match invoke("op", never_called, &globals, &args, Duration::ZERO, 8) {
            Ok(_) => panic!("zero timeout must fail before the call"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            Error::InvalidArgument {
                field: "timeout",
                ..
            }
        ));
    }

    #[test]
    fn timeout_returns_and_a_later_event_still_has_the_slot() {
        let globals = unsafe { std::mem::zeroed() };
        let args = 0u8;
        let error = match invoke(
            "op",
            finish_after_timeout,
            &globals,
            &args,
            Duration::from_millis(20),
            8,
        ) {
            Ok(_) => panic!("the wait should time out"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            Error::Timeout {
                operation: "op",
                ..
            }
        ));
        std::thread::sleep(Duration::from_millis(120));
    }
}
