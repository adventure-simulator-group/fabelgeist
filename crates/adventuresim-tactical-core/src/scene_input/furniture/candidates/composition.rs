//! Small, complete arrangements; every extra kit contributes its real clearances.
use super::*;
use adventuresim_building_generator::furniture::FurnitureKind;

const COMPOSITION_DOMAIN: StreamId = StreamId::new("furniture.group-composition");
const VENDOR_COMPOSITIONS: usize = 4;

pub(super) fn kinds(kind: FurnitureGroupKind, identity: u64) -> Vec<FurnitureKind> {
    use FurnitureKind::{Barrel, CanvasStall, CargoStack, HitchingTrough, TableBenchSet};
    let mut random = COMPOSITION_DOMAIN.rng(identity, &[]);
    match kind {
        FurnitureGroupKind::Vendor => match random.index(VENDOR_COMPOSITIONS) {
            0 => vec![CanvasStall],
            1 => vec![CanvasStall, Barrel],
            2 => vec![CanvasStall, CargoStack],
            _ => vec![CanvasStall, CargoStack, Barrel],
        },
        FurnitureGroupKind::Receiving => vec![CargoStack, CargoStack, Barrel],
        FurnitureGroupKind::HorseStop => vec![HitchingTrough, TableBenchSet],
        FurnitureGroupKind::Domestic if random.boolean() => vec![Barrel],
        FurnitureGroupKind::Domestic => vec![Barrel, Barrel],
        FurnitureGroupKind::Workshop => vec![CargoStack, Barrel],
    }
}
