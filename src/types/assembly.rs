use std::{fmt::Display, ops::{Deref, DerefMut}};

use serde::{Deserialize, Serialize};

use crate::types::Config;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataBlock {
    pub name: String,
    pub start: u64,
    pub length: u64,
    pub entries: Vec<(String, BitArray)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstructionField {
    pub format: String,
    pub input_index: u64,
    pub output_index: u64,
    pub asm_value: String,
    pub raw_value: BitArray,
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(try_from = "String", into = "String")]
pub struct BitArray(Vec<bool>);

impl BitArray {
    pub fn new(data: impl IntoIterator<Item = bool>) -> Self {
        Self(data.into_iter().collect())
    }

    pub fn into_inner(self) -> Vec<bool> {
        self.0
    }

    pub fn from_slice(slice: impl AsRef<[u8]>) -> Self {
        let slice = slice.as_ref().to_vec();
        Self(slice.into_iter().flat_map(|byt| format!("{byt:08b}").chars().map(|v| v == '1').collect::<Vec<bool>>()).collect())
    }

    pub fn trimmed(self, bits: usize) -> Self {
        let inner = self.into_inner();
        let mut trim = inner[(inner.len() - bits)..].to_vec();
        if trim.len() < bits {
            for _ in trim.len()..bits {
                trim.insert(0, false);
            }
        }
        Self(trim)
    }

    pub fn truncate(self, config: &Config) -> Self {
        if u64::try_from(self.len()).unwrap() < config.system.hardware.word_size {
            return self;
        }
        let mut inner = self.into_inner();
        loop {
            let mut empty = true;
            for i in &inner.clone()[..usize::try_from(config.system.hardware.word_size).unwrap()] {
                if *i {
                    empty = false;
                }
            }

            if empty {
                inner = inner[usize::try_from(config.system.hardware.word_size).unwrap()..].to_vec();
            } else {
                break;
            }
        }
        let out = Self(inner);
        out.trimmed(usize::try_from(config.system.hardware.word_size).unwrap())
    }

    pub fn len_aligned(&self, config: &Config) -> u64 {
        u64::try_from(self.0.len()).unwrap().div_ceil(config.system.hardware.word_size) * config.system.hardware.word_size
    }

    pub fn len_words(&self, config: &Config) -> u64 {
        u64::try_from(self.0.len()).unwrap().div_ceil(config.system.hardware.word_size)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut ptr = 0usize;
        let mut output: Vec<u8> = vec![];
        while ptr < self.len() {
            let mut value = 0u8;
            for bit in 0..8u8 {
                if self.0.get(ptr).is_some_and(|v| *v) {
                    value |= 1 << (7 - bit);
                }
                ptr += 1;
            }
            output.push(value);
        }

        output
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }
}

impl Display for BitArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(String::from(self.clone()).as_str())
    }
}

impl Deref for BitArray {
    type Target = Vec<bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BitArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<String> for BitArray {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut result: Vec<bool> = Vec::new();
        for ch in value.chars() {
            match ch {
                '0' => result.push(false),
                '1' => result.push(true),
                other => return Err(format!("Invalid binary character {other}"))
            }
        }

        Ok(Self(result))
    }
}

impl From<BitArray> for String {
    fn from(value: BitArray) -> Self {
        value.into_inner().into_iter().map(|v| if v {'1'} else {'0'}).collect()
    }
}
