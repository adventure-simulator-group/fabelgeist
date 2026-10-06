//! Wet trades combine roofed working rooms with real vessels and supported drying equipment.
use super::{assembly::Assembly, *};
use crate::GenerationResult;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, PlanDirection, Position, PositiveLength,
};
use crate::{GableProfile, RidgeAxis, RoofKind, RoofPiece};

mod envelope;
mod soaking;
#[cfg(test)]
mod tests;
mod textiles;
pub(super) use envelope::build_envelope;

pub(super) fn service_roof(main: Vec2) -> RoofPiece {
    RoofPiece {
        kind: RoofKind::Gable,
        centre: Vec2::new(main.x + 3.8, main.y * 0.5),
        size: Vec2::new(3.2, main.y - 1.2),
        base_height_metres: 3.0,
        pitch_degrees: 24.0,
        ridge_axis: RidgeAxis::Z,
        eave_metres: 0.16,
        gable_profile: GableProfile::Plain,
    }
}

pub(super) fn fit_workplace(
    assembly: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> GenerationResult<()> {
    let width = dimensions.metres().x;
    let depth = dimensions.metres().y;
    let _: () = match assembly.plan.kind {
        WorkplaceKind::Dyer => {
            soaking::dye_kettle(assembly, Vec2::new(1.7, 2.2))?;
            textiles::dye_frames(assembly, dimensions)?;
        }
        WorkplaceKind::Tannery => {
            drying_canopy(assembly, dimensions)?;
            let count = 2 + assembly.plan.size.extra_bays();
            for bay in 0..count {
                soaking::tank(assembly, Vec2::new(width + 3.8, 2.2 + f32::from(bay) * 3.0))?;
            }
            textiles::hide_frame(
                assembly,
                crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(
                    width + 3.8,
                    depth - 1.2,
                ))?,
            )?;
            soaking::fleshing_beam(assembly, Vec2::new(1.7, 2.2))?;
            soaking::fleshing_beam(assembly, Vec2::new(width - 1.7, depth - 2.4))?;
        }
        _ => unreachable!("only dyeing and tanning use the wet-trade programme"),
    };
    Ok(())
}

fn drying_canopy(
    assembly: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> GenerationResult<()> {
    let width = dimensions.metres().x;
    let depth = dimensions.metres().y;
    let front = 0.6;
    let back = depth - 0.6;
    let bays = ((back - front) / 3.0).ceil() as u32;
    for x in [width + 2.2, width + 5.4] {
        for bay in 0..=bays {
            let z = front + (back - front) * bay as f32 / bays as f32;
            assembly.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.4, z))?,
                CuboidDimensions::from_metres(Vec3::new(0.24, 2.8, 0.24))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        assembly.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, front))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, back))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x < width + 3.8 {
                Vec2::NEG_X
            } else {
                Vec2::X
            })?,
            Elevation::<crate::Architectural>::from_metres(2.8)?,
            PositiveLength::from_metres(0.2)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    }
    let _: () = for z in [front, back] {
        assembly.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(width + 2.2, z))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(width + 5.4, z))?,
            PlanDirection::<crate::Architectural>::from_normalized(if z == front {
                Vec2::NEG_Y
            } else {
                Vec2::Y
            })?,
            Elevation::<crate::Architectural>::from_metres(2.8)?,
            PositiveLength::from_metres(0.2)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    };
    Ok(())
}
