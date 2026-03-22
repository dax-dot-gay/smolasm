use std::error::Error;

use clap::Parser;

pub mod types;
pub mod cli;
pub mod parser;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = cli::SmolASM::parse();
    let config = types::Config::load_config(cli.config.clone())?;
    let config_display = serde_json::to_string_pretty(&config)?;
    println!("{config_display}");
    Ok(())
}
