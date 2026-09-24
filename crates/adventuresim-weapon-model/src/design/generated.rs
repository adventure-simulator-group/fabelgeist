//! Renderer-independent output of canonical weapon construction.
use super::WeaponHolderKind;
use crate::DerivedProperties;
use crate::recipe::Material;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Anchor {
    pub name: String,
    pub position: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshPart {
    pub component_id: String,
    pub material: Material,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub bounds: Bounds,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeneratedWeapon {
    pub design_hash: crate::DesignHash,
    pub parts: Vec<MeshPart>,
    pub bounds: Bounds,
    pub anchors: Vec<Anchor>,
    pub derived: DerivedProperties,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeneratedWeaponHolder {
    pub design_hash: crate::DesignHash,
    pub kind: WeaponHolderKind,
    /// Holder coordinates use the same recipe-local frame as the weapon.
    pub grip: [f32; 3],
    pub parts: Vec<MeshPart>,
    pub bounds: Bounds,
    pub derived: DerivedProperties,
}
