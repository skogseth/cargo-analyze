use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
use std::{ffi::OsString, process::Stdio};

use anyhow::{anyhow, Context};
use clap::Parser;

use cargo_analyze::*;

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    let extra_args = cli
        .manifest_path
        .map(|path| vec![OsString::from("--manifest-path"), path.into_os_string()])
        .unwrap_or_else(Vec::new);

    let output = Command::new("cargo")
        .arg("build")
        .args(["--message-format", "json"])
        .args(extra_args)
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn `cargo build`")?
        .wait_with_output()
        .context("failed whilst waiting for `cargo build`")?;

    if !output.status.success() {
        let msg = anyhow!("`cargo build` failed: {}", output.status);
        return Err(msg);
    }

    let reader = BufReader::new(&output.stdout[..]);
    let libs = LinkedLibs::from_metadata(reader);
    println!("{libs}");

    Ok(())
}

#[derive(Debug, Clone, Parser)]
struct Cli {
    /// Currently ignored
    #[arg(long)]
    manifest_path: Option<PathBuf>,
}
