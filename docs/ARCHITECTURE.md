# Architecture

## Decision

The SDK loads Lore's published C library (`lore.dll`, `liblore.so`, or `liblore.dylib`) and calls the asynchronous entry points, then waits for the `END` event.

That is the same library the JavaScript, Python, C#, and Go SDKs bind. Lore's repository also contains a Rust crate named `lore`. It is the implementation of the system, it is not published, and its public surface is the C-oriented argument structs. Depending on that workspace would mean building Lore itself and tracking unpublished internal crates. A subprocess wrapper around the `lore` CLI would drop structured events and the completion error detail.

This crate does not belong in `EpicGames/lore`. Lore's contributing guide asks for an issue before non-trivial work, and a Lore Enhancement Proposal before a new public language binding. The in-tree Rust API already exists. A second binding inside that repository would duplicate it.

## Shape

`Library` loads the dynamic library once per process, under a mutex, and checks `lore_version` against interface 0.10.1. A second path is rejected. `Session` owns an absolute repository path, a timeout, and an event cap.

Calls are synchronous. The C functions are callback-based, not Rust futures, so there is no Tokio dependency. Each call passes `offline = 1`, `local = 1`, and `store_keep_alive = 1`. Token fields stay null. Create also passes `LORE_SHARED_STORE_MODE_DISABLED`. Argument structs are zeroed before their fields are set, so padding is not leftover stack.

Argument structs are `#[repr(C)]` copies of the 0.10.1 header. Their sizes and the ambiguous field offsets were taken from an MSVC build of that header and are asserted in unit tests. The event payload is read at byte offset 8, which is where that header places the union. Strings are copied out before the callback returns. The callback does not panic across the FFI boundary.

The slot that collects events is reference-counted. The waiter holds one reference. A second reference stays until the `END` event, including when the waiter has already returned a timeout. There is no cancel function in the header, so a timed-out call is not aborted.

`read_file` is the only path that buffers file bytes, and it does so under a caller-supplied limit. `write_file` lets Lore write the file.

## What is private

Symbol loading, C structs, and the callback trampoline are crate-private. Callers see domain types (`RevisionId`, `Status`, `Commit`, and so on) and `Error`.
