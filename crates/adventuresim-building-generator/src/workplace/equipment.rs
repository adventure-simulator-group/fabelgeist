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

pub(super) fn fit_workplace(a: &mut Assembly<'_>, program: &BuildingProgram) {
    let (width, depth) = program.footprint.dimensions();
    let w = f32::from(width) * CELL_SIZE_METRES;
    let d = f32::from(depth) * CELL_SIZE_METRES;
    match a.plan.kind {
        WorkplaceKind::Barn => barn(a, w, d),
        WorkplaceKind::Stable => stable(a, w, d),
        WorkplaceKind::Granary => granary(a, w, d),
        WorkplaceKind::Smithy => smithy(a, w, d),
        WorkplaceKind::Bakehouse => bakehouse(a, w, d),
        WorkplaceKind::MarketHall => market(a, w, d),
        WorkplaceKind::Brewery => super::brewing::brewery(a, w, d),
        WorkplaceKind::Malthouse => super::brewing::malthouse(a, w, d),
        WorkplaceKind::Warehouse => super::warehouse::fit_workplace(a, w, d),
        WorkplaceKind::TimberYard | WorkplaceKind::Carpenter => {
            super::craft::fit_workplace(a, w, d)
        }
    }
}
