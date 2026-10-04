//! Sorted, conserved transfer of persisted private water contributions.

use super::{
    ContainerWaterContribution, WaterContributionTransferError, container_water_contribution,
};
use adventuresim_core::{
    material::{MaterialLotId, Microliters},
    physical_object::PhysicalObjectId,
    water_source::{WaterContaminantMicrounits, WaterMaterialContribution},
};
use spacetimedb::ReducerContext;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TransferredWaterContribution {
    lot: MaterialLotId,
    material: WaterMaterialContribution,
}

impl TransferredWaterContribution {
    fn try_sample(
        row: &ContainerWaterContribution,
        container: PhysicalObjectId,
        source_total: Microliters,
        moved: Microliters,
    ) -> Result<Self, WaterContributionTransferError> {
        let lot = MaterialLotId::try_new(row.material_lot_id).map_err(|source| {
            WaterContributionTransferError::InvalidMaterialLot {
                stored_lot_id: row.material_lot_id,
                source,
            }
        })?;
        let material = WaterMaterialContribution::new(
            Microliters::new(row.amount_microliters),
            WaterContaminantMicrounits::new(row.contaminant_load_microunits),
        )
        .sample(source_total, moved)
        .ok_or(WaterContributionTransferError::ExceedsPublicVolume {
            container,
            source_total,
            moved,
        })?;
        Ok(Self { lot, material })
    }

    pub(crate) const fn lot(self) -> MaterialLotId {
        self.lot
    }

    pub(crate) const fn material(self) -> WaterMaterialContribution {
        self.material
    }
}

pub(crate) fn take_container_water_contributions(
    ctx: &ReducerContext,
    container_object_id: PhysicalObjectId,
    source_total: Microliters,
    moved_water: Microliters,
) -> Result<Vec<TransferredWaterContribution>, WaterContributionTransferError> {
    let exceeds_volume = || WaterContributionTransferError::ExceedsPublicVolume {
        container: container_object_id,
        source_total,
        moved: moved_water,
    };
    if moved_water > source_total {
        return Err(exceeds_volume());
    }
    let mut rows = ctx
        .db
        .container_water_contribution()
        .container_object_id()
        .filter(container_object_id.get())
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.material_lot_id);
    let mut moved = Vec::new();
    for mut row in rows {
        let selected = TransferredWaterContribution::try_sample(
            &row,
            container_object_id,
            source_total,
            moved_water,
        )?;
        let contribution = WaterMaterialContribution::new(
            Microliters::new(row.amount_microliters),
            WaterContaminantMicrounits::new(row.contaminant_load_microunits),
        );
        if selected.material().volume().is_zero() {
            continue;
        }
        row.amount_microliters = contribution
            .volume()
            .checked_sub(selected.material().volume())
            .expect("a proportional sample cannot exceed its source volume")
            .get();
        row.contaminant_load_microunits = contribution
            .contaminant_load()
            .checked_sub(selected.material().contaminant_load())
            .expect("a proportional sample cannot exceed its source load")
            .get();
        if row.amount_microliters == 0 {
            ctx.db
                .container_water_contribution()
                .material_lot_id()
                .delete(row.material_lot_id);
        } else {
            ctx.db
                .container_water_contribution()
                .material_lot_id()
                .update(row.clone());
        }
        moved.push(selected);
    }
    Ok(moved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_core::material::MaterialError;
    use adventuresim_world_schema::calendar::StrategicMinute;
    use std::error::Error;

    #[test]
    fn persisted_water_admission_retains_lot_and_exact_transfer_context() {
        let container = PhysicalObjectId::try_new(11).unwrap();
        let mut row = ContainerWaterContribution {
            material_lot_id: 7,
            container_object_id: container.get(),
            amount_microliters: 7,
            contaminant_load_microunits: 13,
            collected_at: StrategicMinute::ZERO,
        };
        let selected = TransferredWaterContribution::try_sample(
            &row,
            container,
            Microliters::new(11),
            Microliters::new(5),
        )
        .unwrap();
        assert_eq!(selected.lot(), MaterialLotId::try_new(7).unwrap());
        assert_eq!(selected.material().volume(), Microliters::new(3));
        assert_eq!(
            selected.material().contaminant_load(),
            WaterContaminantMicrounits::new(5)
        );

        let error = TransferredWaterContribution::try_sample(
            &row,
            container,
            Microliters::new(11),
            Microliters::new(12),
        )
        .unwrap_err();
        assert!(
            matches!(error, WaterContributionTransferError::ExceedsPublicVolume {
            container: actual,
            source_total,
            moved,
        } if actual == container && source_total == Microliters::new(11) && moved == Microliters::new(12))
        );

        row.material_lot_id = 0;
        let error = TransferredWaterContribution::try_sample(
            &row,
            container,
            Microliters::new(11),
            Microliters::new(5),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            WaterContributionTransferError::InvalidMaterialLot {
                stored_lot_id: 0,
                ..
            }
        ));
        assert_eq!(
            error.source().unwrap().downcast_ref::<MaterialError>(),
            Some(&MaterialError::ZeroLotId)
        );
        assert_eq!(row.amount_microliters, 7);
        assert_eq!(row.contaminant_load_microunits, 13);
    }
}
