use std::{
    collections::HashMap,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::types::{AssemblyBlock, Config};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assembly {
    pub path: PathBuf,
    pub config: Config,
    pub blocks: Vec<AssemblyBlock>,
    pub block_map: HashMap<String, usize>,
}

impl Assembly {
    pub fn parse(input_file: impl AsRef<Path>, config: Config) {
        let path = input_file.as_ref().to_path_buf();
        let input = BufReader::new(
            std::fs::File::open(path.clone())
                .expect("The specified input file doesnt exist or couldn't be read!"),
        );
        let mut blocks: Vec<AssemblyBlock> = vec![];
        let mut block_map: HashMap<String, usize> = HashMap::new();

        loop {}
    }
}
