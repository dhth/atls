use super::Entry;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum FSOperation {
    Copy {
        items: Vec<Entry>,
        destination: PathBuf,
    },
    Move {
        items: Vec<Entry>,
        destination: PathBuf,
    },
}
