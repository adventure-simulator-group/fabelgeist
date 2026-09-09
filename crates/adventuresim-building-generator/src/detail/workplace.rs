use super::*;

impl crate::WorkplaceMaterial {
    pub(crate) fn render_material(self) -> BuildingLodMaterial {
        match self {
            Self::Timber => BuildingLodMaterial::Timber,
            Self::UnpaintedTimber => BuildingLodMaterial::InteriorTimber,
            Self::Grain => BuildingLodMaterial::Grain,
            Self::DyedCloth => BuildingLodMaterial::DyedCloth,
            Self::UndyedCloth => BuildingLodMaterial::UndyedCloth,
            Self::Hide => BuildingLodMaterial::Hide,
            Self::ProcessLiquid => BuildingLodMaterial::ProcessLiquid,
            Self::HempRope => BuildingLodMaterial::HempRope,
            Self::Masonry => BuildingLodMaterial::Wall(WallMaterialClass::CivilianMasonry),
            Self::Iron => BuildingLodMaterial::Iron,
        }
    }
}

/// Preserve the working building's major parts in both distant representations.
pub(crate) fn compile_workplace_lod(plan: &BuildingPlan) -> BuildingDetail {
    let mut detail = BuildingDetail { meshes: Vec::new() };
    if let Some(workplace) = &plan.workplace {
        for part in workplace.parts.iter().filter(|part| part.silhouette) {
            if let Some(solid) = plan
                .resolved_geometry
                .solids
                .iter()
                .find(|solid| solid.id == part.solid)
            {
                append_oriented_cuboid(&mut detail, part.material.render_material(), solid, None);
            }
        }
    }
    detail
}
