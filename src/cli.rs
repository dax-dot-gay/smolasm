use clap::{Parser, Subcommand};

#[derive(Subcommand, Clone, Debug)]
pub enum Actions {
    /// Generate information about the 
    Analyze,

    /// Generate machine code based on the input file and config
    Assemble {
        /// An alternate name for the output file (defaults to "<input name>.obj")
        #[arg(short, long)]
        output: Option<String>,
    }
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct SmolASM {
    #[command(subcommand)]
    pub action: Actions,

    /// The path to the input file to process
    pub input: String,

    /// An alternate path to the config file
    #[arg(short, long, default_value_t = String::from("config.kdl"))]
    pub config: String
}
