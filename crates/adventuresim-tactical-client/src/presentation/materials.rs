//! Material and asset plugins shared by tactical gameplay and capture viewers.

use super::*;

pub(super) struct TacticalMaterialsPlugin;

impl Plugin for TacticalMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<TacticalTerrainMaterial>::default(),
            MaterialPlugin::<TacticalVistaMaterial>::default(),
            MaterialPlugin::<TacticalRockMaterial>::default(),
            MaterialPlugin::<TacticalFoliageMaterial>::default(),
            MaterialPlugin::<TacticalPebbleMaterial>::default(),
            MaterialPlugin::<TacticalPebbleBillboardMaterial>::default(),
            adventuresim_procedural_materials::ProceduralMaterialsPlugin,
            MaterialPlugin::<TacticalTreeAggregateBarkMaterial>::default(),
            MaterialPlugin::<TacticalTreeImpostorMaterial>::default(),
            MaterialPlugin::<TacticalMoonMaterial>::default(),
            MaterialPlugin::<TacticalSunMaterial>::default(),
            MaterialPlugin::<TacticalStarMaterial>::default(),
            MaterialPlugin::<TacticalCloudMaterial>::default(),
            MaterialPlugin::<TacticalCloudCompositeMaterial>::default(),
        ))
        // Split from the tuple above so the material-plugin group stays within
        // Bevy's 15-element `Plugins` tuple arity limit.
        .add_plugins((
            MaterialPlugin::<TacticalWeatherMaterial>::default(),
            DoorPresentationPlugin,
            WindowPresentationPlugin,
        ))
        .add_plugins(adventuresim_procedural_textures::BakedTexturesPlugin);
    }
}
