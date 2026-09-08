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
        use adventuresim_weapon_model::{ComponentRole, ComponentShape};
        let point = |width: adventuresim_weapon_model::Millimeters,
                     depth: adventuresim_weapon_model::Millimeters| {
            self.point_reference_area_mm2 / ((width.0 as f32).powi(2) + (depth.0 as f32).powi(2))
        };
        let mut precision: f32 = 0.0;
        for component in &design.components {
            if component.role != ComponentRole::Head {
                continue;
            }
            let candidate = match &component.shape {
                ComponentShape::Blade(value) => point(value.width, value.thickness),
                ComponentShape::Spear(value) => point(value.width, value.thickness),
                ComponentShape::Partisan(value) => point(value.width, value.thickness),
                ComponentShape::Glaive(value) => point(value.width, value.thickness),
                ComponentShape::Bill(value) => point(value.width, value.thickness),
                ComponentShape::Fork(value) => point(value.tine_width, value.thickness),
                ComponentShape::CurvedBeak(value) => point(value.root_section, value.thickness),
                ComponentShape::FacetedBeak(value) => point(value.root, value.thickness),
                ComponentShape::Axe(value) => {
                    self.edge_reference_area_mm2
                        / (value.height.0 as f32 * value.thickness.0 as f32)
                }
                ComponentShape::Mace(value) => {
                    self.broad_reference_area_mm2
                        / (value.length.0 as f32 * 2.0 * value.cusp_radius.0 as f32)
                }
                ComponentShape::GothicMace(value) => {
                    self.broad_reference_area_mm2
                        / (value.length.0 as f32 * 2.0 * value.cusp_radius.0 as f32)
                }
                ComponentShape::HammerPoll(value) => {
                    self.broad_reference_area_mm2
                        / (value.face.0 as f32 * value.face_thickness.0 as f32)
                }
                ComponentShape::Cylinder(value) => {
                    self.broad_reference_area_mm2
                        / (value.length.0 as f32 * 2.0 * value.radius.0 as f32)
                }
                _ => continue,
            };
            precision = precision.max(candidate);
        }
        // A bare staff has no separate head; its shaft is its broad contact.
        if precision == 0.0 {
            for component in &design.components {
                if matches!(
                    component.role,
                    ComponentRole::Structure | ComponentRole::Grip
                ) && let ComponentShape::Cylinder(value) = &component.shape
                {
                    let diameter = 2.0 * value.radius.0 as f32;
                    precision = self.broad_reference_area_mm2 / (value.length.0 as f32 * diameter);
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
