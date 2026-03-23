use std::{
    collections::HashMap, fs::File, io::{BufRead, BufReader, Read}, path::{Path, PathBuf}
};

use serde::{Deserialize, Serialize};

use crate::types::{AssemblyBlock, Config, DataBlock, TextBlock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assembly {
    pub path: PathBuf,
    pub config: Config,
    pub blocks: Vec<AssemblyBlock>,
    pub block_map: HashMap<String, usize>,
}

impl Assembly {
    fn parse_text_block(line: String, input: &mut BufReader<File>, config: &Config) {// -> TextBlock {

    }

    fn parse_data_block(line: String, input: &mut BufReader<File>, config: &Config) {// -> DataBlock {
        
    }

    pub fn parse(input_file: impl AsRef<Path>, config: Config) {
        let path = input_file.as_ref().to_path_buf();
        let mut input = BufReader::new(
            File::open(path.clone())
                .expect("The specified input file doesnt exist or couldn't be read!"),
        );
        let mut blocks: Vec<AssemblyBlock> = vec![];
        let mut block_map: HashMap<String, usize> = HashMap::new();

        loop {
            let mut line = String::new();
            if let Ok(_) = input.read_line(&mut line) {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.len() == 0 {
                    continue;
                }

                if trimmed.starts_with(".data") {
                    let block = Self::parse_data_block(trimmed.to_string(), &mut input, &config);
                } else if trimmed.starts_with(".text") {
                    let block = Self::parse_text_block(trimmed.to_string(), &mut input, &config);
                } else {
                    panic!("Unknown top-level item {trimmed}");
                }
            } else {
                break;
            }
        }
    }
}
