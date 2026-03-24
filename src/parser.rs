use std::{
    collections::HashMap, fs::File, io::{BufRead, BufReader, Read, Seek}, num::ParseIntError, path::{Path, PathBuf}
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
    fn parse_number(value: impl AsRef<str>) -> Result<u64, ParseIntError> {
        let value = value.as_ref().trim().to_string();
        if value.starts_with("0x") {
            u64::from_str_radix(value.split_once('x').unwrap().1, 16)
        } else if value.starts_with("0b") {
            u64::from_str_radix(value.split_once('b').unwrap().1, 2)
        } else {
            u64::from_str_radix(value.as_str(), 10)
        }
    }

    fn parse_text_block(line: String, input: &mut BufReader<File>, config: &Config, state: &mut State) {// -> TextBlock {

    }

    fn parse_data_block(line: String, input: &mut BufReader<File>, config: &Config, state: &mut State) -> DataBlock {
        let header_parts: Vec<&str> = line.split(" ").collect();
        let name = header_parts[1].to_string();
        let (start_str, length_str) = header_parts[2].split_once("..").unwrap();
        let mut total_length = 0u64;

        let mut entries: Vec<(String, BitVec<u8, Msb0>)> = Vec::new();
        loop {
            let mut line = String::new();
            if let Ok(length) = input.read_line(&mut line) {
                let line = line.trim_start();
                if line.starts_with(".data") || line.starts_with(".text") {
                    input.seek_relative(i64::try_from(length).unwrap() * -1).unwrap();
                    break;
                }

                if line.starts_with("\"\"\"") {
                    let mut full_str = line.to_string();
                    while !(full_str.ends_with("\"\"\"\n") && full_str.len() > 4) {
                        let mut rline = String::new();
                        input.read_line(&mut rline).expect("Unexpected EOF");
                        full_str += rline.trim_start();
                    }

                    let full_str = full_str.trim().trim_matches('"').to_string() + "\0";
                    entries.push((full_str.clone(), BitVec::from_slice(full_str.as_bytes())));
                } else if line.starts_with('"') {
                    if line.ends_with("\"\n") && line.len() > 2 {
                        let trimmed = line.trim().trim_matches('"').to_string() + "\0";
                        entries.push((trimmed.clone(), BitVec::from_slice(trimmed.as_bytes())));
                    } else {
                        panic!("Single line strings must terminate with double quotes");
                    }
                } else if line.starts_with("0x") && line.len() > 3 {
                    let trimmed = line.trim().split_once('x').unwrap().1.to_string();
                    let raw = hex::decode(trimmed).expect("Expected valid hex string");
                    entries.push((line.trim().to_string(), BitVec::from_vec(raw)));
                } else if line.starts_with("0b") && line.len() > 3 {
                    let trimmed = line.trim().split_once('b').unwrap().1.to_string();
                    let bits = trimmed.chars().map(|c| c == '1').collect::<BitVec<u8, Msb0>>();
                    entries.push((line.trim().to_string(), bits));
                } else if let Ok(number) = line.trim().parse::<u64>() {
                    entries.push((line.trim().to_string(), BitVec::from_slice(&number.to_le_bytes())));
                } else if line.trim().len() == 0 || line.trim().starts_with("//") {
                    continue;
                } else {
                    panic!("Invalid assembly line: {}", line.trim());
                }
            } else {
                break;
            }
        }

        for (entry_name, bits) in entries.clone() {
            total_length += u64::try_from(bits.len()).unwrap().div_ceil(config.system.hardware.word_size);
        }

        let length = if length_str == "auto" {
            total_length
        } else {
            let parsed = Self::parse_number(length_str).expect(format!("Expected 'auto' or valid length, but got \"{length_str}\"").as_str());
            if total_length > parsed {
                panic!("Allocated memory length ({parsed} words) is exceeded by actual data size ({total_length} words)");
            } else {
                parsed
            }
        };

        let start = if start_str == "auto" {
            state.pointer
        } else {
            Self::parse_number(start_str).expect(format!("Expected 'auto' or valid start address, but got \"{start_str}\"").as_str())
        };

        for i in start..(start + length) {
            if state.allocated.contains(&i) {
                panic!("Address {:#x} has already been allocated!", i);
            } else {
                state.allocated.push(i);
            }
        }

        state.pointer += length;
        DataBlock {
            name,
            start,
            length,
            entries
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
                    println!("DATA:\n\n{}\n", serde_json::to_string_pretty(&block).unwrap());
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
