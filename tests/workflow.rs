// Copyright 2026 tess
// SPDX-License-Identifier: MIT

use std::sync::{Mutex, OnceLock};

use lore_sdk::{discover, FileAction, Library, Session};

static LIBRARY: OnceLock<Library> = OnceLock::new();
static LOCK: Mutex<()> = Mutex::new(());

fn library() -> &'static Library {
    LIBRARY.get_or_init(|| {
        let path = std::env::var("LORE_LIBRARY").unwrap_or_else(|_| {
            panic!("BLOCKED: set LORE_LIBRARY to the Lore 0.10.1 C library");
        });
        Library::open(path).expect("load Lore library")
    })
}

fn session(root: &std::path::Path) -> Session {
    library()
        .session(root)
        .expect("session")
        .timeout(std::time::Duration::from_secs(120))
}

#[test]
fn local_repository_workflow() {
    let _guard = LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    let session = session(&root);

    let created = session.create_repository("sdk-demo").expect("create");
    assert_eq!(created.name, "sdk-demo");
    assert!(
        root.join(".lore").is_dir(),
        "create should write a .lore directory"
    );
    assert_eq!(discover(root.join("nested")).expect("discover"), root);

    let status = session.status().expect("status");
    assert!(!status.branch.is_empty());

    let pattern: Vec<u8> = (0..1024 * 1024).map(|index| (index % 251) as u8).collect();
    let repeated = b"same-bytes".to_vec();
    std::fs::write(root.join("pattern.bin"), &pattern).unwrap();
    std::fs::write(root.join("empty.bin"), b"").unwrap();
    std::fs::write(root.join("one.bin"), &repeated).unwrap();
    std::fs::write(root.join("two.bin"), &repeated).unwrap();

    let dirty = session.status().expect("dirty status");
    let dirty_paths: Vec<_> = dirty.files.iter().map(|file| file.path.as_str()).collect();
    assert!(dirty_paths.iter().any(|path| path.ends_with("pattern.bin")));

    let missing = session.stage(&["missing.bin"]);
    assert!(missing.is_err(), "staging a missing path should fail");

    let staged = session
        .stage(&["pattern.bin", "empty.bin", "one.bin", "two.bin"])
        .expect("stage");
    assert!(staged.iter().any(|entry| entry.action == FileAction::Add));

    session.unstage(&["two.bin"]).expect("unstage");
    let after_unstage = session.status().expect("status after unstage");
    assert!(
        !after_unstage
            .files
            .iter()
            .any(|file| file.path.ends_with("two.bin") && file.staged),
        "two.bin should not stay staged"
    );
    session.stage(&["two.bin"]).expect("stage again");

    let commit = session.commit("add binary files").expect("commit");
    assert!(!commit.revision.is_zero());
    assert!(commit.parent.is_zero());

    let history = session.history(10).expect("history");
    assert!(history
        .iter()
        .any(|entry| entry.revision == commit.revision));

    let round_trip = session
        .read_file("pattern.bin", &commit.revision.to_string(), 1024 * 1024)
        .expect("read pattern");
    assert_eq!(round_trip, pattern);
    let empty = session
        .read_file("empty.bin", &commit.revision.to_string(), 1024)
        .expect("read empty");
    assert!(empty.is_empty());
    let one = session.read_file("one.bin", "", 1024).expect("read one");
    let two = session.read_file("two.bin", "", 1024).expect("read two");
    assert_eq!(one, two);
    assert_eq!(one, repeated);

    let too_big = session.read_file("pattern.bin", "", 16).unwrap_err();
    assert!(matches!(
        too_big,
        lore_sdk::Error::TooLarge { limit: 16, .. }
    ));

    std::fs::write(root.join("pattern.bin"), b"changed").unwrap();
    session.stage(&["pattern.bin"]).expect("stage edit");
    let second = session.commit("edit pattern").expect("second commit");
    let diff = session
        .diff(&commit.revision.to_string(), &second.revision.to_string())
        .expect("diff");
    assert!(diff.iter().any(|entry| entry.path.ends_with("pattern.bin")));
    assert_eq!(session.history(1).expect("history limit").len(), 1);

    let original = session.branches().expect("branches");
    let current = original
        .iter()
        .find(|branch| branch.current)
        .expect("current branch");
    session.create_branch("topic").expect("create branch");
    session.switch_branch("topic").expect("switch");
    let switched = session.branches().expect("branches after switch");
    assert!(switched
        .iter()
        .any(|branch| branch.name == "topic" && branch.current));
    session.switch_branch(&current.name).expect("switch back");
    let restored = session.branches().expect("branches restored");
    assert!(restored
        .iter()
        .any(|branch| branch.name == current.name && branch.current));

    let marker = root.join("pattern.bin");
    assert_eq!(discover(&marker).expect("discover file"), root);

    std::fs::write(root.join("limit-a.bin"), b"a").unwrap();
    std::fs::write(root.join("limit-b.bin"), b"b").unwrap();
    let capped = library().session(&root).expect("session").event_limit(1);
    let limited = capped.stage(&["limit-a.bin", "limit-b.bin"]);
    assert!(matches!(
        limited.unwrap_err(),
        lore_sdk::Error::EventLimit { .. }
    ));
}

#[test]
fn status_outside_a_repository_fails() {
    let _guard = LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let dir = tempfile::tempdir().expect("tempdir");
    let error = library()
        .session(dir.path())
        .expect("session")
        .status()
        .unwrap_err();
    assert!(matches!(error, lore_sdk::Error::Failed { .. }));
}

#[test]
fn empty_inputs_fail_before_the_library_runs_them() {
    let _guard = LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let dir = tempfile::tempdir().expect("tempdir");
    let session = library().session(dir.path()).expect("session");
    assert!(matches!(
        session.stage(&[]).unwrap_err(),
        lore_sdk::Error::InvalidArgument { field: "paths", .. }
    ));
    assert!(matches!(
        session.unstage(&[]).unwrap_err(),
        lore_sdk::Error::InvalidArgument { field: "paths", .. }
    ));
    assert!(matches!(
        session.history(0).unwrap_err(),
        lore_sdk::Error::InvalidArgument { field: "limit", .. }
    ));
    assert!(matches!(
        session.create_branch("").unwrap_err(),
        lore_sdk::Error::InvalidArgument { field: "name", .. }
    ));
    assert!(matches!(
        session.switch_branch("").unwrap_err(),
        lore_sdk::Error::InvalidArgument { field: "name", .. }
    ));
    assert!(library().session("").is_err());
}

#[test]
fn concurrent_status_calls_share_one_library() {
    let _guard = LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    let session = library().session(&root).expect("session");
    session.create_repository("concurrent").expect("create");
    std::fs::write(root.join("a.txt"), b"a").unwrap();
    session.stage(&["a.txt"]).unwrap();
    session.commit("a").unwrap();

    let left = library().session(&root).expect("left");
    let right = library().session(&root).expect("right");
    let (a, b) = std::thread::scope(|scope| {
        let first = scope.spawn(|| left.status());
        let second = scope.spawn(|| right.status());
        (first.join().unwrap(), second.join().unwrap())
    });
    assert!(a.is_ok(), "{a:?}");
    assert!(b.is_ok(), "{b:?}");
}
