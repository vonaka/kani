// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Implements the logic related to the playback subcommand
//! This can be achieved with <kani|cargo kani> playback --test <test_name>

use crate::args::common::Verbosity;
use crate::args::playback_args::{CargoPlaybackArgs, KaniPlaybackArgs, MessageFormat};
use crate::call_cargo::cargo_config_args;
use crate::call_single_file::{LibConfig, base_rustc_flags};
use crate::session::{InstallType, lib_playback_folder, setup_cargo_command};
use crate::util::args::{CargoArg, CommandWrapper, PassTo, RustcArg};
use crate::{session, util};
use anyhow::Result;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::debug;

pub fn playback_cargo(args: CargoPlaybackArgs) -> Result<()> {
    cargo_test(args)
}

pub fn playback_standalone(args: KaniPlaybackArgs) -> Result<()> {
    let install = InstallType::new()?;
    let artifact = build_test(&install, &args)?;
    debug!(?artifact, "playback_standalone");

    if !args.playback.common_opts.quiet() {
        print_artifact(&artifact, args.playback.message_format)
    }

    if !args.playback.only_codegen {
        run_test(&artifact, &args)?;
    }

    Ok(())
}

fn print_artifact(artifact: &Path, format: MessageFormat) {
    match format {
        MessageFormat::Json => {
            println!(r#"{{"artifact":"{}"}}"#, artifact.display())
        }
        MessageFormat::Human => {
            println!("Executable {}", artifact.display())
        }
    }
}

fn run_test(exe: &Path, args: &KaniPlaybackArgs) -> Result<()> {
    let mut cmd = Command::new(exe);

    if args.playback.common_opts.verbose()
        && !args.playback.test_args.contains(&"--nocapture".to_string())
    {
        // Repeated arguments cause an execution error.
        cmd.arg("--nocapture");
    }

    cmd.args(&args.playback.test_args);

    session::run_terminal(&args.playback.common_opts, cmd)?;
    Ok(())
}

fn build_test(install: &InstallType, args: &KaniPlaybackArgs) -> Result<PathBuf> {
    const TEST_BIN_NAME: &str = "kani_concrete_playback";

    if !args.playback.common_opts.quiet() {
        util::info_operation("Building", args.input.to_string_lossy().deref());
    }

    let mut rustc_args = base_rustc_flags(LibConfig::new(lib_playback_folder()?));
    rustc_args.push("--test".into());
    rustc_args.push(RustcArg::from(&args.input));
    rustc_args.push(format!("--crate-name={TEST_BIN_NAME}").into());

    if args.playback.common_opts.verbose() {
        rustc_args.push("--verbose".into());
    }

    if args.playback.message_format == MessageFormat::Json {
        rustc_args.push("--error-format=json".into());
    }

    let mut cmd = Command::new(install.kani_compiler()?);
    cmd.pass_rustc_args(&rustc_args, PassTo::OnlyLocalCrate);

    session::run_terminal(&args.playback.common_opts, cmd)?;

    Ok(PathBuf::from(TEST_BIN_NAME).canonicalize()?)
}

/// Invokes cargo test using Kani compiler and the provided arguments.
fn cargo_test(args: CargoPlaybackArgs) -> Result<()> {
    let install = InstallType::new()?;
    let mut cmd = setup_cargo_command()?;

    let rustc_args = base_rustc_flags(LibConfig::new(lib_playback_folder()?));
    let mut cargo_args: Vec<CargoArg> = vec!["test".into()];

    if args.playback.common_opts.verbose() {
        cargo_args.push("-vv".into());
    } else if args.playback.common_opts.quiet {
        cargo_args.push("--quiet".into())
    }

    if args.playback.message_format == MessageFormat::Json {
        cargo_args.push("--message-format=json".into());
    }

    if args.playback.only_codegen {
        cargo_args.push("--no-run".into());
    }

    cargo_args.append(&mut args.cargo.to_cargo_args());
    cargo_args.append(&mut cargo_config_args());

    // These have to be the last arguments to cargo test.
    if !args.playback.test_args.is_empty() {
        cargo_args.push("--".into());
        cargo_args.extend(args.playback.test_args.iter().map(|arg| arg.into()));
    }

    // Arguments that will only be passed to the target package.
    cmd.pass_cargo_args(&cargo_args)
        .env("RUSTC", &install.kani_compiler()?)
        .pass_rustc_args(&rustc_args, PassTo::AllCrates)
        .env("CARGO_TERM_PROGRESS_WHEN", "never");

    session::run_terminal(&args.playback.common_opts, cmd)?;
    Ok(())
}
