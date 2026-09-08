//! Authored roof proportions shared by the civilian fixture programmes.
use super::*;
impl RoofPiece {
    pub(super) fn civilian(program: &BuildingProgram) -> Self {
        let (kind, ridge_axis, eave_metres, gable_profile) = match program.archetype {
            BuildingArchetype::TownHouse
            | BuildingArchetype::ParishChurch
            | BuildingArchetype::Workplace => {
                (RoofKind::Gable, RidgeAxis::Z, 0.45, GableProfile::Plain)
            }
            BuildingArchetype::HallHouse => {
                (RoofKind::HalfHip, RidgeAxis::Z, 0.65, GableProfile::Plain)
            }
            BuildingArchetype::FachwerkCottage => {
                (RoofKind::Gable, RidgeAxis::Z, 0.5, GableProfile::Plain)
            }
            BuildingArchetype::FachwerkMerchantHouse => {
                (RoofKind::Gable, RidgeAxis::Z, 0.55, GableProfile::Plain)
            }
            BuildingArchetype::RenaissanceTownHall => {
                (RoofKind::HalfHip, RidgeAxis::X, 0.65, GableProfile::Stepped)
            }
            _ => unreachable!("only civilian fixtures use a single civilian roof"),
        };
        let (width, depth) = program.footprint.dimensions();
        let size = Vec2::new(f32::from(width), f32::from(depth)) * CELL_SIZE_METRES;
        Self {
            kind,
            centre: size * 0.5,
            size,
            base_height_metres: program.storeys.len() as f32 * program.storey_height_metres,
            pitch_degrees: program.roof_pitch_degrees,
            ridge_axis,
            eave_metres,
            gable_profile,
        }
    }
}
