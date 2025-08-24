use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Entry {
    inner: PathBuf,
    kind: EntryKind,
    path_str: String,
}

impl Entry {
    pub fn new(path: PathBuf, kind: EntryKind) -> Self {
        let path_str = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            inner: path,
            kind,
            path_str,
        }
    }

    pub fn path_str(&self) -> String {
        match self.kind() {
            EntryKind::Directory => format!("{}/", self.path_str),
            _ => self.path_str.clone(),
        }
    }

    pub fn kind(&self) -> EntryKind {
        self.kind
    }

    pub fn path(&self) -> &Path {
        self.inner.as_path()
    }
}
