//! Database cursor conversion for strategic blood-exposure windows.

use adventuresim_core::{
    disease::{DiseaseId, STARTER_DISEASES, TransmissionVector},
    filth::{self, Deposit},
};
use adventuresim_world_schema::calendar::StrategicMinute;

pub(super) fn blood_infectious_windows(
    deposits: &[Deposit],
    disease_id: DiseaseId,
    from: StrategicMinute,
    to: StrategicMinute,
) -> Vec<(StrategicMinute, StrategicMinute)> {
    filth::blood_infectious_windows(deposits, disease_id, from, to)
}

pub(super) fn has_active_compatible_foreign_blood(
    deposits: &[Deposit],
    from: StrategicMinute,
    to: StrategicMinute,
) -> bool {
    STARTER_DISEASES
        .iter()
        .filter(|definition| definition.supports(TransmissionVector::Blood))
        .any(|definition| !blood_infectious_windows(deposits, definition.id, from, to).is_empty())
}
