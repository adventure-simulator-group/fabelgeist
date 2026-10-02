//! Hollow turned sockets whose terminal rim has repeated open notches.
use super::*;
use crate::recipe::SocketCrenellations;
use std::f64::consts::TAU;

impl Solid {
    pub(crate) fn crenellated_socket(
        profile: &[PlanarPoint],
        inner: &[f64],
        crenels: &SocketCrenellations,
        detail: Detail,
    ) -> Result<Self, String> {
        let RimSampling {
            rows,
            angles,
            crests,
            floor_row,
        } = RimSampling::new(profile, inner, crenels, detail)?;
        construction_budget(angles.len() as f64 * rows.len() as f64 * 8.0)?;
        let point = |row: usize, sector: usize, inner: bool| {
            let angle = angles[sector % angles.len()];
            let radius = rows[row][if inner { 2 } else { 1 }];
            [radius * angle.cos(), rows[row][0], radius * angle.sin()]
        };
        let mut solid = Self::default();
        for sector in 0..angles.len() {
            let next = (sector + 1) % angles.len();
            let end = if crests[sector] {
                rows.len() - 1
            } else {
                floor_row
            };
            for row in 0..end {
                for inner in [false, true] {
                    let [a, b, c, d] = [
                        point(row, sector, inner),
                        point(row, next, inner),
                        point(row + 1, next, inner),
                        point(row + 1, sector, inner),
                    ];
                    if inner {
                        solid.quad(a, b, c, d, 2);
                    } else {
                        solid.quad(a, d, c, b, 1);
                    }
                }
            }
            for row in [0, end] {
                let [a, b, c, d] = [
                    point(row, sector, false),
                    point(row, next, false),
                    point(row, next, true),
                    point(row, sector, true),
                ];
                if row == 0 {
                    solid.quad(a, b, c, d, 0);
                } else {
                    solid.quad(a, d, c, b, 0);
                }
            }
            let previous = (sector + angles.len() - 1) % angles.len();
            if crests[sector] != crests[previous] {
                for row in floor_row..rows.len() - 1 {
                    let [a, b, c, d] = [
                        point(row, sector, false),
                        point(row, sector, true),
                        point(row + 1, sector, true),
                        point(row + 1, sector, false),
                    ];
                    if crests[sector] {
                        solid.quad(a, b, c, d, 3);
                    } else {
                        solid.quad(a, d, c, b, 3);
                    }
                }
            }
        }
        Ok(solid.positive())
    }
}

struct RimSampling {
    rows: Vec<[f64; 3]>,
    angles: Vec<f64>,
    crests: Vec<bool>,
    floor_row: usize,
}
impl RimSampling {
    fn new(
        profile: &[PlanarPoint],
        inner: &[f64],
        crenels: &SocketCrenellations,
        detail: Detail,
    ) -> Result<Self, String> {
        let mut rows: Vec<_> = profile
            .iter()
            .zip(inner)
            .map(|(p, &r)| [p[0], p[1], r])
            .collect();
        let top = rows.last().ok_or("socket needs a profile")?[0];
        let floor = top - crenels.depth.get();
        let before = rows
            .iter()
            .rposition(|p| p[0] <= floor)
            .ok_or("notches leave no socket base")?;
        let floor_row = if rows[before][0] == floor {
            before
        } else {
            let a = rows[before];
            let b = rows[before + 1];
            let t = (floor - a[0]) / (b[0] - a[0]);
            rows.insert(
                before + 1,
                [floor, a[1] + t * (b[1] - a[1]), a[2] + t * (b[2] - a[2])],
            );
            before + 1
        };
        let teeth = crenels.count.0 as usize;
        let fraction = crenels.tooth_fraction.get();
        let radius = rows.iter().map(|p| p[1]).fold(0.0, f64::max);
        let around = detail.radial(radius, 32).max(teeth * 8);
        let tooth_steps = ((around as f64 / teeth as f64 * fraction).ceil() as usize).max(2);
        let notch_steps =
            ((around as f64 / teeth as f64 * (1.0 - fraction)).ceil() as usize).max(2);
        let mut angles = Vec::new();
        let mut crests = Vec::new();
        for tooth in 0..teeth {
            for step in 0..tooth_steps {
                angles.push(
                    TAU * (tooth as f64 + fraction * step as f64 / tooth_steps as f64)
                        / teeth as f64,
                );
                crests.push(true);
            }
            for step in 0..notch_steps {
                angles.push(
                    TAU * (tooth as f64
                        + fraction
                        + (1.0 - fraction) * step as f64 / notch_steps as f64)
                        / teeth as f64,
                );
                crests.push(false);
            }
        }
        Ok(Self {
            rows,
            angles,
            crests,
            floor_row,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::{Count, Metres, Ratio};

    #[test]
    fn notched_socket_is_closed_and_preserves_its_open_bore_and_material_volume() {
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            for count in [3, 5, 8] {
                let rim = SocketCrenellations {
                    count: Count(count),
                    depth: Metres::new(0.007).unwrap(),
                    tooth_fraction: Ratio::new(0.4).unwrap(),
                };
                let solid = Solid::crenellated_socket(
                    &[[0.0, 0.012], [0.02, 0.012]],
                    &[0.01; 2],
                    &rim,
                    detail,
                )
                .unwrap();
                let expected = std::f64::consts::PI
                    * (0.012_f64.powi(2) - 0.01_f64.powi(2))
                    * (0.02 - 0.007 * 0.6);
                assert!((solid.volume() / expected - 1.0).abs() < 0.015);
                assert!(solid.positions.iter().all(|p| p[0].hypot(p[2]) > 0.00999));
                assert!(solid.positions.iter().any(|p| (p[1] - 0.013).abs() < 1e-12));
                assert!(solid.positions.iter().any(|p| p[1] == 0.02));
                crate::construction::surface_tests::closed(solid);
            }
        }
    }
}
