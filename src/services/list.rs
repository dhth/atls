use crate::domain::{Entry, EntryKind};
use anyhow::Context;
use std::path::Path;
use tokio::fs;
use tracing::debug;

pub async fn list_entries_at_directory<P>(path: P) -> anyhow::Result<Vec<Entry>>
where
    P: AsRef<Path>,
{
    debug!("reading directory: {:?}", path.as_ref());
    let mut read_dir_result = fs::read_dir(&path)
        .await
        .inspect_err(|e| debug!("couldn't read directory: {:?}", e))
        .context("couldn't get entries at path")?;

    let mut entries = vec![];
    while let Some(entry) = read_dir_result.next_entry().await? {
        let entry_path = entry.path();
        match fs::metadata(&entry_path).await {
            Ok(m) => {
                let path_kind = if m.is_file() {
                    EntryKind::File
                } else if m.is_dir() {
                    EntryKind::Directory
                } else if m.is_symlink() {
                    EntryKind::Symlink
                } else {
                    EntryKind::Unknown
                };

                entries.push(Entry::new(entry_path, path_kind));
            }
            Err(_e) => {} // TODO: handle this error
        }
    }

    debug!(
        "found {} entries in directory {:?}",
        entries.len(),
        path.as_ref()
    );
    Ok(entries)
}
