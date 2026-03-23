use std::{
    collections::HashMap, fs::File, io::{BufRead, BufReader, Read, Seek}, path::{Path, PathBuf}
};

use serde::{Deserialize, Serialize};

use crate::types::{AssemblyBlock, Config, DataBlock, TextBlock};
use bitvec::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assembly {
    pub path: PathBuf,
    pub config: Config,
    pub blocks: Vec<AssemblyBlock>,
    pub block_map: HashMap<String, usize>,
}

#[derive(Debug)]
struct State {
    pub pointer: u64,
    pub allocated: Vec<u64>
}

impl Default for State {
    fn default() -> Self {
        Self { pointer: 0, allocated: Vec::new() }
    }
}

impl Assembly {
    fn parse_text_block(line: String, input: &mut BufReader<File>, config: &Config, state: &mut State) {// -> TextBlock {

    }

    fn parse_data_block(line: String, input: &mut BufReader<File>, config: &Config, state: &mut State) {// -> DataBlock {
        let header_parts: Vec<&str> = line.split(" ").collect();
        let name = header_parts[1].to_string();
        let (start_str, length_str) = header_parts[2].split_once("..").unwrap();
        let align = header_parts.get(3).is_some_and(|v| *v == "align");
        let mut total_length = 0u64;

        let mut entries: Vec<(String, BitVec<u8, Msb0>)> = Vec::new();
        loop {
            let mut line = String::new();
            if let Ok(length) = input.read_line(&mut line) {
                let line = line.trim();
                if line.starts_with(".data") || line.starts_with(".text") {
                    input.seek_relative(i64::try_from(length).unwrap() * -1).unwrap();
                    break;
                }

                if line.starts_with("\"\"\"") {
                    let mut full_str = line.clone().to_string();
                    while !(full_str.ends_with("\"\"\"\n") && full_str.len() > 4) {
                        input.read_line(&mut full_str).expect("Unexpected EOF");
                    }

                    let full_str = full_str.trim().trim_matches('"').to_string();
                    entries.push((full_str.clone(), BitVec::<u8, Msb0>::from_slice(full_str.as_bytes())));
                }
            } else {
                break;
            }
        }
        
    }

    pub fn parse(input_file: impl AsRef<Path>, config: Config) {
        let path = input_file.as_ref().to_path_buf();
        let mut input = BufReader::new(
            File::open(path.clone())
                .expect("The specified input file doesnt exist or couldn't be read!"),
        );
        let mut blocks: Vec<AssemblyBlock> = vec![];
        let mut block_map: HashMap<String, usize> = HashMap::new();
        let mut state = State::default();

        loop {
            let mut line = String::new();
            if let Ok(_) = input.read_line(&mut line) {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.len() == 0 {
                    continue;
                }

                if trimmed.starts_with(".data") {
                    let block = Self::parse_data_block(trimmed.to_string(), &mut input, &config, &mut state);
                } else if trimmed.starts_with(".text") {
                    let block = Self::parse_text_block(trimmed.to_string(), &mut input, &config, &mut state);
                } else {
                    panic!("Unknown top-level item {trimmed}");
                }
            } else {
                break;
            }
        }
    }
}
