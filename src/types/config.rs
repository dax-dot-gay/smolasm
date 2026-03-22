use std::{
    collections::HashMap,
    fs,
    ops::{Deref, DerefMut},
    path::Path
};

use kdl::{KdlDocument, KdlNode, KdlValue};
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

impl VariantConfig {
    pub(self) fn parse_variant(doc: KdlNode) -> Self {
        let variant: u64 = doc
            .get(0)
            .expect("<variant> expects a single argument")
            .as_integer()
            .expect("The <variant> discriminator should be an unsigned integer")
            .try_into()
            .expect("The <variant> discriminator should fit within a u64");
        let name: String = doc.children().expect("<variant> should have children").get("name").expect("<variant> requires exactly one <name> child").get(0).expect("<name> requires a single argument").as_string().expect("<name> should be a string").to_string();
        let mut alias: Vec<String> = Vec::new();
        for child in doc.iter_children() {
            match child.name().to_string().as_str() {
                "name" => (),
                "alias" => alias.push(child.get(0).expect("<alias> requires a single argument").as_string().expect("<alias> should be a string").to_string()),
                other => panic!("Unknown child node of <variant>: {}", other)
            }
        }

        Self {
            variant, name, alias
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FieldType {
    Enum(HashMap<u64, VariantConfig>),
    Raw,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldConfig {
    pub name: String,
    pub bits: u64,
    pub field_type: FieldType,
}

impl FieldConfig {
    pub(self) fn parse_field(doc: KdlNode) -> Self {
        let name = doc
            .get(0)
            .expect("<field> needs a name argument")
            .as_string()
            .unwrap()
            .to_string();
        let children = doc.children().expect("<field> requires children").clone();
        let bits: u64 = children
            .get("bits")
            .expect("<field> requires <bits> child")
            .get(0)
            .expect("<bits> requires a single numerical argument")
            .as_integer()
            .unwrap()
            .try_into()
            .expect("<bits> should fit within a u64");
        let field_type_str = children
            .get("type")
            .expect("<field> requires <type> child")
            .get(0)
            .expect("<type> requires either 'enum' or 'raw'")
            .as_string()
            .unwrap()
            .to_string();
        let field_type = match field_type_str.as_str() {
            "enum" => FieldType::Enum(doc.iter_children().filter_map(|child| if child.name().to_string() == String::from("variant") {
                let parsed = VariantConfig::parse_variant(child.clone());
                Some((parsed.variant, parsed))
            } else {None}).collect()),
            "raw" => FieldType::Raw,
            other => panic!("Unknown field type \"{}\"", other),
        };

        Self {
            name,
            bits,
            field_type,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstructionFieldConfig {
    pub value: String,
    pub index_in: u64,
    pub index_out: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstructionConfig {
    pub name: String,
    pub fields: Vec<InstructionFieldConfig>,
}

impl InstructionConfig {
    pub(self) fn parse_instruction(doc: KdlNode) -> Self {
        let name = doc.get(0).expect("<instruction> requires a single argument").as_string().expect("<instruction> should have a single string argument").to_string();
        let fields: Vec<InstructionFieldConfig> = doc.iter_children().cloned().map(|child| {
            if child.name().to_string().as_str() == "field" {
                InstructionFieldConfig { 
                    value: child.get("value").expect("<field> expects value=\"string\"").as_string().expect("<field>.value should be a string").to_string(), 
                    index_in: child.get("in").expect("<field> expects in=u64").as_integer().expect("<field>.in should be u64").try_into().unwrap(), 
                    index_out: child.get("out").expect("<field> expects out=u64").as_integer().expect("<field>.out should be u64").try_into().unwrap()
                }
            } else {
                panic!("Unknown child of instruction/{}: {}", name, child.name().to_string());
            }
        }).collect();
        Self { name, fields }
    }
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
        let system = SystemConfig::parse(
            parsed
                .get("system")
                .expect("Requires <system> node")
                .clone(),
        );

        let mut fields: HashMap<String, FieldConfig> = HashMap::new();
        let mut instructions: HashMap<String, InstructionConfig> = HashMap::new();

        for child in parsed.into_iter() {
            match child.name().to_string().as_str() {
                "system" => (),
                "field" => {
                    let parsed = FieldConfig::parse_field(child);
                    let _ = fields.insert(parsed.name.clone(), parsed);
                },
                "instruction" => {
                    let parsed = InstructionConfig::parse_instruction(child);
                    let _ = instructions.insert(parsed.name.clone(), parsed);
                },
                other => panic!("Unknown top-level node: \"{other}\"")
            }
        }

        Ok(Self { system, fields, instructions })
    }
}
