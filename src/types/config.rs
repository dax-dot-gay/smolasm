use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::ByteString;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AsmFieldMeta {
    pub name: String,
    pub size: u64,

    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImmediateKind {
    #[default]
    Bytes,
    SignedInt,
    UnsignedInt,
    SignedFloat,
    UnsignedFloat,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnumOption {
    pub discriminator: ByteString,
    pub aliases: Vec<String>,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpCode {
    pub code: ByteString,
    pub aliases: Vec<String>,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AsmField {
    #[serde(alias = "raw", alias = "imm")]
    Immediate {
        meta: AsmFieldMeta,

        #[serde(default)]
        kind: ImmediateKind,
    },
    #[serde(alias = "addr")]
    Address { meta: AsmFieldMeta },
    #[serde(alias = "enum")]
    Enumerator {
        meta: AsmFieldMeta,
        values: HashMap<ByteString, EnumOption>,
    },
    #[serde(alias = "opc", alias = "code", alias = "instruction")]
    Opcode {
        meta: AsmFieldMeta,
        values: HashMap<ByteString, OpCode>,
    },
}

impl AsmField {
    pub fn meta(&self) -> AsmFieldMeta {
        match self {
            AsmField::Immediate { meta, .. } => meta.clone(),
            AsmField::Address { meta } => meta.clone(),
            AsmField::Enumerator { meta, .. } => meta.clone(),
            AsmField::Opcode { meta, .. } => meta.clone(),
        }
    }

    pub fn name(&self) -> String {
        self.meta().name
    }

    pub fn size(&self) -> u64 {
        self.meta().size
    }

    pub fn comment(&self) -> Option<String> {
        self.meta().comment
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AsmConfig {
    pub output_format: crate::formats::OutputFormat,
    pub word_size: u64,

    #[serde(default)]
    pub fields: HashMap<String, AsmField>,
}
