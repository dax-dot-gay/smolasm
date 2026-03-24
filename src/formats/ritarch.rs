use std::{fs::File, io::Write};

use crate::{
    formats::OutputFormatter,
    types::{BitArray, DataBlock, TextBlock},
};

pub struct RitArch;

impl OutputFormatter for RitArch {
    fn assemble(assembly: crate::parser::Assembly, path: std::path::PathBuf) -> () {
        let mut output = File::create(path).unwrap();
        let mut entrypoint_address = 0u64;
        for block in assembly.blocks {
            match block {
                crate::types::AssemblyBlock::Data(DataBlock { start, entries, .. }) => {
                    let mut offset = 0u64;
                    for (_, bits) in entries {
                        let chunks =
                            bits.to_chunks(assembly.config.system.hardware.word_size.clone());
                        let offset_bits = BitArray::from_slice(&(start + offset).to_be_bytes())
                            .truncate(&assembly.config);
                        let length_bits = BitArray::from_slice(&chunks.len().to_be_bytes())
                            .truncate(&assembly.config);
                        output
                            .write_all(
                                format!(
                                    "{} {} {}\n",
                                    offset_bits.to_hex(),
                                    length_bits.to_hex(),
                                    chunks
                                        .into_iter()
                                        .map(|v| v.to_hex())
                                        .collect::<Vec<String>>()
                                        .join(" ")
                                )
                                .as_bytes(),
                            )
                            .unwrap();
                        offset += bits.len_words(&assembly.config);
                    }
                }
                crate::types::AssemblyBlock::Text(TextBlock {
                    name,
                    start,
                    instructions,
                    ..
                }) => {
                    if name == String::from("main") {
                        entrypoint_address = start;
                    }

                    let mut offset = 0u64;
                    for (_, fields) in instructions {
                        let mut joined = BitArray::default();
                        for field in fields {
                            joined.extend(field.raw_value.into_inner());
                        }
                        let chunks =
                            joined.to_chunks(assembly.config.system.hardware.word_size.clone());
                        let offset_bits = BitArray::from_slice(&(start + offset).to_be_bytes())
                            .truncate(&assembly.config);
                        let length_bits = BitArray::from_slice(&chunks.len().to_be_bytes())
                            .truncate(&assembly.config);
                        output
                            .write_all(
                                format!(
                                    "{} {} {}\n",
                                    offset_bits.to_hex(),
                                    length_bits.to_hex(),
                                    chunks
                                        .into_iter()
                                        .map(|v| v.to_hex())
                                        .collect::<Vec<String>>()
                                        .join(" ")
                                )
                                .as_bytes(),
                            )
                            .unwrap();
                        offset += joined.len_words(&assembly.config);
                    }
                }
            }
        }

        output
            .write_all(
                BitArray::from_slice(&entrypoint_address.to_be_bytes())
                    .truncate(&assembly.config)
                    .to_hex()
                    .as_bytes(),
            )
            .unwrap();
        output.flush().unwrap();
    }
}
