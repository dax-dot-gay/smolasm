use std::{collections::HashMap, fs, path::Path};

use kdl::{KdlDocument, KdlNode};
use serde::{Deserialize, Serialize};

use crate::{ConfigError, SmolError};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemHardware {
    pub address_size: u64,
    pub word_size: u64,
}

impl SystemHardware {
    fn arg_u64(doc: &KdlDocument, key: impl Into<String>) -> crate::Result<u64> {
        let key = key.into();
        u64::try_from(
            doc.get(&key)
                .ok_or(ConfigError::invalid_content(format!(
                    "system.hardware is missing a required field: {key}"
                )))?
                .get(0)
                .ok_or(ConfigError::invalid_content(format!(
                    "system.hardware.{key} requires a single argument."
                )))?
                .as_integer()
                .ok_or(ConfigError::invalid_content(format!(
                    "system.hardware.{key} should be an unsigned integer."
                )))?,
        )
        .or(Err(ConfigError::invalid_content(format!(
            "system.hardware.{key} should be an unsigned integer."
        ))
        .into()))
    }
    pub(self) fn parse_hardware(doc: KdlNode) -> crate::Result<Self> {
        let children = doc
            .children()
            .ok_or(ConfigError::invalid_content(
                "system.hardware must have child nodes!",
            ))?
            .clone();
        Ok(Self {
            address_size: Self::arg_u64(&children, "address_size")?,
            word_size: Self::arg_u64(&children, "word_size")?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemConfig {
    pub name: String,
    pub format: String,
    pub hardware: SystemHardware,
}

impl SystemConfig {
    pub(self) fn parse(doc: KdlNode) -> crate::Result<Self> {
        let name = doc
            .children()
            .ok_or(ConfigError::invalid_content(
                "system node requires child nodes",
            ))?
            .get("name")
            .ok_or(ConfigError::invalid_content(
                "Missing expected node system.name",
            ))?
            .get(0)
            .ok_or(ConfigError::invalid_content(
                "system.name requires a single positional argument",
            ))?
            .as_string()
            .ok_or(ConfigError::invalid_content(
                "system.name should be a string",
            ))?
            .to_string();
        let format = doc
            .children()
            .unwrap()
            .get("format")
            .ok_or(ConfigError::invalid_content(
                "Missing expected node system.format",
            ))?
            .get(0)
            .ok_or(ConfigError::invalid_content(
                "Missing system.format requires a single positional argument",
            ))?
            .as_string()
            .ok_or(ConfigError::invalid_content(
                "system.format should be a string",
            ))?
            .to_string();

        let hardware = SystemHardware::parse_hardware(
            doc.children()
                .unwrap()
                .get("hardware")
                .ok_or(ConfigError::invalid_content(
                    "Missing expected node system.hardware",
                ))?
                .clone(),
        )?;

        Ok(Self {
            name,
            format,
            hardware,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VariantConfig {
    pub variant: u64,
    pub name: String,
    pub alias: Vec<String>,
}

impl VariantConfig {
    pub(self) fn parse_variant(doc: KdlNode) -> crate::Result<Self> {
        let variant: u64 = doc
            .get(0)
            .ok_or(ConfigError::invalid_content(
                "field.variant requires a single discriminator argument",
            ))?
            .as_integer()
            .ok_or(ConfigError::invalid_content(
                "The discriminator argument of field.variant should be an unsigned integer (u64)",
            ))?
            .try_into()
            .or(Err(ConfigError::invalid_content(
                "The discriminator of field.variant should be an unsigned integer (u64)",
            )))?;
        let name: String = doc
            .children()
            .ok_or(ConfigError::invalid_content(
                "field.variant requires child nodes",
            ))?
            .get("name")
            .ok_or(ConfigError::invalid_content(
                "Missing expected child node field.variant.name",
            ))?
            .get(0)
            .ok_or(ConfigError::invalid_content(
                "field.variant.name requires a single argument",
            ))?
            .as_string()
            .ok_or(ConfigError::invalid_content(
                "field.variant.name should be a string",
            ))?
            .to_string();
        let mut alias: Vec<String> = Vec::new();
        for child in doc.iter_children() {
            match child.name().to_string().as_str() {
                "name" => (),
                "alias" => alias.push(
                    child
                        .get(0)
                        .ok_or(ConfigError::invalid_content(
                            "field.variant.alias requires a single argument",
                        ))?
                        .as_string()
                        .ok_or(ConfigError::invalid_content(
                            "field.variant.alias should be a string",
                        ))?
                        .to_string(),
                ),
                other => {
                    return Err(ConfigError::invalid_content(format!(
                        "Unknown child node \"{other}\" of field.variant"
                    ))
                    .into());
                }
            }
        }

        Ok(Self {
            variant,
            name,
            alias,
        })
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
    pub(self) fn parse_field(doc: KdlNode) -> crate::Result<Self> {
        let name = doc
            .get(0)
            .ok_or(ConfigError::invalid_content(
                "field nodes requires a single positional argument",
            ))?
            .as_string()
            .ok_or(ConfigError::invalid_content(
                "field name argument should be a string",
            ))?
            .to_string();
        let children = doc.children().expect("<field> requires children").clone();
        let bits: u64 = children
            .get("bits")
            .ok_or(ConfigError::invalid_content(
                "Missing expected node field.bits",
            ))?
            .get(0)
            .ok_or(ConfigError::invalid_content(
                "field.bits requires a single positional argument",
            ))?
            .as_integer()
            .ok_or(ConfigError::invalid_content(
                "field.bits should be an unsigned integer (u64)",
            ))?
            .try_into()
            .or(Err(ConfigError::invalid_content(
                "field.bits should be an unsigned integer (u64)",
            )))?;
        let field_type_str = children
            .get("type")
            .ok_or(ConfigError::invalid_content(
                "Missing expected node field.type",
            ))?
            .get(0)
            .ok_or(ConfigError::invalid_content(
                "field.type requires a single positional argument",
            ))?
            .as_string()
            .ok_or(ConfigError::invalid_content(
                "field.type should be a string ('enum' or 'raw')",
            ))?
            .to_string();
        let field_type = match field_type_str.as_str() {
            "enum" => FieldType::Enum(
                doc.iter_children()
                    .filter_map(|child| {
                        if child.name().to_string() == String::from("variant") {
                            match VariantConfig::parse_variant(child.clone()) {
                                Ok(parsed) => Some(Ok((parsed.variant, parsed))),
                                Err(e) => Some(Err(e)),
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Result<HashMap<_, _>, _>>()?,
            ),
            "raw" => FieldType::Raw,
            other => {
                return Err(ConfigError::invalid_content(format!(
                    "Unknown field type in field.type: {other}"
                ))
                .into());
            }
        };

        Ok(Self {
            name,
            bits,
            field_type,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstructionFieldConfig {
    pub value: String,
    pub index_in: u64,
    pub index_out: u64,
    pub default: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstructionConfig {
    pub name: String,
    pub fields: Vec<InstructionFieldConfig>,
}

impl InstructionConfig {
    pub(self) fn parse_instruction(doc: KdlNode) -> crate::Result<Self> {
        let name = doc
            .get(0)
            .ok_or(ConfigError::invalid_content(
                "instruction nodes require a single positional argument",
            ))?
            .as_string()
            .ok_or(ConfigError::invalid_content(
                "instruction[0] should be a string",
            ))?
            .to_string();
        let fields: Vec<InstructionFieldConfig> = doc
            .iter_children()
            .cloned()
            .map(|child| {
                if child.name().to_string().as_str() == "field" {
                    Ok(InstructionFieldConfig {
                        value: child
                            .get("value")
                            .ok_or(ConfigError::invalid_content("instruction.field missing expected parameter \"value\""))?
                            .as_string()
                            .ok_or(ConfigError::invalid_content("instruction.field['value'] should be a string"))?
                            .to_string(),
                        index_in: child
                            .get("in")
                            .ok_or(ConfigError::invalid_content("instruction.field missing expected parameter \"in\""))?
                            .as_integer()
                            .ok_or(ConfigError::invalid_content("instruction.field['in'] should be a u64"))?
                            .try_into()
                            .or(Err(SmolError::from(ConfigError::invalid_content("instruction.field['in'] should be a u64"))))?,
                        index_out: child
                            .get("out")
                            .ok_or(ConfigError::invalid_content("instruction.field missing expected parameter \"out\""))?
                            .as_integer()
                            .ok_or(ConfigError::invalid_content("instruction.field['out'] should be a u64"))?
                            .try_into()
                            .unwrap(),
                        default: if let Some(res) = child
                            .get("default")
                            .and_then(|v| Some(
                                v.as_integer()
                                .ok_or(
                                    SmolError::from(ConfigError::invalid_content("instruction.field['default'] should be an unsigned integer"))).and_then(|v| u64::try_from(v).or(Err(SmolError::from(ConfigError::invalid_content("instruction.field['default'] should be an unsigned integer"))))))) {
                                Some(res?)
                            } else {None}
                    })
                } else {
                    Err(ConfigError::invalid_content(format!("Unknown child node '{}' of instruction", child.name().to_string())).into())
                }
            })
            .collect::<crate::Result<Vec<InstructionFieldConfig>>>()?;
        Ok(Self { name, fields })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub system: SystemConfig,
    pub fields: HashMap<String, FieldConfig>,
    pub instructions: HashMap<String, InstructionConfig>,
}

impl Config {
    pub fn load_config(path: impl AsRef<Path>) -> crate::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let content =
            fs::read_to_string(path.clone()).or(Err(ConfigError::not_found(path.clone())))?;
        let parsed: KdlDocument = content.parse()?;
        let system = SystemConfig::parse(
            parsed
                .get("system")
                .ok_or(ConfigError::invalid_content(
                    "Missing <system> node at top-level",
                ))
                .cloned()?,
        )?;

        let mut fields: HashMap<String, FieldConfig> = HashMap::new();
        let mut instructions: HashMap<String, InstructionConfig> = HashMap::new();

        for child in parsed.into_iter() {
            match child.name().to_string().as_str() {
                "system" => (),
                "field" => {
                    let parsed = FieldConfig::parse_field(child)?;
                    let _ = fields.insert(parsed.name.clone(), parsed);
                }
                "instruction" => {
                    let parsed = InstructionConfig::parse_instruction(child)?;
                    let _ = instructions.insert(parsed.name.clone(), parsed);
                }
                other => {
                    return Err(ConfigError::invalid_content(format!(
                        "Unknown top-level node: <{other}>"
                    )))?;
                }
            }
        }

        Ok(Self {
            system,
            fields,
            instructions,
        })
    }
}
