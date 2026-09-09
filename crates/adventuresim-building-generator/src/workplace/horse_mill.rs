//! Static horse-driven grist mill with an honest animal circuit and overhead sweep clearance.
use super::{assembly::Assembly, *};

mod clearance;
mod drive;
mod envelope;
mod milling;
pub(super) use clearance::audit_circuit;
pub(super) use envelope::build_envelope;

const DRIVE_CENTRE: Vec2 = Vec2::new(6.5, 6.0);
const ANIMAL_TRACK_INNER_METRES: f32 = 3.2;
const ANIMAL_TRACK_OUTER_METRES: f32 = 4.5;
#[cfg(test)]
mod tests;

pub(super) fn fit_workplace(a: &mut Assembly<'_>, _w: f32, _d: f32) {
    drive::assemble(a, DRIVE_CENTRE);
    milling::stone_and_hopper(a, DRIVE_CENTRE + Vec2::new(2.12, 0.0));
    for bay in 0..=a.plan.size.extra_bays() {
        for x in [3.5, 9.0] {
            milling::grain_bin(a, Vec2::new(x, 13.4 + f32::from(bay) * 3.0));
        }
    }
}
