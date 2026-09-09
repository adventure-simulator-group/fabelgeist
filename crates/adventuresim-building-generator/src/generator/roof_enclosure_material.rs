//! Gable infill follows the resolved wall envelope's construction.
use crate::{RoofMaterial, WallAssembly, WallMaterialClass};

pub(super) fn for_walls(walls: &[WallAssembly]) -> RoofMaterial {
    if walls
        .iter()
        .any(|wall| wall.material == WallMaterialClass::TimberInfill)
    {
        RoofMaterial::TimberInfill
    } else if walls
        .iter()
        .any(|wall| wall.material == WallMaterialClass::RubbleMasonry)
    {
        RoofMaterial::RubbleInfill
    } else {
        RoofMaterial::MasonryInfill
    }
}
