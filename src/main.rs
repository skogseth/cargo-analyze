use std::collections::VecDeque;
use std::ffi::{OsStr, OsString};
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context};
use clap::Parser;

use cargo_analyze::Metadata;

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    let extra_args = match &cli.manifest_path {
        Some(path) => vec![OsStr::new("--manifest-path"), path.as_os_str()],
        None => Vec::new(),
    };

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
        return Err(anyhow!("`cargo build` failed: {}", output.status));
    }

    let reader = BufReader::new(&output.stdout[..]);
    let metadata = Metadata::from_reader(reader);

    // Makeshift `try`-block
    || -> Result<(), anyhow::Error> {
        let mut stdout = std::io::stdout().lock();

        stdout.write_all(b"Libraries linked by rustc:\n")?;
        writeln!(stdout, "{}", metadata.linked_libs)?;

        if !cli.ignore_binary_analysis {
            for executable in &metadata.executables {
                let mut libs = cargo_analyze::binary::analyze(executable.as_std_path())?;
                libs.sort_unstable();
                libs.retain(|e| e != "self");

                let filename = executable
                    .file_name()
                    .expect("Failed to get filename of executable");
                writeln!(stdout, "Libraries linked to executable `{filename}`:")?;

                for lib in libs {
                    writeln!(stdout, " - {lib}")?;
                }
            }
        }

        Ok(())
    }()
    .expect("Failed to write to standard output");

    Ok(())
}

#[derive(Debug, Parser)]
#[command(bin_name = "cargo")]
struct Cli {
    #[arg(long, value_name = "PATH")]
    manifest_path: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    ignore_binary_analysis: bool,
}

impl Cli {
    fn parse() -> Cli {
        let mut args: VecDeque<OsString> = std::env::args_os().collect();

        // Strip extra `analyze`, if invoked via cargo:
        // `cargo-analyze ...` -> ["cargo-analyze", ...]
        // `cargo analyze ...` -> ["cargo-analyze", "analyze", ...]
        if args.get(1).is_some_and(|arg| arg == OsStr::new("analyze")) {
            args.swap_remove_front(1).expect("this element must exist");
        }

        // This function will ignore it's first argument (as it is assumed to be the program name)
        Cli::parse_from(args)
    }
}
