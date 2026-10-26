//! Material and asset plugins shared by tactical gameplay and capture viewers.

use super::*;

pub(super) struct TacticalMaterialsPlugin;

impl Plugin for TacticalMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, terrain::load_terrain_prepass_shader);
        app.add_plugins((
            MaterialPlugin::<TacticalTerrainMaterial>::default(),
            MaterialPlugin::<TacticalVistaMaterial>::default(),
            MaterialPlugin::<TacticalRockMaterial>::default(),
            MaterialPlugin::<TacticalFoliageMaterial>::default(),
            adventuresim_procedural_materials::ProceduralMaterialsPlugin,
            MaterialPlugin::<TacticalTreeAggregateBarkMaterial>::default(),
            MaterialPlugin::<TacticalTreeImpostorMaterial>::default(),
            MaterialPlugin::<TacticalMoonMaterial>::default(),
            MaterialPlugin::<TacticalStarMaterial>::default(),
            MaterialPlugin::<TacticalCloudMaterial>::default(),
            MaterialPlugin::<TacticalCloudCompositeMaterial>::default(),
        ))
        // Split from the tuple above so the material-plugin group stays within
        // Bevy's 15-element `Plugins` tuple arity limit.
        .add_plugins((
            // Loose stone renders through bevy_eidolon: one batch per patch
            // mesh and LOD instead of two or three entities per square metre.
            bevy_eidolon::prelude::InstancedMaterialPlugin::<TacticalPebbleMaterial>::default(),
            bevy_eidolon::prelude::GpuCullComputePlugin::<TacticalPebbleMaterial>::default(),
            bevy_eidolon::prelude::InstancedMaterialPlugin::<TacticalPebbleBillboardMaterial>::default(),
            bevy_eidolon::prelude::GpuCullComputePlugin::<TacticalPebbleBillboardMaterial>::default(),
            bevy_eidolon::prelude::InstancedMaterialPlugin::<
                crate::presentation::vista::tree_cards::TacticalVistaTreeInstancedMaterial,
            >::default(),
            bevy_eidolon::prelude::GpuCullComputePlugin::<
                crate::presentation::vista::tree_cards::TacticalVistaTreeInstancedMaterial,
            >::default(),
            MaterialPlugin::<CityGroundMaterial>::default(),
            MaterialPlugin::<TacticalWeatherMaterial>::default(),
            DoorPresentationPlugin,
            WindowPresentationPlugin,
        ))
        .add_plugins(adventuresim_procedural_textures::BakedTexturesPlugin)
        .add_plugins(ground_scatter::plants::PlantPresentationPlugin);
    }
}
