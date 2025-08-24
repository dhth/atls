mod common;
mod domain;
mod log;
mod services;
mod tui;

use anyhow::Context;
use log::setup_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging().context("couldn't set up logging")?;

    let path = tokio::fs::canonicalize(".")
        .await
        .context("couldn't canonicalize path")?;

    tui::run(path).await?;

    Ok(())
}
