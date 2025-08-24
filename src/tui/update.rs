use super::cmd::Cmd;
use super::common::*;
use super::model::*;
use super::msg::Msg;
use tracing::debug;

pub fn update(model: &mut Model, msg: Msg) -> Vec<Cmd> {
    debug!("tui got message: {:#?}", &msg);
    let mut cmds = vec![];
    match msg {
        // user actions
        Msg::GoBackOrQuit => model.go_back_or_quit(),
        Msg::GoToNextSession => model.go_to_next_session(),
        Msg::GoToPane(pane) => {
            model.last_active_pane = Some(model.active_pane);
            model.active_pane = pane;
        }
        Msg::GoToPreviousSession => model.go_to_previous_session(),
        Msg::GoToSession(index) => model.go_to_session(index),
        Msg::NavigateIntoDir => {
            if let Some(directory_address) = model.get_current_directory() {
                cmds.push(Cmd::ReadDir((directory_address.into(), true)));
            }
        }
        Msg::NavigateOutOfDir => {
            if let Some(directory_address) = model.get_parent_directory() {
                cmds.push(Cmd::ReadDir((directory_address.into(), true)));
            } else {
                model.user_msg = Some(UserMsg::error("no parent found"));
            }
        }
        Msg::QuitImmediately => model.running_state = RunningState::Done,
        Msg::SelectFirst => model.select_first(),
        Msg::SelectLast => model.select_last(),
        Msg::SelectNext => model.select_next(),
        Msg::MarkPath => model.toggle_path_marked_status(),
        Msg::SelectPrevious => model.select_previous(),
        Msg::TerminalResize(new_width, new_height) => {
            model.terminal_dimensions.update(new_width, new_height);
            model.terminal_too_small =
                !(new_width >= MIN_TERMINAL_WIDTH && new_height >= MIN_TERMINAL_HEIGHT);
        }
        // internal
        Msg::DirectoryRead {
            session_info,
            entries,
            navigated_to,
        } => {
            model.update_entries_for_session(session_info, entries, navigated_to);
        }
        Msg::ReadingDirFailed(error) => {
            model.user_msg = Some(UserMsg::error(format!("reading directory failed: {error}")));
        }
    }

    if let Some(message) = &mut model.user_msg {
        let clear = if message.frames_left == 0 {
            true
        } else {
            message.frames_left -= 1;
            false
        };

        if clear {
            model.user_msg = None;
        }
    }

    cmds
}
