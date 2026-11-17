//! Counted stock and exact remaining load share one combat mass calculation.

use adventuresim_core::inventory_measurement::InventoryMassKg;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{ItemProperties, TacticalItemQuantity};

/// Authored stock uses unit mass and count. Durable lots and filled containers
/// supply the exact mass of this row, excluding separately carried child rows.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[reflect(opaque)]
#[reflect(Component, PartialEq, Clone, Serialize, Deserialize)]
pub enum TacticalInventoryMass {
    #[default]
    Counted,
    Measured(InventoryMassKg),
}

impl TacticalInventoryMass {
    pub(super) fn carried_kilograms(
        self,
        properties: &ItemProperties,
        quantity: TacticalItemQuantity,
    ) -> f32 {
        match self {
            // Unit kilograms and counts enter the existing f32 combat kernel
            // here. Measured rows already include their entire remaining load.
            Self::Counted => properties.weight * quantity.get() as f32,
            Self::Measured(mass) => mass.kilograms(),
        }
    }
}
