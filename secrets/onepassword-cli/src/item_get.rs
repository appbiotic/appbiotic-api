use std::io::{stdout, BufWriter};

use anyhow::Context;
use appbiotic_api_secrets_onepassword::OnePassword;
use clap::{ArgMatches, Command};

pub static NAME: &str = "item-get";

pub fn cmd() -> Command {
    Command::new(NAME)
        .about("Gets a secret from a vault")
        .arg(args::vault())
        .arg(args::item())
}

pub async fn exec(matches: &ArgMatches, client: Box<dyn OnePassword>) -> anyhow::Result<()> {
    let vault = matches.get_one::<String>(args::VAULT).unwrap();
    let item = matches.get_one::<String>(args::ITEM).unwrap();
    let value = client.item_get(vault.to_string(), item.to_string()).await?;
    serde_json::to_writer_pretty(BufWriter::new(stdout()), &value)
        .context("Failed to write user JSON")
}

mod args {
    use clap::{builder::NonEmptyStringValueParser, Arg};

    pub static ITEM: &str = "item";
    pub static VAULT: &str = "vault";

    pub fn item() -> Arg {
        Arg::new(ITEM)
            .help("The item to retrieve")
            .value_parser(NonEmptyStringValueParser::new())
            .value_name("ITEM")
            .required(true)
    }

    pub fn vault() -> Arg {
        Arg::new(VAULT)
            .help("The vault for retrieving the secret")
            .long(VAULT)
            .value_name("VAULT")
            .value_parser(NonEmptyStringValueParser::new())
            .required(true)
    }
}
