use crate::domain::Entry;

use super::common::{Pane, SessionInfo};
use super::model::Model;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

#[derive(Debug)]
pub enum Msg {
    // user actions
    CopyMarkedItems,
    GoBackOrQuit,
    GoToNextSession,
    GoToPane(Pane),
    GoToPreviousSession,
    GoToSession(usize),
    MarkPath,
    MoveMarkedItems,
    NavigateIntoDir,
    NavigateOutOfDir,
    QuitImmediately,
    SelectFirst,
    SelectLast,
    SelectNext,
    SelectPrevious,
    TerminalResize(u16, u16),
    // internal
    FSOperationFinished(anyhow::Result<()>),
    DirectoryRead {
        session_info: SessionInfo,
        entries: Vec<Entry>,
        navigated_to: bool,
    },
    ReadingDirFailed(String),
}

pub fn get_event_handling_msg(model: &Model, event: Event) -> Option<Msg> {
    match event {
        Event::Key(key_event) => match model.terminal_too_small {
            true => match key_event.kind {
                KeyEventKind::Press => match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => Some(Msg::GoBackOrQuit),
                    _ => None,
                },
                _ => None,
            },
            false => match key_event.kind {
                KeyEventKind::Press => match model.active_pane {
                    Pane::Explorer => match key_event.code {
                        KeyCode::Char(' ') => Some(Msg::MarkPath),
                        KeyCode::Char('j') | KeyCode::Down => Some(Msg::SelectNext),
                        KeyCode::Char('k') | KeyCode::Up => Some(Msg::SelectPrevious),
                        KeyCode::Char('g') => Some(Msg::SelectFirst),
                        KeyCode::Char('G') => Some(Msg::SelectLast),
                        KeyCode::Tab => Some(Msg::GoToNextSession),
                        KeyCode::Char('1') => Some(Msg::GoToSession(0)),
                        KeyCode::Char('2') => Some(Msg::GoToSession(1)),
                        KeyCode::Char('3') => Some(Msg::GoToSession(2)),
                        KeyCode::Char('4') => Some(Msg::GoToSession(3)),
                        KeyCode::BackTab => Some(Msg::GoToPreviousSession),
                        KeyCode::Char('l') | KeyCode::Right => Some(Msg::NavigateIntoDir),
                        KeyCode::Char('h') | KeyCode::Left => Some(Msg::NavigateOutOfDir),
                        KeyCode::Char('p') if !model.marked_paths.is_empty() => {
                            Some(Msg::CopyMarkedItems)
                        }
                        KeyCode::Char('v') if !model.marked_paths.is_empty() => {
                            Some(Msg::MoveMarkedItems)
                        }
                        KeyCode::Esc | KeyCode::Char('q') => Some(Msg::GoBackOrQuit),
                        KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
                            Some(Msg::QuitImmediately)
                        }
                        KeyCode::Char('?') => Some(Msg::GoToPane(Pane::Help)),
                        _ => None,
                    },
                    Pane::Help => match key_event.code {
                        KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc => {
                            Some(Msg::GoBackOrQuit)
                        }
                        KeyCode::Char('c') => {
                            if key_event.modifiers == KeyModifiers::CONTROL {
                                Some(Msg::QuitImmediately)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    },
                },
                _ => None,
            },
        },
        Event::Resize(w, h) => Some(Msg::TerminalResize(w, h)),
        _ => None,
    }
}
