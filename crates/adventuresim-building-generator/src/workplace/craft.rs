//! Timber trades reserve covered stock bays beside an uninterrupted handling lane.
use super::{assembly::Assembly, *};

mod envelope;
mod stock;

pub(super) use envelope::build_envelope;

pub(super) fn fit_workplace(a: &mut Assembly<'_>, w: f32, d: f32) {
    match a.plan.kind {
        WorkplaceKind::TimberYard => stock::timber_yard(a, w, d),
        WorkplaceKind::Carpenter => stock::joinery(a, w, d),
        _ => unreachable!("only timber trades use the craft programme"),
    }
}

#[cfg(test)]
mod tests;
