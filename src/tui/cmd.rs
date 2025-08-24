use super::common::SessionInfo;
use super::msg::Msg;
use crate::services::list_entries_at_directory;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum Cmd {
    ReadDir((SessionInfo, bool)),
}

pub async fn handle_command(command: Cmd, event_tx: Sender<Msg>) {
    match command {
        Cmd::ReadDir((session_info, navigated_to)) => {
            tokio::spawn(async move {
                let msg = match list_entries_at_directory(&session_info.path).await {
                    Ok(r) => Msg::DirectoryRead {
                        session_info,
                        entries: r,
                        navigated_to,
                    },
                    Err(e) => Msg::ReadingDirFailed(e.to_string()),
                };

                let _ = event_tx.try_send(msg);
            });
        }
    }
}
