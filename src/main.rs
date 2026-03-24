use std::{error::Error, path::Path};

use clap::Parser;

use crate::parser::Assembly;

pub mod cli;
pub mod formats;
pub mod parser;
pub mod types;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = cli::SmolASM::parse();
    let config = types::Config::load_config(cli.config.clone())?;
    let assembled = Assembly::parse(cli.input.clone(), config);

    match cli.action.clone() {
        cli::Actions::Analyze => assembled.analyze(cli.config.clone()),
        cli::Actions::Asm { output } => formats::assemble(
            assembled,
            output.unwrap_or(format!(
                "{}.out",
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
