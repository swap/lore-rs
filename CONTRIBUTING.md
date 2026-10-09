# Contributing

This repository is a community SDK. It is not the Lore project. Changes to Lore's on-disk format, wire protocol, or C API belong in [EpicGames/lore](https://github.com/EpicGames/lore) and follow that repository's rules, including its DCO.

For this crate:

- Keep the wrapped interface pinned to a Lore version you have measured. Do not guess struct layouts.
- Add a test for behavior you change. Integration tests must use a temporary directory and `LORE_USE_SERVICE=0`.
- Do not add a GPL, LGPL, or AGPL dependency. Lore is MIT, and this crate is MIT.
- If you use an AI tool to prepare a patch, say which tool in the pull request. Review the diff yourself. You are responsible for it.

`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test --lib`, and `cargo test --doc` should pass before you open a pull request.
