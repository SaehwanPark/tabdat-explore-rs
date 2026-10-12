//! Lossless internal domain exchange. This is not the Python-compatible CLI v1 envelope.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

pub const DOMAIN_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainEnvelope<T> {
  #[serde(deserialize_with = "schema_version")]
  schema_version: u32,
  pub data: T,
}

impl<T> DomainEnvelope<T> {
  pub fn new(data: T) -> Self {
    Self {
      schema_version: DOMAIN_SCHEMA_VERSION,
      data,
    }
  }
  pub fn schema_version(&self) -> u32 {
    self.schema_version
  }
}

fn schema_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
  let value = u32::deserialize(deserializer)?;
  if value == DOMAIN_SCHEMA_VERSION {
    Ok(value)
  } else {
    Err(D::Error::custom("unsupported domain schema version"))
  }
}

pub(crate) mod signed_decimal {
  use super::*;
  pub fn serialize<S: Serializer>(value: &i128, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
  }
  pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i128, D::Error> {
    String::deserialize(deserializer)?
      .parse()
      .map_err(D::Error::custom)
  }
}

pub(crate) mod unsigned_decimal {
  use super::*;
  pub fn serialize<S: Serializer>(value: &u128, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
  }
  pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u128, D::Error> {
    String::deserialize(deserializer)?
      .parse()
      .map_err(D::Error::custom)
  }
}

pub(crate) mod float_text {
  use super::*;
  // Text keeps +/- infinity, NaN and negative zero distinguishable from SQL NULL.
  pub fn serialize<S: Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
  }
  pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    String::deserialize(deserializer)?
      .parse()
      .map_err(D::Error::custom)
  }
}
