# lore-sdk

Community Rust SDK for [Lore](https://github.com/EpicGames/lore), Epic Games' open source version control system.

This is not an Epic Games project and it is not endorsed by Epic. The official language SDKs are [lore-js](https://github.com/EpicGames/lore-js), [lore-python](https://github.com/EpicGames/lore-python), [lore-dotnet](https://github.com/EpicGames/lore-dotnet), and [lore-go](https://github.com/EpicGames/lore-go). Lore's own repository already contains the Rust implementation of the system. That crate is `publish = false`, so applications cannot depend on it from crates.io. This crate is the small external alternative: it loads the published C library, which Lore documents as the supported interface.

The wrapped interface is **0.10.1**. A library that reports a different version is refused, because the struct layouts in this crate were measured against that header.

## Prerequisites

- Rust 1.85 or newer. The crate's own code would build on 1.79, but the resolved `tempfile` dependency pulls `getrandom` 0.4, which requires 1.85.
- The Lore 0.10.1 C library for your platform, from the [v0.10.1 release](https://github.com/EpicGames/lore/releases/tag/v0.10.1):
  - Windows: `lore.dll`
  - Linux: `liblore.so`
  - macOS: `liblore.dylib`

Point `LORE_LIBRARY` at that file, or point `LORE_BUILD_PATH` at the directory that contains it.

If your user config turns on Lore's background service, set `LORE_USE_SERVICE=0` for the process that loads this crate. Otherwise Lore may relay the call to that service. This crate does not change process environment variables.

## Install

```toml
[dependencies]
lore-sdk = "0.1"
```

The dynamic library is not bundled. It has to be present at runtime.

## Quickstart

```rust
use lore_sdk::Library;

fn main() -> lore_sdk::Result<()> {
    let library = Library::from_env()?;
    let session = library.session("demo")?;
    session.create_repository("demo")?;
    std::fs::write("demo/notes.txt", "hello")?;
    session.stage(&["notes.txt"])?;
    let commit = session.commit("add notes")?;
    println!("{commit:?}");
    library.shutdown()?;
    Ok(())
}
```

`examples/local.rs` prints the status of an existing repository:

```sh
LORE_LIBRARY=/path/to/liblore.so cargo run --example local -- /path/to/repo
```

## What you can do

`Session` talks to one working tree. Every call sets Lore's offline and local flags. `create_repository` also disables the shared store, so a machine-wide shared-store setting is not inherited.

| Operation | Method |
| --- | --- |
| Create a local repository | `Session::create_repository` |
| Find a checkout | `discover` |
| Working tree status | `Session::status` |
| Stage and unstage | `Session::stage`, `Session::unstage` |
| Commit | `Session::commit` |
| History of the current branch | `Session::history` |
| Diff two revisions | `Session::diff` |
| List, create, and switch branches | `Session::branches`, `Session::create_branch`, `Session::switch_branch` |
| Read committed bytes | `Session::write_file`, `Session::read_file` |

`write_file` asks Lore to write the file to a path you choose. The SDK does not read those bytes. `read_file` writes the whole file into a temporary directory, then copies at most `max_bytes + 1` bytes into memory so it can return `Error::TooLarge` without buffering the rest. The temporary directory is removed when the call returns. If the call times out, Lore may still be writing that file; there is no way to cancel it, and deleting the directory can fail while the write is open.

`session` stores an absolute path. A relative path is resolved against the current directory at the moment `session` is called, not on each later operation.

Every call sets `offline`, `local`, and `store_keep_alive`. Create also disables the shared store. Token fields are left empty.

Paths passed into Lore must be UTF-8 and must not contain a NUL byte. Arguments are copied by the library before the call returns.

## Errors

Fallible functions return `lore_sdk::Result`.

- `Error::InvalidArgument` is returned before Lore runs, for an empty path list, an empty branch name, a zero timeout, a NUL byte, or an input longer than 1 MiB.
- `Error::Failed` is a Lore completion status other than zero, with Lore's message. The numeric code is `Complete.status`.
- `Error::Timeout` means this crate stopped waiting. The library call can still be running. If a capped or malformed event stream never reaches `END`, the result is a timeout rather than `EventLimit`.
- `Error::EventLimit` means too many retained events. The call still waits for `END`. Strings are not truncated: a string longer than 1 MiB fails the call with `Error::Malformed`.
- `Error::ShutDown` is what `open`, `version`, and session operations return after `shutdown` has been attempted.

`Library::shutdown` calls `lore_shutdown` and, from that moment, rejects session operations. If `lore_shutdown` returns non-zero, `shutdown` returns `Error::Failed` and a later `shutdown` calls it again. After it returns zero, later `shutdown` calls return `Ok(())`. Dropping a `Library` does not shut it down.

## Capability matrix

| Capability | State |
| --- | --- |
| Local create, status, stage, unstage, commit | Implemented |
| History, diff, branch list, create, switch | Implemented |
| Committed file bytes | Implemented. `write_file` lets Lore write a path. `read_file` writes that file to a temp directory, then keeps at most the caller limit in memory |
| Repository discovery | Implemented (parent walk for a `.lore` directory) |
| Progress events | Out of scope. Events are accepted so the call can finish, and not returned |
| Clone, push, fetch, remotes, auth | Out of scope. These sessions never send tokens or open a remote |
| Merge, cherry-pick, revert, locks, metadata, revision trees, storage streaming | Out of scope |
| Cancellation | Not in the 0.10.1 C header |
| Sparse checkouts and VFS mounts | Not wrapped. Create uses no VFS |

## Platforms and compatibility

Tested on Windows x64 with `lore.dll` from the Lore v0.10.1 release and rustc 1.98.1. The Linux and macOS file names are selected in code. Those builds were not executed on this machine. CI is set up to run the Ubuntu x64 library as well; macOS is not in CI. Lore publishes a macOS library for Apple silicon only.

The minimum supported Rust version is 1.85. Direct dependencies are `libloading` (ISC), `thiserror` (MIT OR Apache-2.0), and `tempfile` (MIT OR Apache-2.0). Transitive crates include Unicode-3.0 (`unicode-ident`). `Cargo.lock` is checked in so that graph stays put.

Lore is pre-1.0. A future interface version will need new layouts; this crate will not pretend the 0.10.1 layouts still match.

## Limitations

- One loaded library per process, matching Lore's process-global runtime.
- Calls block the thread that made them. There is no async runtime.
- `history(0)` is rejected. The C API treats zero as unlimited.
- An event storm stops being stored after `Session::event_limit` (default 100_000) and the call fails.
- A string longer than 1 MiB fails the call. It is not silently truncated.
- `examples/local.rs` prints the library version, the current branch and revision, and each reported file. It calls `shutdown` before it returns.

## Tests

```sh
cargo test --lib
cargo test --doc
```

Integration tests need the C library and should not use the background service:

```sh
LORE_USE_SERVICE=0 LORE_LIBRARY=/path/to/lore.dll cargo test --tests
```

On PowerShell:

```powershell
$env:LORE_USE_SERVICE = "0"
$env:LORE_LIBRARY = "C:\path\to\lore.dll"
cargo test --tests
```

The integration tests create temporary repositories and do not take a repository path from the environment.

## Troubleshooting

- **Incompatible version.** The file is not the 0.10.1 library. `Library::version` reports what was loaded when the version is accepted; a mismatch fails in `open`.
- **Missing symbol.** The file is not a Lore C library, or it is a build that does not export the async entry points.
- **Status on a directory that is not a checkout.** Lore returns `Error::Failed`. `discover` is a separate walk for a `.lore` directory and returns `Error::NotARepository`.
- **Calls affect another repository.** The user-level service is enabled. Run with `LORE_USE_SERVICE=0`.
- **Process hangs on exit.** Call `Library::shutdown` when you are done. Lore's worker threads are not daemon threads.

## License

MIT. See [LICENSE](LICENSE). Lore itself is also MIT, Copyright Epic Games, Inc.

Building a program with this crate does not grant rights to Lore's trademarks.
