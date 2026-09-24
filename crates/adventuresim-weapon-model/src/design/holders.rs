//! Body-mounted fixtures fitted to a complete weapon recipe.
use super::{Millimeters, Permille, WeaponDesign};
use crate::recipe::Material;
use serde::{Deserialize, Serialize};

/// Render-only carry fixture derived from the complete weapon recipe.
///
/// Blade weapons receive a fitted, full-length sheath or scabbard. Compact
/// hafted weapons receive a leather frog/loop around the grip. Long polearms
/// deliberately have no body-mounted holder.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum WeaponHolderKind {
    BladeSheath,
    HaftLoop,
}

/// A durable, smithable holder recipe. The fitted weapon recipe is captured
/// at fitting time so the holder remains independently reproducible even when
/// it is empty or the weapon later changes custody.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponHolderDesign {
    pub catalog_id: String,
    pub kind: WeaponHolderKind,
    pub fitted_weapon: WeaponDesign,
    pub body_material: Material,
    pub fitting_material: Material,
    pub clearance: Millimeters,
    pub wall_thickness: Millimeters,
    pub throat_length: Millimeters,
    pub chape_length: Millimeters,
    pub loop_position: Permille,
    pub loop_bar_radius: Millimeters,
    pub hanger_width: Millimeters,
    pub hanger_height: Millimeters,
}
