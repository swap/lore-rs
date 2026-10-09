// Copyright 2026 tess
// SPDX-License-Identifier: MIT

use lore_sdk::{Error, Library};

#[test]
fn shutdown_rejects_later_calls() {
    let path = std::env::var("LORE_LIBRARY").unwrap_or_else(|_| {
        panic!("BLOCKED: set LORE_LIBRARY to the Lore 0.10.1 C library");
    });
    let library = Library::open(path).expect("load");
    let version = library.version().expect("version");
    assert!(lore_sdk::interface_compatible(&version));
    let session = library.session(std::env::temp_dir()).expect("session");
    library.shutdown().expect("shutdown");
    assert!(matches!(library.version().unwrap_err(), Error::ShutDown));
    assert!(matches!(session.status().unwrap_err(), Error::ShutDown));
    library.shutdown().expect("second shutdown is a no-op");
}
