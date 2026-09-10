//! Small, complete arrangements; every extra kit contributes its real clearances.
use super::*;
use adventuresim_building_generator::furniture::FurnitureKind;

const COMPOSITION_DOMAIN: u64 = 0x7665_6e64_7374_6f6b;
const VENDOR_COMPOSITIONS: u64 = 4;

pub(super) fn kinds(kind: FurnitureGroupKind, identity: u64) -> Vec<FurnitureKind> {
    use FurnitureKind::{Barrel, CanvasStall, CargoStack, HitchingTrough, TableBenchSet};
    let choice = mix64(identity ^ COMPOSITION_DOMAIN);
    match kind {
        FurnitureGroupKind::Vendor => match choice % VENDOR_COMPOSITIONS {
            0 => vec![CanvasStall],
            1 => vec![CanvasStall, Barrel],
            2 => vec![CanvasStall, CargoStack],
            _ => vec![CanvasStall, CargoStack, Barrel],
        },
        FurnitureGroupKind::Receiving => vec![CargoStack, CargoStack, Barrel],
        FurnitureGroupKind::HorseStop => vec![HitchingTrough, TableBenchSet],
        FurnitureGroupKind::Domestic if choice & 1 == 0 => vec![Barrel],
        FurnitureGroupKind::Domestic => vec![Barrel, Barrel],
        FurnitureGroupKind::Workshop => vec![CargoStack, Barrel],
    }
}
