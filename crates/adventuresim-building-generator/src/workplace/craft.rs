//! Timber trades reserve covered stock bays beside an uninterrupted handling lane.
use super::{assembly::Assembly, *};

mod envelope;
mod stock;

pub(super) use envelope::build_envelope;

pub(super) fn fit_workplace(
    a: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> Result<(), crate::GenerationError> {
    let _: () = match a.plan.kind {
        WorkplaceKind::TimberYard => stock::timber_yard(a, dimensions)?,
        WorkplaceKind::Carpenter => stock::joinery(a, dimensions)?,
        _ => unreachable!("only timber trades use the craft programme"),
    };
    Ok(())
}

#[cfg(test)]
mod tests;
