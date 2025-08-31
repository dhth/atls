use super::common::SessionInfo;

use super::msg::Msg;
use crate::domain::FSOperation;
use crate::services;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum Cmd {
    RunFSOperation(FSOperation),
    ReadDir((SessionInfo, bool)),
}

pub async fn handle_command(command: Cmd, event_tx: Sender<Msg>) {
    match command {
        Cmd::RunFSOperation(operation) => {
            tokio::task::spawn_blocking(move || {
                let result = match operation {
                    FSOperation::Copy { items, destination } => {
                        services::copy_entries_to_destination(
                            items.as_slice(),
                            destination.as_path(),
                        )
                    }
                    FSOperation::Move { items, destination } => {
                        services::move_entries_to_destination(
                            items.as_slice(),
                            destination.as_path(),
                        )
                    }
                };

                let _ = event_tx.try_send(Msg::FSOperationFinished(result));
            });
        }
        Cmd::ReadDir((session_info, navigated_to)) => {
            tokio::spawn(async move {
                let msg = match services::list_entries_at_directory(&session_info.path).await {
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
