//! Authored plate identities retained through fitting and export.
use std::ops::Range;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArmorComponentRole {
    Plate,
    LeatherStraps,
    Buckles,
    Fauld,
    Tassets,
    Skull,
    Bevor,
    Visor,
}

impl ArmorComponentRole {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Plate => "plate",
            Self::LeatherStraps => "leather_straps",
            Self::Buckles => "buckles",
            Self::Fauld => "fauld",
            Self::Tassets => "tassets",
            Self::Skull => "skull",
            Self::Bevor => "bevor",
            Self::Visor => "visor",
        }
    }
}

/// Reference-pose hinge in the same metre coordinate space as the mesh.
/// Descriptive only: this does not introduce an animated joint.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArmorHinge {
    pub origin: [f32; 3],
    pub axis: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArmorComponent {
    pub role: ArmorComponentRole,
    pub vertices: Range<usize>,
    pub indices: Range<usize>,
    pub hinge: Option<ArmorHinge>,
    pub material: Option<ArmorComponentMaterial>,
}

/// Unlit surface parameters; generated textures supply normal and AO detail.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArmorComponentMaterial {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}
