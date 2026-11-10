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
    Buffe,
    Besagew,
    JointExtension,
    OuterFabric,
    Undercloth,
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
            Self::Buffe => "buffe",
            Self::Besagew => "besagew",
            Self::JointExtension => "joint_extension",
            Self::OuterFabric => "outer_fabric",
            Self::Undercloth => "undercloth",
        }
    }

    /// The name of the component's trim band.
    pub const fn trim_name(self) -> &'static str {
        match self {
            Self::Plate => "plate.trim",
            Self::LeatherStraps => "leather_straps.trim",
            Self::Buckles => "buckles.trim",
            Self::Fauld => "fauld.trim",
            Self::Tassets => "tassets.trim",
            Self::Skull => "skull.trim",
            Self::Bevor => "bevor.trim",
            Self::Visor => "visor.trim",
            Self::Buffe => "buffe.trim",
            Self::Besagew => "besagew.trim",
            Self::JointExtension => "joint_extension.trim",
            Self::OuterFabric => "outer_fabric.trim",
            Self::Undercloth => "undercloth.trim",
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
    pub mount: Option<crate::PlateMount>,
    pub material: Option<ArmorComponentMaterial>,
}

/// Unlit surface parameters; generated textures supply normal and AO detail.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArmorComponentMaterial {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}
