use std::{fs::read_to_string, path::PathBuf};

use anyhow::{bail, Context};
use appbiotic_api_secrets_onepassword::OnePassword;
use appbiotic_api_secrets_onepassword_client::tokio::OnePasswordClient;
use clap::{command, ArgMatches, Command};

pub mod api_version;
pub mod item_get;
pub mod user_get_me;

pub fn cli() -> Command {
    command!()
        .about("Test utility for OnePassword API functionality")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(args::service_account_token().required(true))
        .subcommand(api_version::cmd())
        .subcommand(item_get::cmd())
        .subcommand(user_get_me::cmd())
}

pub async fn exec(matches: &ArgMatches) -> anyhow::Result<()> {
    let token_file = matches
        .get_one::<PathBuf>(args::ONEPASSWORD_SERVICE_ACCOUNT_TOKEN_FILE)
        .unwrap();
    let service_token = read_to_string(token_file)
        .with_context(|| format!("Failed to read token file: {token_file:?}"))?;

    let client: Box<dyn OnePassword> = Box::new(OnePasswordClient::new(service_token));

    if let Some(matches) = matches.subcommand_matches(api_version::NAME) {
        api_version::exec(matches, client).await
    } else if let Some(matches) = matches.subcommand_matches(item_get::NAME) {
        item_get::exec(matches, client).await
    } else if let Some(matches) = matches.subcommand_matches(user_get_me::NAME) {
        user_get_me::exec(matches, client).await
    } else {
        bail!("Expected valid subcommand");
    }
}

pub mod args {
    use clap::{builder::PathBufValueParser, Arg};

    pub static ONEPASSWORD_SERVICE_ACCOUNT_TOKEN_FILE: &str =
        "onepassword-service-account-token-file";

    pub fn service_account_token() -> Arg {
        Arg::new(ONEPASSWORD_SERVICE_ACCOUNT_TOKEN_FILE)
            .help("Path to the OnePassword service account token for auth")
            .value_parser(PathBufValueParser::new())
            .long(ONEPASSWORD_SERVICE_ACCOUNT_TOKEN_FILE)
            .env("ONEPASSWORD_SERVICE_ACCOUNT_TOKEN_FILE")
    }
}
