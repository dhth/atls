use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum EntryKind {
    Directory,
    File,
    Symlink,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Entry {
    inner: PathBuf,
    kind: EntryKind,
    path_str: String,
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.kind.cmp(&other.kind) {
            std::cmp::Ordering::Equal => self.inner.cmp(&other.inner),
            other => other,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;
    use std::path::PathBuf;

    #[test]
    fn test_entries_are_sorted_correctly() {
        // GIVEN

        let mut entries = vec![
            Entry::new(PathBuf::from("/home/user/atls/Cargo.lock"), EntryKind::File),
            Entry::new(PathBuf::from("/home/user/atls/src"), EntryKind::Directory),
            Entry::new(PathBuf::from("/home/user/atls/.git"), EntryKind::Directory),
            Entry::new(PathBuf::from("/home/user/atls/link-a"), EntryKind::Symlink),
            Entry::new(PathBuf::from("/home/user/atls/Cargo.toml"), EntryKind::File),
            Entry::new(PathBuf::from("/home/user/atls/file"), EntryKind::Unknown),
            Entry::new(
                PathBuf::from("/home/user/atls/target"),
                EntryKind::Directory,
            ),
            Entry::new(PathBuf::from("/home/user/atls/link-b"), EntryKind::Symlink),
            Entry::new(PathBuf::from("/home/user/atls/.fdignore"), EntryKind::File),
        ];

        // WHEN
        entries.sort();

        // THEN
        let paths = entries
            .into_iter()
            .map(|e| e.path_str())
            .collect::<Vec<_>>();

        assert_yaml_snapshot!(paths, @r#"
        - ".git/"
        - src/
        - target/
        - ".fdignore"
        - Cargo.lock
        - Cargo.toml
        - link-a
        - link-b
        - file
        "#);
    }
}
