//! Crown loops and strapped timber headstock carried by two journal bearings.
use super::*;

pub(super) struct BellHanging {
    pub bell_top: Vec3,
    pub axis_height: f32,
    pub bearing_half_span: f32,
    pub rail_length: f32,
    pub rail_depth: f32,
    pub headstock_height: f32,
}

pub(super) struct HangingPart {
    pub centre: Vec3,
    pub size: Vec3,
    pub role: SolidRole,
}

impl BellHanging {
    pub fn parts(&self) -> Vec<HangingPart> {
        let mut result = self.frame_parts();
        result.extend(self.crown_parts());
        // Emit the load path from the bearings down so the modest-church
        // builder can derive support parents from already resolved contacts.
        result.sort_by(|a, b| {
            let stage = |role| match role {
                SolidRole::ChurchBellFrame | SolidRole::ChurchBellHeadstock => 0,
                SolidRole::ChurchBellFitting
                | SolidRole::ChurchBellAxle
                | SolidRole::ChurchBellBearing => 1,
                _ => 2,
            };
            stage(a.role).cmp(&stage(b.role)).then_with(|| {
                if matches!(
                    a.role,
                    SolidRole::ChurchBellFrame | SolidRole::ChurchBellHeadstock
                ) {
                    std::cmp::Ordering::Equal
                } else {
                    b.centre.y.total_cmp(&a.centre.y)
                }
            })
        });
        result
    }

    fn frame_parts(&self) -> Vec<HangingPart> {
        let mut result = Vec::new();
        let mut part = |offset: Vec3, size: Vec3, role| {
            result.push(HangingPart {
                centre: self.bell_top + offset,
                size,
                role,
            });
        };
        let axis = self.axis_height - self.bell_top.y;
        let journal_radius = self.headstock_height * 0.16;
        let headstock_depth = self.headstock_height;
        let headstock_half_span = self.bearing_half_span - self.rail_depth * 0.55;
        // Fixed rails bear at both ends. Their clear middle leaves the bell's
        // swing plane unobstructed; the headstock and journals rotate about X.
        for side in [-1.0, 1.0] {
            part(
                Vec3::new(
                    side * self.bearing_half_span,
                    axis - journal_radius - self.rail_depth * 0.5,
                    0.0,
                ),
                Vec3::new(self.rail_depth, self.rail_depth, self.rail_length),
                SolidRole::ChurchBellFrame,
            );
        }
        part(
            Vec3::Y * axis,
            Vec3::new(
                headstock_half_span * 2.0,
                self.headstock_height,
                headstock_depth,
            ),
            SolidRole::ChurchBellHeadstock,
        );
        for side in [-1.0, 1.0] {
            let journal_length =
                self.bearing_half_span + self.rail_depth * 0.35 - headstock_half_span;
            part(
                Vec3::new(
                    side * (headstock_half_span + journal_length * 0.5),
                    axis,
                    0.0,
                ),
                Vec3::new(journal_length, journal_radius * 2.0, journal_radius * 2.0),
                SolidRole::ChurchBellAxle,
            );
            // Forged bearing cheeks flank the journal without blocking rotation.
            for z in [-1.0, 1.0] {
                part(
                    Vec3::new(
                        side * self.bearing_half_span,
                        axis - journal_radius * 0.25,
                        z * journal_radius * 1.6,
                    ),
                    Vec3::new(
                        self.rail_depth * 0.65,
                        journal_radius * 1.5,
                        journal_radius * 1.2,
                    ),
                    SolidRole::ChurchBellBearing,
                );
            }
        }
        result
    }

    fn crown_parts(&self) -> Vec<HangingPart> {
        let mut result = Vec::new();
        let mut part = |offset: Vec3, size: Vec3, role| {
            result.push(HangingPart {
                centre: self.bell_top + offset,
                size,
                role,
            });
        };
        let axis = self.axis_height - self.bell_top.y;
        let headstock_depth = self.headstock_height;
        let loop_height = (axis - self.headstock_height * 0.5) * 0.78;
        let loop_width = self.headstock_height * 0.38;
        let metal = self.headstock_height * 0.10;
        let pin_height = loop_height - metal * 1.5;
        // Two cast crown loops are open between their legs. The transverse pin
        // passes through both loops and the two straps wrapping the headstock.
        for z in [-1.0, 1.0] {
            for x in [-1.0, 1.0] {
                part(
                    Vec3::new(
                        x * loop_width * 0.5,
                        loop_height * 0.5 - metal * 0.25,
                        z * metal * 1.2,
                    ),
                    Vec3::new(metal, loop_height + metal * 0.5, metal),
                    SolidRole::ChurchBellCrown,
                );
            }
            part(
                Vec3::new(0.0, loop_height - metal * 0.5, z * metal * 1.2),
                Vec3::new(loop_width + metal, metal, metal),
                SolidRole::ChurchBellCrown,
            );
        }
        part(
            Vec3::Y * pin_height,
            Vec3::new(metal * 1.5, metal * 1.5, headstock_depth + metal * 4.0),
            SolidRole::ChurchBellFitting,
        );
        let strap_top = axis + self.headstock_height * 0.5 + metal;
        for z in [-1.0, 1.0] {
            part(
                Vec3::new(
                    0.0,
                    (pin_height + strap_top) * 0.5,
                    z * (headstock_depth + metal) * 0.5,
                ),
                Vec3::new(metal * 2.5, strap_top - pin_height + metal, metal),
                SolidRole::ChurchBellFitting,
            );
        }
        part(
            Vec3::new(0.0, strap_top - metal * 0.5, 0.0),
            Vec3::new(metal * 2.5, metal, headstock_depth + metal * 2.0),
            SolidRole::ChurchBellFitting,
        );
        result
    }
}
