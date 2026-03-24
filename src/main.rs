use std::error::Error;

use clap::Parser;

use crate::parser::Assembly;

pub mod types;
pub mod cli;
pub mod parser;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = cli::SmolASM::parse();
    let config = types::Config::load_config(cli.config.clone())?;
    let assembled = Assembly::parse(cli.input.clone(), config);

    match cli.action {
        cli::Actions::Analyze => assembled.analyze(cli.config.clone()),
        cli::Actions::Assemble { output } => (),
    }
    Ok(())
}
