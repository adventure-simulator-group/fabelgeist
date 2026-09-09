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
}

impl WorkplaceMaterials {
    pub(super) fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        let mut finish = |role: BuildingLodMaterial| {
            let surface = role.workplace_surface().expect("workplace surface role");
            materials.add(StandardMaterial {
                base_color: Color::srgb(surface.srgb[0], surface.srgb[1], surface.srgb[2]),
                perceptual_roughness: surface.perceptual_roughness,
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
        }
    }
}
