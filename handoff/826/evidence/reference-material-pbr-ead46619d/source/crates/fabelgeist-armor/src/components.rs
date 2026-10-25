//! Authored plate identities and their single retained appearance source.

use crate::{SurfaceMaterial, TextileAppearance};
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Reference-pose hinge in the same metre coordinate space as the mesh.
/// Descriptive only: this does not introduce an animated joint.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArmorHinge {
    pub origin: [f32; 3],
    pub axis: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmorComponent {
    pub role: ArmorComponentRole,
    pub vertices: Range<usize>,
    pub indices: Range<usize>,
    pub hinge: Option<ArmorHinge>,
    pub mount: Option<crate::PlateMount>,
    /// One source of scalar appearance or procedural cloth appearance.
    pub appearance: ComponentAppearance,
}

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
    BodiceFabric,
    WaistSupportBand,
    WaistFrontBand,
    CollarFabric,
    EdgeBinding,
    Neckband,
    CollarTrim,
    ClothTies,
    ClothFasteners,
    Undercloth,
    Lining,
    Interlining,
    Boning,
    Hoops,
}

/// Inheritance, an authored scalar surface, or a mutable procedural textile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentAppearance {
    Inherit,
    Surface(SurfaceMaterial),
    Textile(TextileAppearance),
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
            Self::BodiceFabric => "bodice_fabric",
            Self::WaistSupportBand => "waist_support_band",
            Self::WaistFrontBand => "waist_front_band",
            Self::CollarFabric => "collar_fabric",
            Self::EdgeBinding => "edge_binding",
            Self::Neckband => "neckband",
            Self::CollarTrim => "collar_trim",
            Self::ClothTies => "cloth_ties",
            Self::ClothFasteners => "cloth_fasteners",
            Self::Undercloth => "undercloth",
            Self::Lining => "lining",
            Self::Interlining => "interlining",
            Self::Boning => "boning",
            Self::Hoops => "hoops",
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
            Self::BodiceFabric => "bodice_fabric.trim",
            Self::WaistSupportBand => "waist_support_band.trim",
            Self::WaistFrontBand => "waist_front_band.trim",
            Self::CollarFabric => "collar_fabric.trim",
            Self::EdgeBinding => "edge_binding.trim",
            Self::Neckband => "neckband.trim",
            Self::CollarTrim => "collar_trim.trim",
            Self::ClothTies => "cloth_ties.trim",
            Self::ClothFasteners => "cloth_fasteners.trim",
            Self::Undercloth => "undercloth.trim",
            Self::Lining => "lining.trim",
            Self::Interlining => "interlining.trim",
            Self::Boning => "boning.trim",
            Self::Hoops => "hoops.trim",
        }
    }
}

impl ComponentAppearance {
    pub fn material(&self) -> Option<SurfaceMaterial> {
        match self {
            Self::Inherit => None,
            Self::Surface(material) => Some(*material),
            Self::Textile(fabric) => Some(fabric.material()),
        }
    }

    pub fn textile(&self) -> Option<&TextileAppearance> {
        match self {
            Self::Textile(fabric) => Some(fabric),
            Self::Inherit | Self::Surface(_) => None,
        }
    }

    pub fn textile_mut(&mut self) -> Option<&mut TextileAppearance> {
        match self {
            Self::Textile(fabric) => Some(fabric),
            Self::Inherit | Self::Surface(_) => None,
        }
    }
}
