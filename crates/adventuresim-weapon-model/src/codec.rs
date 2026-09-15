use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    GENERATOR_VERSION, HOLDER_GENERATOR_VERSION, HOLDER_SCHEMA_VERSION, SCHEMA_VERSION,
    ValidationError, WeaponDesign, WeaponHolderDesign, validate, validate_holder,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema_version: u16,
    generator_version: u16,
    design: WeaponDesign,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HolderEnvelope {
    schema_version: u16,
    generator_version: u16,
    design: WeaponHolderDesign,
}

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("weapon recipe transport exceeds the size limit")]
    TransportSize,
    #[error("weapon design transport is malformed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported weapon schema version {found}; expected {expected}")]
    SchemaVersion { found: u16, expected: u16 },
    #[error("unsupported weapon generator version {found}; expected {expected}")]
    GeneratorVersion { found: u16, expected: u16 },
    #[error("weapon design failed validation")]
    InvalidDesign(Vec<ValidationError>),
}

pub fn encode_holder(design: &WeaponHolderDesign) -> Result<Vec<u8>, CodecError> {
    validate_holder(design).map_err(CodecError::InvalidDesign)?;
    serialize_holder(design)
}
pub(crate) fn serialize_holder(design: &WeaponHolderDesign) -> Result<Vec<u8>, CodecError> {
    Ok(serde_json::to_vec(&HolderEnvelope {
        schema_version: HOLDER_SCHEMA_VERSION,
        generator_version: HOLDER_GENERATOR_VERSION,
        design: design.clone(),
    })?)
}

pub fn decode_holder(bytes: &[u8]) -> Result<WeaponHolderDesign, CodecError> {
    let design = parse_holder(bytes)?;
    validate_holder(&design).map_err(CodecError::InvalidDesign)?;
    Ok(design)
}
pub(crate) fn parse_holder(bytes: &[u8]) -> Result<WeaponHolderDesign, CodecError> {
    if bytes.len() > crate::MAX_ENCODED_RECIPE_BYTES {
        return Err(CodecError::TransportSize);
    }
    let envelope: HolderEnvelope = serde_json::from_slice(bytes)?;
    if envelope.schema_version != HOLDER_SCHEMA_VERSION {
        return Err(CodecError::SchemaVersion {
            found: envelope.schema_version,
            expected: HOLDER_SCHEMA_VERSION,
        });
    }
    if envelope.generator_version != HOLDER_GENERATOR_VERSION {
        return Err(CodecError::GeneratorVersion {
            found: envelope.generator_version,
            expected: HOLDER_GENERATOR_VERSION,
        });
    }
    Ok(envelope.design)
}

pub fn encode(design: &WeaponDesign) -> Result<Vec<u8>, CodecError> {
    validate(design).map_err(CodecError::InvalidDesign)?;
    serialize_weapon(design)
}
pub(crate) fn serialize_weapon(design: &WeaponDesign) -> Result<Vec<u8>, CodecError> {
    Ok(serde_json::to_vec(&Envelope {
        schema_version: SCHEMA_VERSION,
        generator_version: GENERATOR_VERSION,
        design: design.clone(),
    })?)
}

pub fn decode(bytes: &[u8]) -> Result<WeaponDesign, CodecError> {
    let design = parse_weapon(bytes)?;
    validate(&design).map_err(CodecError::InvalidDesign)?;
    Ok(design)
}
pub(crate) fn parse_weapon(bytes: &[u8]) -> Result<WeaponDesign, CodecError> {
    if bytes.len() > crate::MAX_ENCODED_RECIPE_BYTES {
        return Err(CodecError::TransportSize);
    }
    let envelope: Envelope = serde_json::from_slice(bytes)?;
    if envelope.schema_version != SCHEMA_VERSION {
        return Err(CodecError::SchemaVersion {
            found: envelope.schema_version,
            expected: SCHEMA_VERSION,
        });
    }
    if envelope.generator_version != GENERATOR_VERSION {
        return Err(CodecError::GeneratorVersion {
            found: envelope.generator_version,
            expected: GENERATOR_VERSION,
        });
    }
    Ok(envelope.design)
}
