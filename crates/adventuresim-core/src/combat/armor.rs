use serde::{Deserialize, Serialize};

use crate::equipment::ArmorSurface;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmorImpactOutcome {
    Stopped,
    Deflected,
    Penetrated,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArmorImpact {
    pub surface: ArmorSurface,
    pub outcome: ArmorImpactOutcome,
    pub resisted_energy_joules: f32,
    pub transmitted_energy_joules: f32,
    pub penetrated_energy_joules: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ContactEnergyResolution {
    pub armor_impact: Option<ArmorImpact>,
    pub cut_energy_joules: f32,
    pub blunt_energy_joules: f32,
}

pub(super) fn effective_edge_resistance(
    surface: ArmorSurface,
    precision: super::ContactPrecision,
) -> f32 {
    precision.resistance(surface.resistance)
}

pub(super) fn resolve_contact_energy(
    surface: Option<ArmorSurface>,
    attack: f32,
    incident_energy_joules: f32,
    precision: super::ContactPrecision,
) -> ContactEnergyResolution {
    let incident = incident_energy_joules.max(0.0);
    if incident <= f32::EPSILON {
        return ContactEnergyResolution {
            armor_impact: None,
            cut_energy_joules: 0.0,
            blunt_energy_joules: 0.0,
        };
    }
    let edge_energy = incident * precision.concentrated_fraction();
    let blunt_energy = incident - edge_energy;
    let Some(surface) = surface else {
        return ContactEnergyResolution {
            armor_impact: None,
            cut_energy_joules: edge_energy,
            blunt_energy_joules: blunt_energy,
        };
    };

    let penetrated = (edge_energy - effective_edge_resistance(surface, precision)).max(0.0);
    let stopped_edge = edge_energy - penetrated;
    // Some stopped edge energy deforms the armor into the body; the remainder
    // is reflected or retained by the armor. Blunt-channel energy starts as a
    // transmission candidate. Padding consumes that shared budget once.
    let edge_transmission_candidate = stopped_edge * 0.5;
    let transmission_candidate = edge_transmission_candidate + blunt_energy;
    let padding_resisted = transmission_candidate.min(surface.padding.max(0.0));
    let transmitted = transmission_candidate - padding_resisted;
    let resisted = stopped_edge - edge_transmission_candidate + padding_resisted;
    debug_assert!((incident - resisted - transmitted - penetrated).abs() < 0.001);
    let impact = ArmorImpact {
        surface,
        outcome: if penetrated > f32::EPSILON {
            ArmorImpactOutcome::Penetrated
        } else if attack < 1.0 {
            ArmorImpactOutcome::Deflected
        } else {
            ArmorImpactOutcome::Stopped
        },
        resisted_energy_joules: resisted,
        transmitted_energy_joules: transmitted,
        penetrated_energy_joules: penetrated,
    };
    ContactEnergyResolution {
        armor_impact: Some(impact),
        cut_energy_joules: penetrated,
        blunt_energy_joules: transmitted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_catalog_schema::EquipmentMaterial;

    #[test]
    fn rigid_steel_surface_does_not_inherit_garment_flexibility() {
        let surface = ArmorSurface {
            inventory_item_id: Some(7),
            material: Some(EquipmentMaterial::RoughSteel),
            resistance: 100.0,
            padding: 5.0,
            flexibility: 0.8,
        };
        assert_eq!(
            effective_edge_resistance(surface, super::super::ContactPrecision::new(1.0)),
            100.0
        );
    }

    #[test]
    fn armor_partition_conserves_incident_energy_across_channels() {
        let surface = ArmorSurface {
            inventory_item_id: Some(7),
            material: Some(EquipmentMaterial::RoughSteel),
            resistance: 52.0,
            padding: 18.0,
            flexibility: 0.4,
        };
        for incident in [0.01, 1.0, 20.0, 76.5, 200.0] {
            for precision in [0.0, 0.1, 0.5, 1.0, 2.0, 4.0] {
                let resolved = resolve_contact_energy(
                    Some(surface),
                    1.0,
                    incident,
                    super::super::ContactPrecision::new(precision),
                );
                let impact = resolved.armor_impact.unwrap();
                let partition = impact.resisted_energy_joules
                    + impact.transmitted_energy_joules
                    + impact.penetrated_energy_joules;
                assert!((partition - incident).abs() < 0.001);
                assert!(
                    resolved.cut_energy_joules + resolved.blunt_energy_joules <= incident + 0.001
                );
            }
        }
    }
}
