use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
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
        Self(slice.into_iter().flat_map(|byt| format!("{byt:b}").chars().map(|v| v == '1').collect::<Vec<bool>>()).collect())
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
