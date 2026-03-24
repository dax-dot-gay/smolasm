use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::parser::Assembly;

mod ritarch;

pub trait OutputFormatter {
    fn assemble(assembly: Assembly, path: PathBuf) -> ();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum OutputFormat {
    RITARCH,
}

impl From<String> for OutputFormat {
    fn from(value: String) -> Self {
        match value.to_lowercase().trim() {
            "ritarch" => Self::RITARCH,
            other => panic!("Unknown output format: {other}"),
        }
    }
}

pub fn assemble(assembly: Assembly, path: impl AsRef<Path>) {
    let path = path.as_ref().to_path_buf();
    match OutputFormat::from(assembly.config.system.format.clone()) {
        OutputFormat::RITARCH => ritarch::RitArch::assemble(assembly, path),
    }
}
