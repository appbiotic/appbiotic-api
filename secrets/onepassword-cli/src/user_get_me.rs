use std::io::{stdout, BufWriter};

use anyhow::Context;
use appbiotic_api_secrets_onepassword::OnePassword;
use clap::{ArgMatches, Command};

pub static NAME: &str = "user-get-me";

pub fn cmd() -> Command {
    Command::new(NAME).about("Displays info about the currently authenticated user")
}

pub async fn exec(_matches: &ArgMatches, client: Box<dyn OnePassword>) -> anyhow::Result<()> {
    let value = client.user_get_me().await?;
    serde_json::to_writer_pretty(BufWriter::new(stdout()), &value)
        .context("Failed to write user JSON")
}
