use std::{
    collections::HashMap,
    fs,
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
};

use kdl::{KdlDocument, KdlError, KdlNode, KdlValue};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(remote = "KdlValue")]
pub enum KdlValueDef {
    String(String),
    Integer(i128),
    Float(f64),
    Bool(bool),
    Null,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct KdlValueWrapper(#[serde(with = "KdlValueDef")] KdlValue);

impl From<KdlValue> for KdlValueWrapper {
    fn from(value: KdlValue) -> Self {
        Self(value)
    }
}

impl Deref for KdlValueWrapper {
    type Target = KdlValue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KdlValueWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemConfig {
    pub name: String,
    pub format: String,
    pub format_args: HashMap<String, KdlValueWrapper>,
}

impl SystemConfig {
    pub(self) fn parse(doc: KdlNode) -> Self {
        let name = doc
            .children()
            .expect("<system> requires children")
            .get("name")
            .expect("<system> requires a <name> entry")
            .get(0)
            .expect("<name> requires a single argument")
            .as_string()
            .expect("<name> should be a string")
            .to_string();
        let format = doc
            .children()
            .unwrap()
            .get("format")
            .expect("<system> requires a <format> entry")
            .get(0)
            .expect("<format> requires a name argument")
            .as_string()
            .expect("<format> name should be a string")
            .to_string();

        let mut format_args: HashMap<String, KdlValueWrapper> = HashMap::new();
        for child in doc
            .children()
            .unwrap()
            .get("format")
            .unwrap()
            .iter_children()
        {
            format_args.insert(
                child.name().to_string(),
                child
                    .get(0)
                    .expect("Expects a single argument")
                    .clone()
                    .into(),
            );
        }

        Self {
            name,
            format,
            format_args,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VariantConfig {
    pub variant: u64,
    pub name: String,
    pub alias: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FieldType {
    Enum(HashMap<u64, VariantConfig>),
    Raw,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldConfig {
    pub name: String,
    pub bits: i128,
    pub field_type: FieldType,
}

impl FieldConfig {
    pub(self) fn parse_field(doc: KdlNode) -> Self {
        let name = doc.get(0).expect("<field> needs a name argument").as_string().unwrap().to_string();
        let children = doc.children().expect("<field> requires children").clone();
        let bits = children.get("bits").expect("<field> requires <bits> child").get(0).expect("<bits> requires a single numerical argument").as_integer().unwrap();
        let field_type_str = children.get("type").expect("<field> requires <type> child").get(0).expect("<type> requires either 'enum' or 'raw'").as_string().unwrap().to_string();
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstructionFieldConfig {
    pub value: String,
    pub index_in: usize,
    pub index_out: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstructionConfig {
    pub name: String,
    pub fields: Vec<InstructionFieldConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub system: SystemConfig,
    pub fields: HashMap<String, FieldConfig>,
    pub instructions: HashMap<String, InstructionConfig>,
}

impl Config {
    pub fn load_config(path: impl AsRef<Path>) -> Result<Self, kdl::KdlError> {
        let content = fs::read_to_string(path).expect("Failed to open file at specified path.");
        let parsed: KdlDocument = content.parse()?;
        let system_config = SystemConfig::parse(
            parsed
                .get("system")
                .expect("Requires <system> node")
                .clone(),
        );

        Err(KdlError {
            input: Arc::new(String::new()),
            diagnostics: Vec::new(),
        })
    }
}
