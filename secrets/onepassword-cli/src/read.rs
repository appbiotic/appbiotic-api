use std::io::{stdout, Write};

use anyhow::Context;
use appbiotic_api_secrets_onepassword::{FieldResource, OnePassword, ReadRequest};
use clap::{ArgMatches, Command};

pub static NAME: &str = "read";

pub fn cmd() -> Command {
    Command::new(NAME)
        .about("Reads secret content from a vault item")
        .arg(args::vault())
        .arg(args::item())
        .arg(args::field())
}

pub async fn exec(matches: &ArgMatches, client: Box<dyn OnePassword>) -> anyhow::Result<()> {
    let vault = matches.get_one::<String>(args::VAULT).unwrap();
    let item = matches.get_one::<String>(args::ITEM).unwrap();
    let field = matches.get_one::<String>(args::FIELD).unwrap();

    let request = ReadRequest {
        resource: FieldResource {
            vault: vault.to_string(),
            item: item.to_string(),
            field: field.to_string(),
        },
    };

    let value = client.read(request).await?;
    stdout()
        .write_all(&value.content)
        .context("Failed to write secret content")
}

mod args {
    use clap::{builder::NonEmptyStringValueParser, Arg};

    pub static VAULT: &str = "vault";
    pub static ITEM: &str = "item";
    pub static FIELD: &str = "field";

    pub fn vault() -> Arg {
        Arg::new(VAULT)
            .help("The vault for retrieving the secret content")
            .long(VAULT)
            .value_name("VAULT")
            .value_parser(NonEmptyStringValueParser::new())
            .required(true)
    }

    pub fn item() -> Arg {
        Arg::new(ITEM)
            .help("The item containing the secret content")
            .long(ITEM)
            .value_parser(NonEmptyStringValueParser::new())
            .value_name("ITEM")
            .required(true)
    }

    pub fn field() -> Arg {
        Arg::new(FIELD)
            .help("The field containing the secret content")
            .long(FIELD)
            .value_parser(NonEmptyStringValueParser::new())
            .value_name("FIELD")
            .required(true)
    }
}
