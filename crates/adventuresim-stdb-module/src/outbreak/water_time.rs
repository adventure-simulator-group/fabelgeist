//! Translate durable water lot anchors into elapsed contamination time.

use adventuresim_world_schema::calendar::StrategicMinute;

use super::WaterMaterialLot;

pub(super) fn concentration_at_collection(lot: &WaterMaterialLot, at: StrategicMinute) -> f32 {
    adventuresim_core::food::contamination_at(
        lot.concentration_anchor,
        lot.growth_per_hour,
        at.elapsed_since(lot.anchor_minute),
    )
}
