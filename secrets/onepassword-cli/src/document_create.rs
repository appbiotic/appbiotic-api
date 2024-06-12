use std::{
    io::{stdout, BufWriter},
    path::PathBuf,
};

use anyhow::Context;
use appbiotic_api_secrets_onepassword::{DocumentCreateRequest, OnePassword};
use clap::{ArgMatches, Command};

pub static NAME: &str = "document-create";

pub fn cmd() -> Command {
    Command::new(NAME)
        .about("Creates a document secret in a vault")
        .arg(args::vault())
        .arg(args::title())
        .arg(args::file_name())
        .arg(args::file_path_positional())
}

pub async fn exec(matches: &ArgMatches, client: Box<dyn OnePassword>) -> anyhow::Result<()> {
    let vault = matches.get_one::<String>(args::VAULT).unwrap().to_owned();
    let title = matches.get_one::<String>(args::TITLE).unwrap().to_owned();
    let file_name = matches
        .get_one::<String>(args::FILE_NAME)
        .unwrap()
        .to_owned();
    let file_path = matches
        .get_one::<PathBuf>(args::FILE_PATH)
        .unwrap()
        .to_owned();

    let data = std::fs::read(file_path)?;

    let request = DocumentCreateRequest {
        title,
        vault,
        file_name,
        data,
        tags: vec![],
    };

    let value = client.document_create(request).await?;
    serde_json::to_writer_pretty(BufWriter::new(stdout()), &value)
        .context("Failed to write created document JSON")
}

mod args {
    use clap::{
        builder::{NonEmptyStringValueParser, PathBufValueParser},
        Arg,
    };

    pub static FILE_PATH: &str = "file-path";
    pub static FILE_NAME: &str = "file-name";
    pub static TITLE: &str = "title";
    pub static VAULT: &str = "vault";

    pub fn file_path_positional() -> Arg {
        Arg::new(FILE_PATH)
            .required(true)
            .value_name("FILE_PATH")
            .value_parser(PathBufValueParser::new())
    }

    pub fn file_name() -> Arg {
        Arg::new(FILE_NAME)
            .value_parser(NonEmptyStringValueParser::new())
            .value_name("FILE_NAME")
            .long(FILE_NAME)
            .required(true)
    }

    pub fn title() -> Arg {
        Arg::new(TITLE)
            .help("The document item's title to create")
            .value_parser(NonEmptyStringValueParser::new())
            .value_name("TITLE")
            .long(TITLE)
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
