use std::process::ExitCode;

async fn execute() -> anyhow::Result<()> {
    let cmd = appbiotic_api_secrets_onepassword_cli::cli();
    let matches = cmd.get_matches();
    appbiotic_api_secrets_onepassword_cli::exec(&matches).await
}

#[tokio::main]
async fn main() -> ExitCode {
    match execute().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
