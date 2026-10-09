// Copyright 2026 tess
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use lore_sdk::Library;

fn main() -> lore_sdk::Result<()> {
    let mut args = std::env::args().skip(1);
    let repository =
        args.next()
            .map(PathBuf::from)
            .ok_or_else(|| lore_sdk::Error::InvalidArgument {
                field: "repository",
                message: "usage: local <repository>".to_string(),
            })?;
    let library = Library::from_env()?;
    println!("{}", library.version()?);
    let status = library.session(repository)?.status()?;
    println!(
        "{} @ {} ({})",
        status.branch, status.revision, status.revision_number
    );
    for file in status.files {
        println!("{:?} {} ({} bytes)", file.action, file.path, file.size);
    }
    library.shutdown()?;
    Ok(())
}
