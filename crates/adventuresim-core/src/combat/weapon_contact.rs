//! One continuous contact-concentration scale, independent of handling accuracy.
use super::WeaponContactParameters;
use crate::equipment::PlayerEquipment;

#[cfg(test)]
mod tests;

impl WeaponContactParameters {
    /// Calibrated working-section concentration. Multiple heads provide
    /// alternative contacts: choose the most concentrated usable surface,
    /// never add their scores. Furniture cannot make a weapon pointier.
    pub fn precision_for_design(
        self,
        design: &adventuresim_weapon_model::WeaponDesign,
    ) -> ContactPrecision {
        use adventuresim_weapon_model::{
            ComponentRole,
            recipe::{Metres, Shape},
        };
        let mm = |value: Metres| (value.get() * 1000.0) as f32;
        let point = |width: Metres, depth: Metres| {
            self.point_reference_area_mm2 / (mm(width).powi(2) + mm(depth).powi(2))
        };
        let mut precision: f32 = 0.0;
        for component in &design.recipe.components {
            if component.role != Some(ComponentRole::Head) {
                continue;
            }
            let candidate = match &component.shape {
                Shape::LoftedBlade(p) => point(p.width, p.thickness),
                Shape::Blade(p) => point(p.width, p.thickness),
                Shape::SectionBlade(p) => point(p.width, p.thickness),
                Shape::DiamondBlade(p) => point(p.width, p.thickness),
                Shape::Spear(p) => point(p.width, p.thickness),
                Shape::Partisan(p) => point(p.width, p.thickness),
                Shape::Glaive(p) => point(p.width, p.thickness),
                Shape::Bill(p) => point(p.width, p.thickness),
                Shape::Fork(p) => point(p.working_tine_width(), p.thickness),
                Shape::Beak(p) => point(p.working_root(), p.working_thickness()),
                Shape::Pick(p) => point(p.working_diameter(), p.working_diameter()),
                Shape::FacetedBeak(p) => point(p.root, p.thickness),
                Shape::Axe(p) => self.edge_reference_area_mm2 / (mm(p.height) * mm(p.thickness)),
                Shape::Mace(p) => {
                    self.broad_reference_area_mm2 / (mm(p.length) * 2.0 * mm(p.cusp_radius))
                }
                Shape::Hammer(p) => {
                    self.broad_reference_area_mm2
                        / (mm(p.face) * mm(p.face_thickness.unwrap_or(p.thickness)))
                }
                Shape::Shaft(p) => {
                    self.broad_reference_area_mm2 / (mm(p.length) * 2.0 * mm(p.radius))
                }
                _ => continue,
            };
            precision = precision.max(candidate);
        }
        if precision == 0.0 {
            if let Some(p) = &design.recipe.shaft {
                precision = self.broad_reference_area_mm2 / (mm(p.length) * 2.0 * mm(p.radius));
            }
            for component in &design.recipe.components {
                if matches!(
                    component.role,
                    Some(ComponentRole::Structure | ComponentRole::Grip)
                ) && let Shape::Shaft(p) = &component.shape
                {
                    precision = self.broad_reference_area_mm2 / (mm(p.length) * 2.0 * mm(p.radius));
                    break;
                }
            }
        }
        ContactPrecision::new(precision)
    }

    /// Wrist-angle error grows with grip-to-tip distance. Imbalance increases
    /// that angular error; neither contact precision nor weapon identity enters.
    pub fn handling_accuracy(self, equipment: &impl PlayerEquipment) -> f32 {
        let lever = equipment.weapon_grip_to_tip().max(0.0) / self.reference_control_length_metres;
        let angular_error = 1.0 + equipment.weapon_balance().max(0.0);
        (1.0 + (lever * angular_error).powi(2)).sqrt().recip()
    }
}

/// Validated dimensionless concentration, not a weapon damage-type tag.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactPrecision(f32);

impl ContactPrecision {
    pub fn new(value: f32) -> Self {
        assert!(
            value.is_finite() && value >= 0.0,
            "invalid contact precision"
        );
        Self(value)
    }

    pub const fn value(self) -> f32 {
        self.0
    }

    /// The same bounded concentration controls concentrated injury and gap access.
    pub fn concentrated_fraction(self) -> f32 {
        self.0 / (1.0 + self.0)
    }

    pub fn resistance(self, resistance: f32) -> f32 {
        if self.0 > 0.0 {
            resistance.max(0.0) / self.0
        } else {
            f32::INFINITY
        }
    }
}
