use std::{path::PathBuf, process::ExitCode};

use anyhow::Context;
use appbiotic_api_protogen_spec::ProtogenSpec;

fn main() -> ExitCode {
    match build_all() {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:?}");
            ExitCode::FAILURE
        }
    }
}

fn build_all() -> anyhow::Result<()> {
    build_prost_serde()?;
    Ok(())
}

fn build_prost_serde() -> anyhow::Result<()> {
    let protogen_spec: ProtogenSpec = serde_json::from_str(include_str!("../../protogen.json"))
        .context("Failed to deserialize package_spec.json")?;
    let package_name = env!("CARGO_PKG_NAME");
    let dependencies = vec![];
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").context("Failed to get OUT_DIR")?);
    appbiotic_api_prost_serde_build::build(protogen_spec, package_name, dependencies, out_dir)?;
    Ok(())
}
