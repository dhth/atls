use std::path::PathBuf;

pub const MAX_NUM_SESSIONS: usize = 4;
pub const MIN_TERMINAL_WIDTH: u16 = 50;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;

const HELP_CONTENT_RAW: &str = include_str!("static/help.txt");

pub fn get_help_content() -> String {
    HELP_CONTENT_RAW.to_string()
}

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub index: usize,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct DirectoryAddress {
    pub session_index: usize,
    pub path: PathBuf,
}

impl From<DirectoryAddress> for SessionInfo {
    fn from(val: DirectoryAddress) -> Self {
        SessionInfo {
            index: val.session_index,
            path: val.path,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Pane {
    Explorer,
    Help,
}

impl std::fmt::Display for Pane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pane::Explorer => write!(f, "explorer"),
            Pane::Help => write!(f, "help"),
        }
    }
}

pub(super) struct TerminalDimensions {
    pub(super) width: u16,
    pub(super) height: u16,
}

impl TerminalDimensions {
    pub(super) fn update(&mut self, new_width: u16, new_height: u16) {
        self.width = new_width;
        self.height = new_height;
    }
}

#[cfg(test)]
impl From<(u16, u16)> for TerminalDimensions {
    fn from(value: (u16, u16)) -> Self {
        let (width, height) = value;
        Self { width, height }
    }
}
