use std::path::Path;

use clap::Parser;
use colored::Colorize;

use crate::parser::Assembly;

pub mod cli;
pub mod formats;
pub mod parser;
pub mod types;
pub mod error;
pub use error::*;

fn main_internal() -> crate::Result<()> {
    let cli = cli::SmolASM::parse();
    let config = types::Config::load_config(cli.config.clone())?;
    let assembled = Assembly::parse(cli.input.clone(), config)?;

    match cli.action.clone() {
        cli::Actions::Analyze => assembled.analyze(cli.config.clone()),
        cli::Actions::Asm { output } => formats::assemble(
            assembled,
            output.unwrap_or(format!(
                "{}.obj",
                Path::new(&cli.input.clone())
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            )),
        ),
    }
    Ok(())
}

fn main() -> () {
    match main_internal() {
        Ok(_) => (),
        Err(e) => println!("{}\t{e}", "Error:".bright_red().bold())
    }
}
