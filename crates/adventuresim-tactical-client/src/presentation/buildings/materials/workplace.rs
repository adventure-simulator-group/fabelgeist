//! Physical working surfaces retain their authored finish independently of facade paint.
use adventuresim_building_generator::BuildingLodMaterial;
use bevy::prelude::*;

pub(super) struct WorkplaceMaterials {
    pub(super) grain: Handle<StandardMaterial>,
    pub(super) dyed_cloth: Handle<StandardMaterial>,
    pub(super) undyed_cloth: Handle<StandardMaterial>,
    pub(super) hide: Handle<StandardMaterial>,
    pub(super) process_liquid: Handle<StandardMaterial>,
    pub(super) hemp_rope: Handle<StandardMaterial>,
    pub(super) carved_sandstone: Handle<StandardMaterial>,
    pub(super) lead_alloy: Handle<StandardMaterial>,
    pub(super) bronze: Handle<StandardMaterial>,
    pub(super) candle_wax: Handle<StandardMaterial>,
    pub(super) earthenware: Handle<StandardMaterial>,
    pub(super) glazed_tile: Handle<StandardMaterial>,
    pub(super) millstone: Handle<StandardMaterial>,
    pub(super) timber_end_grain: Handle<StandardMaterial>,
}

impl WorkplaceMaterials {
    pub(super) fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        let mut finish = |role: BuildingLodMaterial| {
            let surface = role.workplace_surface().expect("workplace surface role");
            materials.add(StandardMaterial {
                base_color: Color::srgb(surface.srgb[0], surface.srgb[1], surface.srgb[2]),
                perceptual_roughness: surface.perceptual_roughness,
                metallic: surface.metallic,
                ..default()
            })
        };
        Self {
            grain: finish(BuildingLodMaterial::Grain),
            dyed_cloth: finish(BuildingLodMaterial::DyedCloth),
            undyed_cloth: finish(BuildingLodMaterial::UndyedCloth),
            hide: finish(BuildingLodMaterial::Hide),
            process_liquid: finish(BuildingLodMaterial::ProcessLiquid),
            hemp_rope: finish(BuildingLodMaterial::HempRope),
            carved_sandstone: finish(BuildingLodMaterial::CarvedSandstone),
            lead_alloy: finish(BuildingLodMaterial::LeadAlloy),
            bronze: finish(BuildingLodMaterial::Bronze),
            candle_wax: finish(BuildingLodMaterial::CandleWax),
            earthenware: finish(BuildingLodMaterial::Earthenware),
            glazed_tile: finish(BuildingLodMaterial::GlazedTile),
            millstone: finish(BuildingLodMaterial::Millstone),
            timber_end_grain: finish(BuildingLodMaterial::TimberEndGrain),
        }
    }
}
