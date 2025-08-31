use crate::domain::Entry;
use anyhow::Context;
use fs_extra::{copy_items, dir::CopyOptions, move_items};
use std::path::Path;
use tracing::{debug, error};

pub fn copy_entries_to_destination<P>(entries: &[Entry], destination: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let paths = entries
        .iter()
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();

    debug!("copying paths: {:?}", &paths);

    let destination = destination.as_ref().to_path_buf();
    copy_items(
        paths.as_slice(),
        destination,
        &CopyOptions::new().overwrite(true),
    )
    .map(|_| ())
    .context("couldn't copy items")
    .inspect_err(|e| {
        error!("copying items failed: {:?}", e);
    })
}

pub fn move_entries_to_destination<P>(entries: &[Entry], destination: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let paths = entries
        .iter()
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();

    debug!("moving paths: {:?}", &paths);

    let destination = destination.as_ref().to_path_buf();
    move_items(
        paths.as_slice(),
        destination,
        &CopyOptions::new().overwrite(true),
    )
    .map(|_| ())
    .context("couldn't move items")
    .inspect_err(|e| {
        error!("moving items failed: {:?}", e);
    })
}
