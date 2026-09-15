use super::*;
impl CapturePose {
    pub(in crate::tactical_scene_viewer) fn accepts_plant(
        self,
        species: adventuresim_plant_generator::PlantSpecies,
    ) -> bool {
        match self {
            Self::Fungus {
                species: requested, ..
            } => species == adventuresim_plant_generator::PlantSpecies::Fungus(requested),
            _ => true,
        }
    }

    pub(in crate::tactical_scene_viewer) fn plant_camera(self, root: Vec3) -> (Vec3, Vec3, Vec3) {
        let (distance, target_height, elevation) = match self {
            Self::Plant { distance } => (distance, 0.18, distance.clamp(0.35, 1.5)),
            Self::Fungus { distance, .. } => (distance, 0.06, (distance * 0.3).clamp(0.10, 1.5)),
            _ => unreachable!("only botanical poses use plant camera framing"),
        };
        let target = root + Vec3::Y * target_height;
        let approach = Vec3::new(-root.x, 0.0, -root.z).normalize_or(Vec3::Z);
        (
            target + approach * distance + Vec3::Y * elevation,
            target,
            Vec3::Y,
        )
    }
}
pub(in crate::tactical_scene_viewer) const PLANT_REVIEW_VIEWS: [CaptureViewSpec; 4] = [
    v!(
        "warmup",
        "Production vegetation warmup",
        CapturePose::Ground,
        55.0,
        100
    )
    .warmup()
    .vista(),
    v!(
        "plant-contact",
        "Actual scattered plant and ground contact",
        CapturePose::Plant { distance: 0.4 },
        40.0,
        100
    )
    .settled_readback_pair(),
    v!(
        "plant-community",
        "Actual scattered plant at gameplay distance",
        CapturePose::Plant { distance: 4.0 },
        55.0,
        100
    )
    .settled_readback_pair()
    .vista(),
    v!(
        "plant-distance",
        "Plant community at detail fade distance",
        CapturePose::Plant { distance: 20.0 },
        55.0,
        100
    )
    .settled_readback_pair()
    .vista(),
];
