//! Timber trades reserve covered stock bays beside an uninterrupted handling lane.
use super::{assembly::Assembly, *};
use crate::GenerationResult as Result;

mod envelope;
mod stock;

pub(super) use envelope::build_envelope;

pub(super) fn fit_workplace(
    assembly: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> Result<()> {
    let _: () = match assembly.plan.kind {
        WorkplaceKind::TimberYard => stock::timber_yard(assembly, dimensions)?,
        WorkplaceKind::Carpenter => stock::joinery(assembly, dimensions)?,
        _ => unreachable!("only timber trades use the craft programme"),
    };
    Ok(())
}

#[cfg(test)]
mod tests;
