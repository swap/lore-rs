# Release readiness

Overall status: **READY FOR RELEASE**

This is a community crate. It is not an Epic Games project and it is not endorsed by Epic. It is not published to crates.io.

The decision covers local Lore workflows against the published C library for interface 0.10.1, on Windows x64 and Linux x64. macOS is recognized by file name and was not executed. A passing test run does not mean the crate is free of defects.

## Contribution

| Item | Evidence |
| --- | --- |
| Where it belongs | A separate community repository. Lore's contributing guide asks for an issue before non-trivial work and a Lore Enhancement Proposal before a new public language binding. The in-tree `lore` crate already exists and is `publish = false`. No pull request was opened against `EpicGames/lore`. |
| Repository | https://github.com/swap/lore-rs |
| Remote | `origin` `https://github.com/swap/lore-rs.git` (fetch and push) |
| Branch | `main` |
| Base | This repository was created empty. There is no upstream base branch and no pull request. |
| Implementation commit | `864cf3880a37088d5da082138394fd349024d91f` |
| CI fix commit | `caded6b5298ba0aea32e95c44488601422a7a4bd` |
| Report commit | `315839f56213a6adaa528bc5d3b33c00955c5d38` |
| Pull request | Not created. The branch is the repository's initial history, not a review against an existing project. |
| State | BRANCH PUSHED. CI VERIFIED. Not AWAITING MAINTAINER REVIEW. Not MERGED. |

The first Actions run, https://github.com/swap/lore-rs/actions/runs/37913536786, failed on `windows-latest` because `cargo fmt --all -- --check` rejected CRLF checkouts (`Incorrect newline style`). `ubuntu-latest` in that run passed, including integration tests. `.gitattributes` forces LF. The rerun is the verified one.

Verified CI: https://github.com/swap/lore-rs/actions/runs/37913699235

- Conclusion: `success`
- Head: `caded6b5298ba0aea32e95c44488601422a7a4bd`
- `windows-latest`: format, clippy, unit tests (13), doc tests (2), integration step (13 + 1 + 4) passed
- `ubuntu-latest`: the same steps passed, with the same counts

The integration step runs `cargo test --tests`, which on this Cargo also runs the library tests, then `tests/shutdown.rs` (1) and `tests/workflow.rs` (4). Both jobs downloaded the Lore v0.10.1 library (`lore.dll` on Windows, `liblore.so` on Ubuntu) and set `LORE_USE_SERVICE=0`.

The report commit `315839f56213a6adaa528bc5d3b33c00955c5d38` was checked at https://github.com/swap/lore-rs/actions/runs/37913996759. Conclusion: `success`. Both `windows-latest` and `ubuntu-latest` passed format, clippy, unit tests, doc tests, and integration tests. The commit that records this paragraph is documentation only.

AI assistance: a Cursor agent drafted this crate. Layouts were taken from an MSVC build of the Lore 0.10.1 header, and the commands below were executed. No `Signed-off-by` line was added. Lore's DCO applies to `EpicGames/lore`, and this commit is not one of those.

## Environment

- Local OS: Windows 10.0.26200, x64
- Local toolchain: rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1
- MSRV check: rustc 1.85.0 (4d91de4e4 2025-02-17), installed with `rustup toolchain install 1.85.0 --profile minimal`
- Lore library, local: `C:\Users\sa1nt\AppData\Local\Temp\lore-lib\lib\lore.dll` from the v0.10.1 Windows release asset `liblore-v0.10.1-x86_64-pc-windows-msvc.zip`
- Lore library, CI: the v0.10.1 Windows and Linux x64 release assets
- `LORE_USE_SERVICE=0` for every integration run
- Tests use temporary directories only

## Checks

| Check | Result | Procedure | Outcome |
| --- | --- | --- | --- |
| Format | PASS | `cargo fmt --all -- --check` locally, then the same step on `windows-latest` and `ubuntu-latest` in run 37913699235 | Local exit 0. Both CI jobs passed after the LF checkout fix. |
| Clippy | PASS | `cargo clippy --all-targets -- -D warnings` locally and in both CI jobs | Clean. |
| Compile, stable | PASS | `cargo test` builds on 1.98.1 and on CI stable | Finished without errors. |
| Compile, MSRV 1.85 | PASS | `cargo +1.85.0 test --lib`, `--doc`, and `--tests -- --test-threads=1` with `LORE_LIBRARY` set | 13 library, 2 doc, 1 shutdown, 4 workflow passed. |
| Library tests | PASS | `cargo test --lib` on 1.98.1 | 13 passed, 0 failed. |
| Doc tests | PASS | `cargo test --doc` on 1.98.1 | 2 passed. `cargo test --lib --doc` is rejected by Cargo, so CI runs them as separate steps. |
| Integration, Windows local | PASS | `LORE_USE_SERVICE=0`, `LORE_LIBRARY` pointed at the v0.10.1 DLL, `cargo test --tests -- --test-threads=1` | 13 library, 1 shutdown, 4 workflow passed. Workflow finished in 0.33s and included a 1 MiB round trip. |
| Integration, CI Windows | PASS | Run 37913699235, `windows-latest` | 13, 1, and 4 passed. |
| Integration, CI Linux | PASS | Run 37913699235, `ubuntu-latest`, `liblore.so` from the v0.10.1 x64 tarball | 13, 1, and 4 passed. |
| macOS | UNVERIFIED | No Apple silicon runner and no local macOS library | Not a tested platform. The loader looks for `liblore.dylib`. Lore publishes an aarch64 macOS asset only. |
| Package list | PASS | `cargo package --list --allow-dirty` | Lists the crate sources, `Cargo.lock`, docs except this file, and the example. Does not list `.github/`, this file, or a Lore library. Cargo warned that `documentation` and `homepage` are unset. `repository` is `https://github.com/swap/lore-rs`. |
| License inventory | PASS | crates.io API `version.license` for every dependency in `Cargo.lock` except this crate | Every crate can be used under MIT, Apache-2.0, ISC, or Unicode-3.0. `r-efi` 6.0.0 is `MIT OR Apache-2.0 OR LGPL-2.1-or-later`. `linux-raw-sys` 0.12.1 and `rustix` 1.1.5 also offer `Apache-2.0 WITH LLVM-exception`. Those extra terms are alternatives, not requirements. |
| `cargo deny` | UNVERIFIED | `deny.toml` is present. `cargo deny` is not installed and is not a CI step | The inventory above is the license evidence. The deny config has not been executed. |
| RustSec listing | PASS | GitHub contents of `rustsec/advisory-db` `crates/<name>` for each locked crate | Only `once_cell` has a file: RUSTSEC-2019-0017, patched in `>= 1.0.1`. The lockfile has `once_cell` 1.21.4. `cargo audit` is not installed, so this was a directory listing rather than that tool. |
| Security review | PASS | Code review of the FFI, process, and filesystem paths. An earlier independent review pass listed defects; the high ones were fixed and retested. | No remaining critical or high defect. See the defect list. |
| Benchmarks | UNVERIFIED | Not run | No numbers are claimed. The calls block on Lore and copy event strings. |

## Feature to test

| Feature | Test |
| --- | --- |
| Create, `.lore` directory, discover from a directory and from a file | `local_repository_workflow` |
| Status of a dirty tree | `local_repository_workflow` |
| Stage, missing path, unstage, staged flag | `local_repository_workflow` |
| Commit, parent of the first commit, history, `history(1)` | `local_repository_workflow` |
| 1 MiB pattern with interior NUL bytes, empty file, repeated bytes | `local_repository_workflow` via `read_file`, which calls `write_file` |
| `read_file` limit | `Error::TooLarge` with limit 16 |
| Diff of two commits | `local_repository_workflow` |
| Branch list, create, switch, switch back | `local_repository_workflow` |
| Event cap | `event_limit(1)` while staging two files returns `Error::EventLimit` |
| Status outside a checkout | `status_outside_a_repository_fails` expects `Error::Failed` |
| Empty path list, empty branch, `history(0)`, empty repository path | `empty_inputs_fail_before_the_library_runs_them` |
| Two overlapping status calls | `concurrent_status_calls_share_one_library` |
| Version pin and shutdown latch | `shutdown_rejects_later_calls` |
| Interface match is exact before a `+` build suffix | `version_match_is_exact_before_the_build_suffix` and the `interface_compatible` doctest |
| Struct sizes and offsets | `sizes_match_lore_0_10_1_header`, `field_offsets_match_lore_0_10_1_header` |
| Timeout does not free the callback slot | `timeout_does_not_free_the_slot_before_end`, `timeout_returns_and_a_later_event_still_has_the_slot` |
| Missing library does not stick in the process slot | `missing_library_file_fails_without_sticking` |

Remotes, authentication, merge, locks, cancellation, and progress callbacks are out of scope and are not advertised as implemented.

## Defects

Fixed before the verified commit:

- High: `Library::open` could load two libraries. The process mutex is now held across the load.
- High: a failed `lore_shutdown` could be treated as done. `shutdown` retries until the call returns zero.
- High: C argument padding was not cleared. Argument structs are zeroed.
- High: Windows CI would pass an MSYS path to `LoadLibrary`. The workflow converts it with `cygpath -w`.
- High: `cargo test --lib --doc` does not run. CI splits the steps. The first Windows job then failed on CRLF; `.gitattributes` fixes that, and run 37913699235 passed.

Still open, none critical or high:

| Severity | Item | Plan |
| --- | --- | --- |
| Low | The header has no cancel function. A timeout returns while Lore can still be running. `read_file` may fail to delete its temp directory until that write finishes. | Documented on `read_file` and in the README. Do not pretend the call was aborted. |
| Low | `shutdown` does not wake a call that is already waiting. | The waiter uses its own timeout. |
| Low | `event_limit` counts events, not bytes. Each copied string is capped at 1 MiB and a longer string fails the call. | Accepted. The default cap is 100_000 events. |
| Low | `cargo deny` is not executed. | The crates.io license list is the current evidence. Wire `cargo deny` into CI if the tool is adopted. |
| Low | macOS is not tested. | Do not treat `liblore.dylib` as a verified target until an Apple silicon run exists. |
| Low | `Library::from_env` directory lookup is not covered by a unit test. | Add a temp-directory test if that lookup changes. |

## Diff

`git diff` against the empty repository at `caded6b5298ba0aea32e95c44488601422a7a4bd`:

- `864cf38` added the crate: 24 files, 2918 insertions
- `caded6b` added `.gitattributes`: 1 file, 1 insertion
- `315839f` added this file

The commit that records the report's own CI run only edits this file.
