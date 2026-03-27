use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    path::{Path, PathBuf},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{
    ParsingError, SmolError,
    types::{AssemblyBlock, BitArray, Config, DataBlock, InstructionField, TextBlock},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assembly {
    pub path: PathBuf,
    pub config: Config,
    pub blocks: Vec<AssemblyBlock>,
    pub block_map: HashMap<String, usize>,
    pub label_map: HashMap<String, u64>,
}

#[derive(Debug)]
struct State {
    pub pointer: u64,
    pub allocated: Vec<u64>,
    pub labels: HashMap<String, u64>,
    pub line: u64,
}

impl State {
    pub fn report_error(&self, error: impl Into<crate::ParsingError>) -> SmolError {
        SmolError::AssemblyError {
            line: self.line.clone(),
            err: error.into(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            pointer: 0,
            allocated: Vec::new(),
            labels: HashMap::new(),
            line: 0,
        }
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

    fn get_bits(value: impl AsRef<str>, trim: usize, state: &State) -> crate::Result<BitArray> {
        let value = value.as_ref().trim().to_string();
        if value.starts_with("0x") && value.len() > 2 {
            Ok(BitArray::from_slice(
                hex::decode(value.split_once('x').unwrap().1)
                    .or_else(|e| Err(state.report_error(e)))?
                    .as_slice(),
            )
            .trimmed(trim))
        } else if value.starts_with("0b") && value.len() > 2 {
            Ok(
                BitArray::new(value.split_once('b').unwrap().1.chars().map(|c| c == '1'))
                    .trimmed(trim),
            )
        } else {
            let parsed = Self::parse_number(value.clone()).map_err(|e| state.report_error(e))?;
            Ok(BitArray::from_slice(&parsed.to_be_bytes()).trimmed(trim))
        }
    }

    fn parse_text_block(
        line: String,
        input: &mut BufReader<File>,
        config: &Config,
        state: &mut State,
    ) -> crate::Result<TextBlock> {
        let header_parts: Vec<&str> = line.split(" ").collect();
        let name = header_parts[1].to_string();
        let (start_str, length_str) = header_parts[2].split_once("..").ok_or(state.report_error(ParsingError::syntax("Block range specifier should follow the format \"<start|auto>..<length|auto>\", but was missing the \"..\" delimiter.")))?;
        let mut total_length = 0u64;
        let mut instructions: Vec<(String, Vec<InstructionField>)> = vec![];

        loop {
            let mut line = String::new();
            if let Ok(length) = input.read_line(&mut line) {
                state.line += 1;
                if length == 0 {
                    break;
                }
                let line = line.trim().to_string();
                if line.starts_with(".data") || line.starts_with(".text") {
                    input
                        .seek_relative(i64::try_from(length).unwrap() * -1)
                        .unwrap();
                    state.line -= 1;
                    break;
                }

                if line.len() == 0 || line.starts_with("//") {
                    continue;
                }

                let mut processing_formats: HashMap<String, Vec<InstructionField>> = config
                    .instructions
                    .clone()
                    .keys()
                    .map(|k| (k.clone(), vec![]))
                    .collect();
                for (field_index, field) in line.split(" ").map(|v| v.to_string()).enumerate() {
                    let mut to_remove: Vec<String> = Vec::new();
                    for format_name in processing_formats.clone().keys().cloned() {
                        let inst = config.instructions.get(&format_name).unwrap().clone();
                        if let Some(target_format) = inst
                            .fields
                            .clone()
                            .into_iter()
                            .find(|v| v.index_in == u64::try_from(field_index).unwrap())
                        {
                            let (field_id, constraint) = if target_format.value.contains("=") {
                                let (id, cons) = target_format.value.split_once('=').unwrap();
                                (
                                    id.to_string(),
                                    cons.split('|')
                                        .map(|v| v.to_string())
                                        .collect::<Vec<String>>(),
                                )
                            } else {
                                (target_format.value.clone(), Vec::new())
                            };

                            let selected_field =
                                config
                                    .fields
                                    .get(&field_id)
                                    .cloned()
                                    .ok_or(state.report_error(ParsingError::syntax(format!(
                                        "Unknown field type referenced in config: {field_id}"
                                    ))))?;
                            match selected_field.field_type.clone() {
                                crate::types::FieldType::Enum(variants) => {
                                    if let Some((discriminator, variant)) =
                                        variants.clone().into_iter().find_map(|(k, v)| {
                                            if v.name.clone() == field || v.alias.contains(&field) {
                                                Some((k.clone(), v.clone()))
                                            } else {
                                                None
                                            }
                                        })
                                    {
                                        if constraint.is_empty()
                                            || constraint.contains(&variant.name)
                                        {
                                            let ivec =
                                                processing_formats.get_mut(&format_name).unwrap();
                                            ivec.push(InstructionField {
                                                format: target_format.value.clone(),
                                                input_index: target_format.index_in,
                                                output_index: target_format.index_out,
                                                asm_value: field.clone(),
                                                raw_value: Some(
                                                    BitArray::from_slice(
                                                        &discriminator.to_be_bytes(),
                                                    )
                                                    .trimmed(
                                                        usize::try_from(selected_field.bits)
                                                            .unwrap(),
                                                    ),
                                                ),
                                                reference_value: None,
                                                bits: selected_field.bits,
                                            });
                                        } else {
                                            to_remove.push(format_name);
                                        }
                                    } else {
                                        to_remove.push(format_name);
                                    }
                                }
                                crate::types::FieldType::Raw => {
                                    if !constraint.is_empty() {
                                        return Err(state.report_error(ParsingError::syntax(format!("Config attempted to specify a constraint on a raw field: {field_id}"))));
                                    }

                                    let ivec = processing_formats.get_mut(&format_name).unwrap();

                                    if field.starts_with("@") {
                                        let trimmed_label = field.trim_matches('@').to_string();
                                        let (label, offset) = if trimmed_label.contains("+") {
                                            let (_label, _offset) =
                                                trimmed_label.split_once("+").unwrap();
                                            (
                                                _label.to_string(),
                                                _offset
                                                    .parse::<i64>()
                                                    .or_else(|e| Err(state.report_error(e)))?
                                                    .abs(),
                                            )
                                        } else if trimmed_label.contains("-") {
                                            let (_label, _offset) =
                                                trimmed_label.split_once("-").unwrap();
                                            (
                                                _label.to_string(),
                                                _offset
                                                    .parse::<i64>()
                                                    .or_else(|e| Err(state.report_error(e)))?
                                                    .abs()
                                                    * -1,
                                            )
                                        } else {
                                            (trimmed_label.to_string(), 0)
                                        };
                                        ivec.push(InstructionField {
                                            format: target_format.value.clone(),
                                            input_index: target_format.index_in,
                                            output_index: target_format.index_out,
                                            asm_value: field.clone(),
                                            raw_value: None,
                                            reference_value: Some((label, offset)),
                                            bits: selected_field.bits,
                                        });
                                    } else {
                                        ivec.push(InstructionField {
                                            format: target_format.value.clone(),
                                            input_index: target_format.index_in,
                                            output_index: target_format.index_out,
                                            asm_value: field.clone(),
                                            raw_value: Some(Self::get_bits(
                                                field.clone(),
                                                usize::try_from(selected_field.bits).unwrap(),
                                                &state,
                                            )?),
                                            reference_value: None,
                                            bits: selected_field.bits,
                                        });
                                    }
                                }
                            }
                        } else {
                            to_remove.push(format_name);
                        }
                    }

                    for i in to_remove {
                        let _ = processing_formats.remove(&i);
                    }
                }

                let mut to_remove: Vec<String> = vec![];
                for format_name in processing_formats.clone().into_keys() {
                    let inst = config.instructions.get(&format_name).cloned().unwrap();
                    let ivec = processing_formats.get_mut(&format_name).unwrap();
                    if inst.fields.len() > ivec.len() {
                        let mut sorted = inst.fields.clone();
                        sorted.sort_by_key(|v| v.index_in);
                        for field in sorted {
                            if field.index_in >= u64::try_from(ivec.len()).unwrap() {
                                if let Some(default) = field.default {
                                    let (field_id, _) = if field.value.contains("=") {
                                        let (id, cons) = field.value.split_once('=').unwrap();
                                        (
                                            id.to_string(),
                                            cons.split('|')
                                                .map(|v| v.to_string())
                                                .collect::<Vec<String>>(),
                                        )
                                    } else {
                                        (field.value.clone(), Vec::new())
                                    };

                                    let selected_field = config
                                        .fields
                                        .get(&field_id)
                                        .cloned()
                                        .ok_or(
                                        state.report_error(ParsingError::syntax(format!(
                                            "Unknown field type referenced in config: {field_id}"
                                        ))),
                                    )?;
                                    ivec.push(InstructionField {
                                        format: field.value.clone(),
                                        input_index: field.index_in,
                                        output_index: field.index_out,
                                        asm_value: format!("{default}"),
                                        raw_value: Some(Self::get_bits(
                                            format!("{default}"),
                                            usize::try_from(selected_field.bits).unwrap(),
                                            &state,
                                        )?),
                                        reference_value: None,
                                        bits: selected_field.bits,
                                    })
                                } else {
                                    to_remove.push(format_name.clone());
                                }
                            }
                        }
                    }
                }

                for i in to_remove {
                    let _ = processing_formats.remove(&i);
                }

                if processing_formats.len() == 0 {
                    return Err(state.report_error(ParsingError::unknown(
                        line.clone(),
                        "This instruction did not resolve to any configured instruction formats.",
                    )));
                } else if processing_formats.len() > 1 {
                    return Err(state.report_error(ParsingError::unknown(line.clone(), "This instruction resolved to more than one configured instruction format, and is thus ambiguous.")));
                } else {
                    let format = processing_formats
                        .into_values()
                        .collect::<Vec<_>>()
                        .first()
                        .cloned()
                        .unwrap();
                    instructions.push((line.clone(), format));
                }
            } else {
                break;
            }
        }

        for (_, bits) in instructions.clone() {
            let mut instbits = 0u64;
            for field in bits {
                instbits += field.bits;
            }
            total_length += instbits.div_ceil(config.system.hardware.word_size);
        }

        let length = if length_str == "auto" {
            total_length
        } else {
            let parsed = Self::parse_number(length_str).or(Err(state.report_error(
                ParsingError::syntax(format!(
                    "Expected valid length string or 'auto', but got \"{length_str}\" instead"
                )),
            )))?;
            if total_length > parsed {
                return Err(state.report_error(ParsingError::syntax(format!("Allocated memory length ({parsed} words) is exceeded by actual data size ({total_length} words)"))));
            } else {
                parsed
            }
        };

        let start = if start_str == "auto" {
            state.pointer
        } else {
            Self::parse_number(start_str).or(Err(state.report_error(ParsingError::syntax(
                format!("Expected valid start string or 'auto', but got \"{start_str}\" instead"),
            ))))?
        };

        for i in start..(start + length) {
            if state.allocated.contains(&i) {
                return Err(state.report_error(ParsingError::Allocation(i)));
            } else {
                state.allocated.push(i);
            }
        }

        state.pointer += length;
        let _ = state.labels.insert(name.clone(), start);

        Ok(TextBlock {
            name,
            start,
            length,
            instructions,
        })
    }

    fn parse_data_block(
        line: String,
        input: &mut BufReader<File>,
        config: &Config,
        state: &mut State,
    ) -> crate::Result<DataBlock> {
        let header_parts: Vec<&str> = line.split(" ").collect();
        let name = header_parts[1].to_string();
        let (start_str, length_str) = header_parts[2].split_once("..").ok_or(state.report_error(ParsingError::syntax("Block range specifier should follow the format \"<start|auto>..<length|auto>\", but was missing the \"..\" delimiter.")))?;
        let mut total_length = 0u64;

        let mut entries: Vec<(String, BitArray)> = Vec::new();
        loop {
            let mut line = String::new();
            if let Ok(length) = input.read_line(&mut line) {
                state.line += 1;
                if length == 0 {
                    break;
                }
                let line = line.trim_start();
                if line.starts_with(".data") || line.starts_with(".text") {
                    input
                        .seek_relative(i64::try_from(length).unwrap() * -1)
                        .unwrap();
                    state.line -= 1;
                    break;
                }

                if line.starts_with("\"\"\"") {
                    let mut full_str = line.to_string();
                    while !(full_str.ends_with("\"\"\"\n") && full_str.len() > 4) {
                        let mut rline = String::new();
                        input.read_line(&mut rline).or(Err(state.report_error(ParsingError::syntax("Unexpected EOF in multiline string"))))?;
                        full_str += rline.trim_start();
                    }

                    let full_str = full_str.trim().trim_matches('"').to_string() + "\0";
                    entries.push((full_str.clone(), BitArray::from_slice(full_str.as_bytes())));
                } else if line.starts_with('"') {
                    if line.ends_with("\"\n") && line.len() > 2 {
                        let trimmed = line.trim().trim_matches('"').to_string() + "\0";
                        entries.push((trimmed.clone(), BitArray::from_slice(trimmed.as_bytes())));
                    } else {
                        return Err(state.report_error(ParsingError::syntax("Single line strings must terminate with double quotes")));
                    }
                } else if line.starts_with("0x") && line.len() > 3 {
                    let trimmed = line.trim().split_once('x').unwrap().1.to_string();
                    let raw = hex::decode(trimmed).or_else(|e| Err(state.report_error(e)))?;
                    entries.push((line.trim().to_string(), BitArray::from_slice(&raw)));
                } else if line.starts_with("0b") && line.len() > 3 {
                    let trimmed = line.trim().split_once('b').unwrap().1.to_string();
                    let bits = BitArray::new(trimmed.chars().map(|c| c == '1'));

                    entries.push((line.trim().to_string(), bits));
                } else if let Ok(number) = line.trim().parse::<u64>() {
                    entries.push((
                        line.trim().to_string(),
                        BitArray::from_slice(&number.to_be_bytes()).truncate(&config),
                    ));
                } else if line.trim().len() == 0 || line.trim().starts_with("//") {
                    continue;
                } else {
                    return Err(state.report_error(ParsingError::syntax(format!("Invalid assembly line: {}", line.trim()))));
                }
            } else {
                break;
            }
        }

        for (_, bits) in entries.clone() {
            total_length += bits.len_words(&config);
        }

        let length = if length_str == "auto" {
            total_length
        } else {
            let parsed = Self::parse_number(length_str).or(Err(state.report_error(
                ParsingError::syntax(format!(
                    "Expected valid length string or 'auto', but got \"{length_str}\" instead"
                )),
            )))?;
            if total_length > parsed {
                return Err(state.report_error(ParsingError::syntax(format!("Allocated memory length ({parsed} words) is exceeded by actual data size ({total_length} words)"))));
            } else {
                parsed
            }
        };

        let start = if start_str == "auto" {
            state.pointer
        } else {
            Self::parse_number(start_str).or(Err(state.report_error(ParsingError::syntax(
                format!("Expected valid start string or 'auto', but got \"{start_str}\" instead"),
            ))))?
        };

        for i in start..(start + length) {
            if state.allocated.contains(&i) {
                return Err(state.report_error(ParsingError::Allocation(i)));
            } else {
                state.allocated.push(i);
            }
        }

        state.pointer += length;
        let _ = state.labels.insert(name.clone(), start);

        Ok(DataBlock {
            name,
            start,
            length,
            entries,
        })
    }

    pub fn parse(input_file: impl AsRef<Path>, config: Config) -> crate::Result<Self> {
        let path = input_file.as_ref().to_path_buf();
        let mut input = BufReader::new(
            File::open(path.clone())
                .or(Err(SmolError::NotFound(path.to_string_lossy().to_string())))?,
        );
        let mut blocks: Vec<AssemblyBlock> = vec![];
        let mut block_map: HashMap<String, usize> = HashMap::new();
        let mut state = State::default();

        loop {
            let mut line = String::new();
            if let Ok(length) = input.read_line(&mut line) {
                state.line += 1;
                if length == 0 {
                    break;
                }
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.len() == 0 {
                    continue;
                }

                if trimmed.starts_with(".data") {
                    let block = Self::parse_data_block(
                        trimmed.to_string(),
                        &mut input,
                        &config,
                        &mut state,
                    )?;
                    blocks.push(AssemblyBlock::Data(block.clone()));
                    block_map.insert(block.name.clone(), blocks.len() - 1);
                } else if trimmed.starts_with(".text") {
                    let block = Self::parse_text_block(
                        trimmed.to_string(),
                        &mut input,
                        &config,
                        &mut state,
                    )?;
                    blocks.push(AssemblyBlock::Text(block.clone()));
                    block_map.insert(block.name.clone(), blocks.len() - 1);
                } else {
                    return Err(state.report_error(ParsingError::syntax(format!("Unknown top-level line {trimmed}"))));
                }
            } else {
                break;
            }
        }

        Ok(Self {
            path,
            config,
            blocks,
            block_map,
            label_map: state.labels,
        })
    }

    pub fn analyze(&self, config_path: String) {
        println!(
            "{} {}",
            "Currently Analyzing:".bold().bright_white(),
            self.path.clone().to_string_lossy().to_string()
        );
        println!(
            "{} {}",
            "Config:".bold().bright_white(),
            config_path.clone()
        );
        println!("{}", "Blocks:".bold().bright_white());
        for block in self.blocks.clone() {
            match block {
                AssemblyBlock::Data(DataBlock {
                    name,
                    start,
                    length,
                    entries,
                }) => {
                    println!(
                        "        {}{} @ {:#x}..{:#x}: {} words {} {{",
                        "DATA/".bold(),
                        name,
                        start,
                        start + (length - 1),
                        length,
                        "(inclusive)".dimmed()
                    );
                    if entries.len() > 0 {
                        let mut offset = 0u64;
                        for (entry_text, entry) in entries {
                            println!(
                                "                {:#0x}: {} - {} words {}",
                                start + offset,
                                entry_text.escape_default(),
                                entry.len_words(&self.config),
                                format!("({})", entry.to_hex()).dimmed()
                            );
                            offset += entry.len_words(&self.config);
                        }
                    } else {
                        println!("                {}", "<empty>".dimmed());
                    }
                    println!("        }}");
                }
                AssemblyBlock::Text(TextBlock {
                    name,
                    start,
                    length,
                    instructions,
                }) => {
                    if name == String::from("main") {
                        println!(
                            "{} {}{} @ {:#x}..{:#x}: {} words {} {{",
                            "(entry)".dimmed(),
                            "TEXT/".bold(),
                            name,
                            start,
                            start + (length - 1),
                            length,
                            "(inclusive)".dimmed()
                        );
                    } else {
                        println!(
                            "        {}{} @ {:#x}..{:#x}: {} words {} {{",
                            "TEXT/".bold(),
                            name,
                            start,
                            start + (length - 1),
                            length,
                            "(inclusive)".dimmed()
                        );
                    }
                    if instructions.len() > 0 {
                        let mut offset = 0u64;
                        for (inst_text, fields) in instructions {
                            let mut joined = BitArray::default();
                            for field in fields {
                                joined.extend(field.resolve_value(self.clone()).into_inner());
                            }
                            println!(
                                "                {:#0x}: {} - {} words {}",
                                start + offset,
                                inst_text,
                                joined.len_words(&self.config),
                                format!("({})", joined.to_hex()).dimmed()
                            );
                            offset += joined.len_words(&self.config);
                        }
                    } else {
                        println!("                {}", "<empty>".dimmed());
                    }
                    println!("        }}");
                }
            }
        }
    }
}
