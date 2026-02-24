use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    ByteStream,
    ArchLib,
}
