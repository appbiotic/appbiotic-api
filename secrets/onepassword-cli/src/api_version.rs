use std::io::{stdout, BufWriter};

use anyhow::Context;
use appbiotic_api_secrets_onepassword::{ApiVersionRequest, OnePassword};
use clap::{ArgMatches, Command};

pub static NAME: &str = "api-version";

pub fn cmd() -> Command {
    Command::new(NAME).about("Displays the version of the OnePassword API in use")
}

pub async fn exec(_matches: &ArgMatches, client: Box<dyn OnePassword>) -> anyhow::Result<()> {
    let value = client.api_version(ApiVersionRequest {}).await?;
    serde_json::to_writer_pretty(BufWriter::new(stdout()), &value)
        .context("Failed to write user JSON")
}
