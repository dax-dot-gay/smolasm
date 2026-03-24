use std::error::Error;

use clap::Parser;

use crate::parser::Assembly;

pub mod types;
pub mod cli;
pub mod parser;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = cli::SmolASM::parse();
    let config = types::Config::load_config(cli.config.clone())?;
    Assembly::parse(cli.input.clone(), config);
    Ok(())
}
