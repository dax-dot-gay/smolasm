use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct SmolASM {
    /// The path to the input file to assemble
    pub input: String,

    /// An alternate path to the config file
    #[arg(short, long, default_value_t = String::from("config.kdl"))]
    pub config: String,

    /// An alternate name for the output file (defaults to "<input name>.obj")
    #[arg(short, long)]
    pub output: Option<String>,
}
