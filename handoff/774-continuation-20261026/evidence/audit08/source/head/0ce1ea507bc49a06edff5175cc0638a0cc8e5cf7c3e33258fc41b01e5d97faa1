//! A building selects its deterministic palette once for all modular parts.
use super::*;

#[derive(Clone, Copy)]
pub(crate) struct BuildingPalette<'a> {
    pub(super) assets: &'a TacticalBuildingMaterials,
    pub(super) appearance: &'a AppearanceMaterials,
}

impl BuildingPalette<'_> {
    pub(crate) fn get(self, material: BuildingLodMaterial) -> Handle<StandardMaterial> {
        let palette = self.appearance;
        match material {
            BuildingLodMaterial::Wall(WallMaterialClass::TimberInfill) => palette.infill.clone(),
            BuildingLodMaterial::Wall(WallMaterialClass::CivilianMasonry)
                if palette.finish == FacadeFinish::FullyRendered =>
            {
                palette.infill.clone()
            }
            BuildingLodMaterial::Wall(WallMaterialClass::CivilianMasonry) => {
                self.assets.brick.clone()
            }
            BuildingLodMaterial::Wall(WallMaterialClass::RubbleMasonry) => {
                self.assets.rubble.clone()
            }
            BuildingLodMaterial::Wall(
                WallMaterialClass::InternalTimber | WallMaterialClass::InternalMasonry,
            ) => self.assets.interior_plaster.clone(),
            BuildingLodMaterial::Wall(_) | BuildingLodMaterial::CrownMasonry => {
                self.assets.stone.clone()
            }
            BuildingLodMaterial::Roof(RoofMaterial::ClayTile) => palette.tile.clone(),
            BuildingLodMaterial::Roof(RoofMaterial::Slate) => self.assets.slate.clone(),
            BuildingLodMaterial::Roof(RoofMaterial::Lead) => self.assets.lead.clone(),
            BuildingLodMaterial::Roof(RoofMaterial::TimberShingle) => {
                self.assets.timber_roof.clone()
            }
            BuildingLodMaterial::Roof(RoofMaterial::TimberInfill) => palette.timber.clone(),
            BuildingLodMaterial::Roof(RoofMaterial::MasonryInfill) => self.assets.stone.clone(),
            BuildingLodMaterial::Roof(RoofMaterial::RubbleInfill) => self.assets.rubble.clone(),
            BuildingLodMaterial::Timber => palette.timber.clone(),
            BuildingLodMaterial::InteriorTimber => self.assets.interior_timber.clone(),
            BuildingLodMaterial::DressedStone => self.assets.stone.clone(),
            BuildingLodMaterial::Iron => self.assets.iron.clone(),
            BuildingLodMaterial::CarvedSandstone => self.assets.workplace.carved_sandstone.clone(),
            BuildingLodMaterial::LeadAlloy => self.assets.workplace.lead_alloy.clone(),
            BuildingLodMaterial::Bronze => self.assets.workplace.bronze.clone(),
            BuildingLodMaterial::CandleWax => self.assets.workplace.candle_wax.clone(),
            BuildingLodMaterial::Earthenware => self.assets.workplace.earthenware.clone(),
            BuildingLodMaterial::GlazedTile => self.assets.workplace.glazed_tile.clone(),
            BuildingLodMaterial::Millstone => self.assets.workplace.millstone.clone(),
            BuildingLodMaterial::FurnitureWood(surface) => {
                self.assets.furniture_wood[surface as usize].clone()
            }
            BuildingLodMaterial::TimberEndGrain => self.assets.workplace.timber_end_grain.clone(),
            BuildingLodMaterial::Grain => self.assets.workplace.grain.clone(),
            BuildingLodMaterial::DyedCloth => self.assets.workplace.dyed_cloth.clone(),
            BuildingLodMaterial::UndyedCloth => self.assets.workplace.undyed_cloth.clone(),
            BuildingLodMaterial::Hide => self.assets.workplace.hide.clone(),
            BuildingLodMaterial::ProcessLiquid => self.assets.workplace.process_liquid.clone(),
            BuildingLodMaterial::HempRope => self.assets.workplace.hemp_rope.clone(),
            BuildingLodMaterial::InteriorPlaster => self.assets.interior_plaster.clone(),
            BuildingLodMaterial::Floor => self.assets.floor.clone(),
            BuildingLodMaterial::Glass => self.assets.glass.clone(),
            BuildingLodMaterial::FacadeDetails => self.assets.details.clone(),
            BuildingLodMaterial::CrownMask => self.assets.crown_mask.clone(),
        }
    }
}
