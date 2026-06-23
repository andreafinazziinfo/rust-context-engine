use anyhow::Result;
use clap::Parser;

mod agents;
mod artifact;
mod benchmark;
mod cli;
mod dashboard;
mod dispatch;
mod distiller;
mod doctor;
mod dotnet;
mod filter_pipeline;
mod index_cli;
mod plugins;
mod rewrite;
mod setup;
mod sync_rules;
mod validate;

#[cfg(test)]
mod fuzz_tests;

fn main() {
    let result: Result<()> = dispatch::dispatch(cli::Cli::parse().command);
    if let Err(e) = result {
        eprintln!("rtk: {e}");
        std::process::exit(1);
    }
}
