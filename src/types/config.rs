use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemConfig {
    pub name: String,
    pub format: String,
    pub format_args: HashMap<String, Value>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VariantConfig {
    pub variant: u64,
    pub name: String,
    pub alias: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FieldType {
    Enum(HashMap<u64, VariantConfig>),
    Raw
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldConfig {
    pub name: String,
    pub field_type: FieldType
}