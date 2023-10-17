#![feature(pattern)]

use std::process::{self, ExitCode};

use anyhow::{bail, Context};
use clap::Parser;
use color_print::{self, cformat, cprint, cprintln};

use mmv_lib::{DestinationPathTemplate, SourcePathPattern};

/// multi-mv: rename multiple files matching a pattern
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct CLIArgs {
    /// Source pattern. '*' matches any number of any characters.
    source_pattern: SourcePathPattern,

    #[arg(help = cformat!(
        "Destination template. \
        Markers in format of <green>#NUM</> are replaced by characters matched \
        by a corresponding, i.e. <green>NUM</>th, wildcard.")
    )]
    destination_template: String,

    /// Replace existing files
    #[arg(short, long)]
    force: bool,
}

fn main() -> anyhow::Result<process::ExitCode> {
    let cli_args = CLIArgs::parse();

    let compiled_destination_pattern = DestinationPathTemplate::compile(
        cli_args.destination_template.as_str(),
        cli_args
            .source_pattern
            .wildcards_number()
            .try_into()
            .context("Too many wildcards: number of wildcards must be between 0 and 255")?,
    );

    let calculated_source_destination = cli_args
        .source_pattern
        .matching_files(std::env::current_dir()?.as_path())?
        .into_iter()
        .map(|(source_path, flagments_to_substitute)| {
            (
                source_path,
                compiled_destination_pattern.substitute(
                    &flagments_to_substitute
                        .iter()
                        .map(|string| string.as_str())
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect::<Vec<_>>();

    if calculated_source_destination.is_empty() {
        bail!(cformat!(
            "No files matching pattern <green>{}</>",
            cli_args.source_pattern
        ));
    }

    if !compiled_destination_pattern
        .directory
        .as_os_str()
        .is_empty()
        && !compiled_destination_pattern.directory.exists()
    {
        bail!(
            "Target directory {:#?} doesn't exist",
            compiled_destination_pattern.directory
        );
    }

    let mut failed_at_least_once = false;
    for (source, destination) in calculated_source_destination {
        cprint!("Moving <yellow>{source:?}</> -> <green>{destination:?}</>: ");
        if !cli_args.force && destination.exists() {
            cprintln!("<yellow>Skip</>: file already exists");
            continue;
        }
        if let Err(error) = std::fs::rename(source, destination) {
            cprintln!("<red>Failed</>: {error:#}");
            failed_at_least_once = true;
        } else {
            cprintln!("<green>Done</>");
        }
    }

    Ok(ExitCode::from(failed_at_least_once as u8))
}
