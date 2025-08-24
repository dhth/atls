use super::cmd::{Cmd, handle_command};
use super::common::*;
use super::model::*;
use super::msg::{Msg, get_event_handling_msg};
use super::update::update;
use super::view::view;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::poll;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

const EVENT_POLL_DURATION_MS: u64 = 16;

pub async fn run(root: PathBuf) -> anyhow::Result<()> {
    let mut tui = AppTui::new(root)?;
    tui.run().await
}

struct AppTui {
    pub(super) terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    pub(super) event_tx: Sender<Msg>,
    pub(super) event_rx: Receiver<Msg>,
    pub(super) model: Model,
}

impl AppTui {
    pub fn new(root: PathBuf) -> anyhow::Result<Self> {
        let terminal = ratatui::try_init()?;
        let (event_tx, event_rx) = mpsc::channel(10);

        let (width, height) = ratatui::crossterm::terminal::size()?;

        let terminal_dimensions = TerminalDimensions { width, height };

        let debug = std::env::var("ATLS_DEBUG").unwrap_or_default().trim() == "1";

        let model = Model::new(root, terminal_dimensions, debug);

        Ok(Self {
            terminal,
            event_tx,
            event_rx,
            model,
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let _ = self.terminal.clear();

        // first render
        self.terminal.draw(|f| view(&mut self.model, f))?;
        self.model.render_counter += 1;

        let mut initial_cmds = vec![];
        match self.model.current_session() {
            Session::Uninitialized => {}
            Session::Initialized {
                path,
                entries: _,
                state: _,
            } => {
                initial_cmds.push(Cmd::ReadDir((
                    SessionInfo {
                        index: 0,
                        path: path.clone(),
                    },
                    false,
                )));
            }
        }

        for cmd in initial_cmds {
            handle_command(cmd.clone(), self.event_tx.clone()).await;
        }

        loop {
            tokio::select! {
                Some(message) = self.event_rx.recv() => {
                    let cmds = update(&mut self.model, message);

                    if self.model.running_state == RunningState::Done {
                        break;
                    }

                        self.terminal.draw(|f| view(&mut self.model, f))?;
                        self.model.render_counter += 1;

                    for cmd in cmds {
                        handle_command(cmd.clone(), self.event_tx.clone()).await;
                    }
                }

                Ok(ready) = tokio::task::spawn_blocking(|| poll(Duration::from_millis(EVENT_POLL_DURATION_MS))) => {
                    match ready {
                        Ok(true) => {
                            // non blocking read since poll returned Ok(true)
                            let event = ratatui::crossterm::event::read()?;
                            self.model.event_counter += 1;
                            if let Some(handling_msg) = get_event_handling_msg(&self.model, event) {
                                self.event_tx.try_send(handling_msg)?;
                            }
                        }
                        Ok(false) => continue,
                        Err(e) => {
                                return Err(anyhow::anyhow!(e));
                        }
                    }
                }
            }
        }

        self.exit()
    }

    fn exit(&mut self) -> anyhow::Result<()> {
        ratatui::try_restore()?;
        Ok(())
    }
}
