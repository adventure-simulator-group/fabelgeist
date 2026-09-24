//! Mapping assessed humours to regional health segments.
use crate::medical::HumourVitals;
use adventuresim_core::physiology::Humour;

pub(super) fn assessed_humours(
    values: Option<HumourVitals>,
    scale: f32,
) -> Vec<(Humour, &'static str, f32)> {
    if let Some(values) = values {
        vec![
            (
                adventuresim_core::physiology::Humour::Sanguine,
                "attribute-health-sanguine",
                values.sanguine.abs() * scale,
            ),
            (
                adventuresim_core::physiology::Humour::Phlegmatic,
                "attribute-health-phlegmatic",
                values.phlegmatic.abs() * scale,
            ),
            (
                adventuresim_core::physiology::Humour::Choleric,
                "attribute-health-choleric",
                values.choleric.abs() * scale,
            ),
            (
                adventuresim_core::physiology::Humour::Melancholic,
                "attribute-health-melancholic",
                values.melancholic.abs() * scale,
            ),
        ]
    } else {
        Vec::new()
    }
}

/// Visible physical injury components, bounded by the region's known damage.
pub(super) struct PhysicalDamage {
    pub(super) cut: f32,
    pub(super) frostbite: f32,
    pub(super) fracture: f32,
    pub(super) blunt: f32,
}

impl PhysicalDamage {
    pub(super) fn new(
        physical_damage: f32,
        injury: Option<&crate::spacetimedb::LimbInjury>,
    ) -> Self {
        let cut = injury
            .map_or(0.0, |row| row.cut_damage)
            .min(physical_damage);
        let frostbite = injury
            .map_or(0.0, |row| row.frostbite_damage)
            .min((physical_damage - cut).max(0.0));
        let total_blunt = injury
            .map_or(physical_damage - cut - frostbite, |row| {
                row.bruise_damage.max(row.fracture_damage)
            })
            .min((physical_damage - cut - frostbite).max(0.0));
        let fracture = injury
            .map_or(0.0, |row| row.fracture_damage)
            .min(total_blunt);
        let blunt = (total_blunt - fracture).max(0.0);
        Self {
            cut,
            frostbite,
            fracture,
            blunt,
        }
    }
}
