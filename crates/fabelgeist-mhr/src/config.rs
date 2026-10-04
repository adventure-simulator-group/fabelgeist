//! Runtime mesh detail and pose-corrective choices shared by loading and export.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod error;
pub use error::{CharacterLodError, CharacterLodViolation};

/// Runtime/export detail. Denser source meshes belong to offline baking.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CharacterLod {
    #[default]
    Detailed,
    Reduced,
    Minimal,
}
impl CharacterLod {
    pub const ALL: [Self; 3] = [Self::Detailed, Self::Reduced, Self::Minimal];

    /// Authored topology size for the released MHR mesh.
    pub fn vertices(self) -> MeshVertexCount {
        MeshVertexCount(match self {
            Self::Detailed => 2_461,
            Self::Reduced => 971,
            Self::Minimal => 595,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeshVertexCount(usize);
impl From<usize> for MeshVertexCount {
    fn from(vertices: usize) -> Self {
        Self(vertices)
    }
}
impl std::fmt::Display for MeshVertexCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
impl TryFrom<u8> for CharacterLod {
    type Error = CharacterLodError;

    fn try_from(value: u8) -> Result<Self, CharacterLodError> {
        match value {
            4 => Ok(Self::Detailed),
            5 => Ok(Self::Reduced),
            6 => Ok(Self::Minimal),
            _ => Err(CharacterLodError::unsupported(value)),
        }
    }
}
impl std::str::FromStr for CharacterLod {
    type Err = CharacterLodError;

    fn from_str(value: &str) -> Result<Self, CharacterLodError> {
        let encoded = value.parse::<u8>().map_err(
            |source: std::num::ParseIntError| -> CharacterLodError {
                CharacterLodError::invalid_encoding(value, source)
            },
        )?;
        Self::try_from(encoded).map_err(|source: CharacterLodError| -> CharacterLodError {
            CharacterLodError::from_spelling(value, source)
        })
    }
}
impl std::fmt::Display for CharacterLod {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Detailed => "4",
            Self::Reduced => "5",
            Self::Minimal => "6",
        })
    }
}
impl Serialize for CharacterLod {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(match self {
            Self::Detailed => 4,
            Self::Reduced => 5,
            Self::Minimal => 6,
        })
    }
}
impl<'de> Deserialize<'de> for CharacterLod {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Whether loading/forward evaluation uses the corrective network if present.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PoseCorrectivePolicy {
    #[default]
    Enabled,
    Disabled,
}
impl From<bool> for PoseCorrectivePolicy {
    fn from(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
}
impl std::fmt::Display for PoseCorrectivePolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        })
    }
}
impl Serialize for PoseCorrectivePolicy {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(*self == Self::Enabled)
    }
}
impl<'de> Deserialize<'de> for PoseCorrectivePolicy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bool::deserialize(deserializer).map(Self::from)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoseCorrectiveAvailability {
    Available,
    Unavailable,
}

/// Validated loading choices. Serialization retains numeric LOD and a boolean
/// corrective flag at the external boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MhrConfig {
    pub lod: CharacterLod,
    pub pose_correctives: PoseCorrectivePolicy,
}

#[cfg(test)]
mod tests;
