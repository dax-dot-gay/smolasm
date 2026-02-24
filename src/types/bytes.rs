use std::{fmt::Display, num::ParseIntError};

use hex::{FromHex, FromHexError, ToHex};
use serde::{Deserialize, Serialize};

const HEX_CHARS: &'static str = "abcdef1234567890";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct ByteString(String);

impl ByteString {
    pub fn into_hex(self) -> String {
        self.0
    }

    pub fn from_hex(value: impl AsRef<str>) -> Result<Self, FromHexError> {
        let value = value.as_ref().to_string().to_lowercase();
        if value.chars().all(|v| HEX_CHARS.contains(v)) {
            Ok(Self(value))
        } else {
            for (index, c) in value.chars().enumerate() {
                if !HEX_CHARS.contains(c) {
                    return Err(FromHexError::InvalidHexCharacter { c, index });
                }
            }
            unreachable!("Ain invalid character must have been detected!");
        }
    }

    pub fn bytes(&self) -> Result<Vec<u8>, FromHexError> {
        Vec::<u8>::from_hex(self.0.clone())
    }

    pub fn from_bytes(value: impl AsRef<[u8]>) -> Self {
        Self(value.as_ref().encode_hex())
    }
}

impl Display for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for ByteString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

macro_rules! num_conversions {
    ($type:ident, $($types:ident),+) => {
        impl TryFrom<ByteString> for $type {
            type Error = ParseIntError;
            fn try_from(value: ByteString) -> Result<Self, Self::Error> {
                $type::from_str_radix(&value.into_hex(), 16)
            }
        }

        impl From<$type> for ByteString {
            fn from(value: $type) -> Self {
                Self::from_bytes(value.to_be_bytes())
            }
        }

        num_conversions!($($types),+);
    };
    ($type:ident) => {
        impl TryFrom<ByteString> for $type {
            type Error = ParseIntError;
            fn try_from(value: ByteString) -> Result<Self, Self::Error> {
                $type::from_str_radix(&value.into_hex(), 16)
            }
        }

        impl From<$type> for ByteString {
            fn from(value: $type) -> Self {
                Self::from_bytes(value.to_be_bytes())
            }
        }
    };
}

num_conversions!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl From<String> for ByteString {
    fn from(value: String) -> Self {
        ByteString::from_hex(value.clone()).unwrap_or(ByteString::from_bytes(value))
    }
}

impl From<ByteString> for String {
    fn from(value: ByteString) -> Self {
        value.into_hex()
    }
}
