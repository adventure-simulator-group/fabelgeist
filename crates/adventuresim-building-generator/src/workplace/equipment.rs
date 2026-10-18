use super::{assembly::Assembly, *};
use crate::CELL_SIZE_METRES;
use crate::GenerationResult as Result;

#[path = "fire.rs"]
mod fire;
#[path = "fittings.rs"]
mod fittings;
#[path = "storage.rs"]
mod storage;
use fire::*;
use fittings::*;
use storage::*;

pub(super) fn fit_workplace(assembly: &mut Assembly<'_>, program: &BuildingProgram) -> Result<()> {
    let (width, depth) = program.footprint.dimensions();
    let dimensions = crate::spatial_geometry::PlanDimensions::from_metres(Vec2::new(
        f32::from(width) * CELL_SIZE_METRES,
        f32::from(depth) * CELL_SIZE_METRES,
    ))?;
    let _: () = match assembly.plan.kind {
        WorkplaceKind::Barn => barn(assembly, dimensions)?,
        WorkplaceKind::Stable => stable(assembly, dimensions)?,
        WorkplaceKind::Granary => granary(assembly, dimensions)?,
        WorkplaceKind::Smithy => smithy(assembly, dimensions)?,
        WorkplaceKind::Bakehouse => bakehouse(assembly, dimensions)?,
        WorkplaceKind::MarketHall => market(assembly, dimensions)?,
        WorkplaceKind::Brewery => super::brewing::brewery(assembly, dimensions)?,
        WorkplaceKind::Malthouse => super::brewing::malthouse(assembly, dimensions)?,
        WorkplaceKind::Warehouse => super::warehouse::fit_workplace(assembly, dimensions)?,
        WorkplaceKind::Dyer | WorkplaceKind::Tannery => {
            super::wet::fit_workplace(assembly, dimensions)?
        }
        WorkplaceKind::HorseMill => super::horse_mill::fit_workplace(assembly, dimensions)?,
        WorkplaceKind::TimberYard | WorkplaceKind::Carpenter => {
            super::craft::fit_workplace(assembly, dimensions)?
        }
    };
    Ok(())
}
