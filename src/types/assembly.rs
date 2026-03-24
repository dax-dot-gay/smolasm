use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataBlock {
    pub name: String,
    pub start: u64,
    pub length: u64,
    pub entries: Vec<(String, BitVec<u8, Msb0>)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstructionField {
    pub format: String,
    pub input_index: u64,
    pub output_index: u64,
    pub asm_value: String,
    pub raw_value: BitVec<u8, Msb0>,
    pub bits: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextBlock {
    pub name: String,
    pub start: u64,
    pub length: u64,
    pub instructions: Vec<(String, Vec<InstructionField>)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AssemblyBlock {
    Data(DataBlock),
    Text(TextBlock),
}
