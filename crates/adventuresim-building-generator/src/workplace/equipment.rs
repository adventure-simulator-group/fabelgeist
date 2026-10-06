use super::{assembly::Assembly, *};
use crate::CELL_SIZE_METRES;

#[path = "fire.rs"]
mod fire;
#[path = "fittings.rs"]
mod fittings;
#[path = "storage.rs"]
mod storage;
use fire::*;
use fittings::*;
use storage::*;

pub(super) fn fit_workplace(
    a: &mut Assembly<'_>,
    program: &BuildingProgram,
) -> Result<(), crate::GenerationError> {
    let (width, depth) = program.footprint.dimensions();
    let dimensions = crate::spatial_geometry::PlanDimensions::from_metres(Vec2::new(
        f32::from(width) * CELL_SIZE_METRES,
        f32::from(depth) * CELL_SIZE_METRES,
    ))?;
    let _: () = match a.plan.kind {
        WorkplaceKind::Barn => barn(a, dimensions)?,
        WorkplaceKind::Stable => stable(a, dimensions)?,
        WorkplaceKind::Granary => granary(a, dimensions)?,
        WorkplaceKind::Smithy => smithy(a, dimensions)?,
        WorkplaceKind::Bakehouse => bakehouse(a, dimensions)?,
        WorkplaceKind::MarketHall => market(a, dimensions)?,
        WorkplaceKind::Brewery => super::brewing::brewery(a, dimensions)?,
        WorkplaceKind::Malthouse => super::brewing::malthouse(a, dimensions)?,
        WorkplaceKind::Warehouse => super::warehouse::fit_workplace(a, dimensions)?,
        WorkplaceKind::Dyer | WorkplaceKind::Tannery => super::wet::fit_workplace(a, dimensions)?,
        WorkplaceKind::HorseMill => super::horse_mill::fit_workplace(a, dimensions)?,
        WorkplaceKind::TimberYard | WorkplaceKind::Carpenter => {
            super::craft::fit_workplace(a, dimensions)?
        }
    };
    Ok(())
}
