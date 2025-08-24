use super::common::*;
use crate::common::*;
use crate::domain::{Entry, EntryKind};
use ratatui::widgets::ListState;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::debug;

const USER_MESSAGE_DEFAULT_FRAMES: u16 = 4;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug)]
pub enum MessageKind {
    Info,
    Error,
}

pub struct UserMsg {
    pub frames_left: u16,
    pub value: String,
    pub kind: MessageKind,
}

#[allow(unused)]
impl UserMsg {
    pub(super) fn info<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        UserMsg {
            frames_left: USER_MESSAGE_DEFAULT_FRAMES,
            value: message.into(),
            kind: MessageKind::Info,
        }
    }
    pub(super) fn error<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        UserMsg {
            frames_left: USER_MESSAGE_DEFAULT_FRAMES,
            value: message.into(),
            kind: MessageKind::Error,
        }
    }

    #[allow(unused)]
    pub(super) fn with_frames_left(mut self, frames_left: u16) -> Self {
        self.frames_left = frames_left;
        self
    }

    pub(super) fn internal_error() -> Self {
        Self {
            frames_left: USER_MESSAGE_DEFAULT_FRAMES,
            value: format!(
                "something went wrong; let {} know via {}",
                AUTHOR, ISSUES_URL
            ),
            kind: MessageKind::Error,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntryItem {
    pub entry: Entry,
    pub marked: bool,
}

#[derive(Debug, Clone)]
pub enum Session {
    Uninitialized,
    Initialized {
        path: PathBuf,
        entries: Vec<EntryItem>,
        state: ListState,
    },
}

impl Session {
    fn new_empty(path: PathBuf) -> Self {
        let state = ListState::default();
        let items = vec![];

        Self::Initialized {
            path,
            entries: items,
            state,
        }
    }

    fn new(path: PathBuf, entries: Vec<Entry>) -> Self {
        let mut state = ListState::default();
        if !entries.is_empty() {
            state.select(Some(0));
        }
        let entries = entries
            .into_iter()
            .map(|entry| EntryItem {
                entry,
                marked: false,
            })
            .collect::<Vec<_>>();

        Self::Initialized {
            path,
            entries,
            state,
        }
    }

    fn select_path<P>(&mut self, path_to_select: P) -> bool
    where
        P: AsRef<Path>,
    {
        match self {
            Session::Uninitialized => false,
            Session::Initialized {
                path: _,
                entries,
                state,
            } => {
                for (i, item) in entries.iter().enumerate() {
                    if item.entry.path() == path_to_select.as_ref() {
                        state.select(Some(i));
                        return true;
                    }
                }

                false
            }
        }
    }

    pub fn is_initialized(&self) -> bool {
        match self {
            Session::Uninitialized => false,
            Session::Initialized { .. } => true,
        }
    }
}

pub struct Model {
    pub sessions: Vec<Session>,
    pub current_session_index: usize,
    pub marked_paths: HashSet<Entry>,
    pub last_selections: HashMap<PathBuf, PathBuf>,
    pub active_pane: Pane,
    pub last_active_pane: Option<Pane>,
    pub running_state: RunningState,
    pub user_msg: Option<UserMsg>,
    pub terminal_dimensions: TerminalDimensions,
    pub terminal_too_small: bool,
    pub render_counter: u64,
    pub event_counter: u64,
    pub debug: bool,
    pub help_scroll: usize,
}

impl Model {
    pub fn new(root: PathBuf, terminal_dimensions: TerminalDimensions, debug: bool) -> Self {
        let terminal_too_small = terminal_dimensions.width < MIN_TERMINAL_WIDTH
            || terminal_dimensions.height < MIN_TERMINAL_HEIGHT;

        let mut sessions = vec![Session::new_empty(root.clone())];
        for _ in 1..MAX_NUM_SESSIONS {
            sessions.push(Session::Uninitialized);
        }

        Model {
            sessions,
            current_session_index: 0,
            marked_paths: HashSet::new(),
            last_selections: HashMap::new(),
            active_pane: Pane::Explorer,
            last_active_pane: None,
            running_state: RunningState::Running,
            user_msg: None,
            terminal_dimensions,
            terminal_too_small,
            render_counter: 0,
            event_counter: 0,
            debug,
            help_scroll: 0,
        }
    }

    pub(super) fn go_back_or_quit(&mut self) {
        let active_pane = Some(self.active_pane);
        match self.active_pane {
            Pane::Explorer => {
                if self.marked_paths.is_empty() {
                    let was_last_session = self.close_current_session();
                    if was_last_session {
                        self.running_state = RunningState::Done;
                    }
                } else {
                    self.marked_paths.clear();
                    self.sync_marked_paths_to_current_session();
                }
            }
            Pane::Help => match self.last_active_pane {
                Some(p) => self.active_pane = p,
                None => self.active_pane = Pane::Explorer,
            },
        }

        self.last_active_pane = active_pane;
    }

    pub(super) fn select_next(&mut self) {
        match self.active_pane {
            Pane::Explorer => {
                let current_session = self.current_session_mut();
                match current_session {
                    Session::Uninitialized => {}
                    Session::Initialized {
                        path: _,
                        entries,
                        state,
                    } => {
                        if entries.is_empty() {
                            return;
                        }

                        if let Some(i) = state.selected()
                            && i == entries.len() - 1
                        {
                            return;
                        }

                        state.select_next();
                    }
                }
            }
            Pane::Help => {}
        }
    }

    pub(super) fn select_previous(&mut self) {
        match self.active_pane {
            Pane::Explorer => {
                let current_session = self.current_session_mut();
                match current_session {
                    Session::Uninitialized => {}
                    Session::Initialized {
                        path: _,
                        entries,
                        state,
                    } => {
                        if entries.is_empty() {
                            return;
                        }

                        if let Some(i) = state.selected()
                            && i == 0
                        {
                            return;
                        }

                        state.select_previous();
                    }
                }
            }
            Pane::Help => {}
        }
    }

    pub(super) fn select_first(&mut self) {
        if self.active_pane == Pane::Explorer {
            let current_session = self.current_session_mut();
            match current_session {
                Session::Uninitialized => {}
                Session::Initialized {
                    path: _,
                    entries,
                    state,
                } => {
                    if entries.is_empty() {
                        return;
                    }

                    if let Some(i) = state.selected()
                        && i == 0
                    {
                        return;
                    }

                    state.select_first();
                }
            }
        }
    }
    pub(super) fn select_last(&mut self) {
        if self.active_pane == Pane::Explorer {
            match self.current_session_mut() {
                Session::Uninitialized => {}
                Session::Initialized {
                    path: _,
                    entries,
                    state,
                } => {
                    if entries.is_empty() {
                        return;
                    }

                    let last_index = entries.len() - 1;
                    if let Some(i) = state.selected()
                        && i == last_index
                    {
                        return;
                    }

                    state.select(Some(last_index));
                }
            }
        }
    }

    pub(super) fn update_entries_for_session(
        &mut self,
        session_info: SessionInfo,
        entries: Vec<Entry>,
        navigated_to: bool,
    ) {
        for i in 0..self.sessions.len() {
            if navigated_to && i == session_info.index {
                if let Some(session_path) = self.current_session_path()
                    && let Some(selected_path) = self.currently_selected_path()
                {
                    debug!(
                        "add to last_selections: {:?}->{:?}",
                        &session_path, &selected_path
                    );
                    self.last_selections.insert(session_path, selected_path);
                }
                // create a new session with the new path
                self.sessions[i] = Session::new(session_info.path.clone(), entries.clone());
                if let Some(last_selection) = self.last_selections.get(&session_info.path) {
                    debug!(
                        "got last selection: {:?}->{:?}",
                        &session_info.path, last_selection
                    );
                    self.sessions[i].select_path(last_selection);
                } else {
                    debug!(
                        "didn't have last selection for path: {:?}",
                        &session_info.path
                    );
                }

                continue;
            }

            // maybe some other session is on the same path
            // if so, update its entries as well
            match &self.sessions[i] {
                Session::Uninitialized => {}
                Session::Initialized {
                    path: session_path,
                    entries: _,
                    state: _,
                } => {
                    if session_path == &session_info.path {
                        // TODO: improve this to handle changes to the directory
                        // If that happens, the selected entry will jump around
                        self.sessions[i] = Session::new(session_info.path.clone(), entries.clone());
                    }
                }
            }
        }
    }

    pub(super) fn current_session(&self) -> &Session {
        &self.sessions[self.current_session_index]
    }

    pub(super) fn current_session_mut(&mut self) -> &mut Session {
        &mut self.sessions[self.current_session_index]
    }

    pub(super) fn toggle_path_marked_status(&mut self) {
        let current_session = &mut self.sessions[self.current_session_index];
        match current_session {
            Session::Uninitialized => todo!(),
            Session::Initialized {
                path: _,
                entries,
                state,
            } => {
                if let Some(selected_index) = state.selected() {
                    if selected_index >= entries.len() {
                        self.user_msg = Some(UserMsg::internal_error());
                        return;
                    }

                    let item = &mut entries[selected_index];
                    if self.marked_paths.contains(&item.entry) {
                        self.marked_paths.remove(&item.entry);
                        item.marked = false;
                    } else {
                        self.marked_paths.insert(item.entry.clone());
                        item.marked = true;
                    }

                    if selected_index == entries.len() - 1 {
                        return;
                    }

                    state.select(Some(selected_index + 1));
                }
            }
        }
    }

    pub(super) fn go_to_next_session(&mut self) {
        if self.num_initialized_sessions() == 1 {
            self.sessions[1] = self.sessions[0].clone();
            self.current_session_index = 1;
            return;
        }

        if self.current_session_index == MAX_NUM_SESSIONS - 1 {
            self.current_session_index = 0;
        } else {
            self.current_session_index += 1;
        }

        if let Session::Uninitialized = &self.current_session() {
            self.go_to_next_session();
        } else {
            self.sync_marked_paths_to_current_session();
        }
    }

    pub(super) fn go_to_previous_session(&mut self) {
        if self.num_initialized_sessions() == 1 {
            return;
        }

        if self.current_session_index == 0 {
            self.current_session_index = MAX_NUM_SESSIONS - 1;
        } else {
            self.current_session_index -= 1;
        }

        if let Session::Uninitialized = &self.current_session() {
            self.go_to_previous_session();
        } else {
            self.sync_marked_paths_to_current_session();
        }
    }

    pub(super) fn go_to_session(&mut self, index: usize) {
        if index >= self.sessions.len() {
            return;
        }

        if index == self.current_session_index {
            return;
        }

        let target_session = &self.sessions[index];
        if !target_session.is_initialized() {
            let current_session = self.current_session();
            self.sessions[index] = current_session.clone();
        }
        self.current_session_index = index;

        self.sync_marked_paths_to_current_session();
    }

    pub(super) fn get_current_directory(&self) -> Option<DirectoryAddress> {
        if let Session::Initialized {
            path: _,
            entries,
            state,
        } = self.current_session()
            && let Some(i) = state.selected()
            && i < entries.len()
        {
            let current_entry = &entries[i].entry;
            if let EntryKind::Directory = current_entry.kind() {
                return Some(DirectoryAddress {
                    session_index: self.current_session_index,
                    path: current_entry.path().to_path_buf(),
                });
            }
        }

        None
    }

    pub(super) fn get_parent_directory(&self) -> Option<DirectoryAddress> {
        if let Session::Initialized {
            path: _,
            entries,
            state,
        } = self.current_session()
            && let Some(i) = state.selected()
            && i < entries.len()
        {
            let current_entry = &entries[i].entry;
            if let Some(parent) = current_entry.path().parent().and_then(|p| p.parent()) {
                return Some(DirectoryAddress {
                    session_index: self.current_session_index,
                    path: parent.to_path_buf(),
                });
            }
        }

        None
    }

    fn num_initialized_sessions(&self) -> usize {
        self.sessions.iter().filter(|s| s.is_initialized()).count()
    }

    fn sync_marked_paths_to_current_session(&mut self) {
        let current_session = &mut self.sessions[self.current_session_index];
        match current_session {
            Session::Uninitialized => {}
            Session::Initialized {
                path: _,
                entries,
                state: _,
            } => {
                for item in entries {
                    item.marked = self.marked_paths.contains(&item.entry);
                }
            }
        }
    }

    fn current_session_path(&mut self) -> Option<PathBuf> {
        if let Session::Initialized { path, .. } = self.current_session() {
            Some(path.clone())
        } else {
            None
        }
    }

    fn currently_selected_path(&mut self) -> Option<PathBuf> {
        if let Session::Initialized {
            path: _,
            entries,
            state,
        } = self.current_session()
            && let Some(i) = state.selected()
            && i < entries.len()
        {
            Some(entries[i].entry.path().to_path_buf())
        } else {
            None
        }
    }

    fn close_current_session(&mut self) -> bool {
        self.sessions[self.current_session_index] = Session::Uninitialized;

        if self.num_initialized_sessions() == 0 {
            return true;
        }

        let mut next_index = if self.current_session_index > 0 {
            self.current_session_index - 1
        } else {
            self.sessions.len() - 1
        };

        loop {
            if self.sessions[next_index].is_initialized() {
                self.current_session_index = next_index;
                break;
            }

            next_index = if next_index > 0 {
                next_index - 1
            } else {
                self.sessions.len() - 1
            };
        }

        false
    }
}
