//! Design-only constrained triangulation. Wearer fitting never changes its topology.
use crate::pierced_plate_domain::{MINIMUM_WEB_MM, PiercedDomain, edge_distance, rounded_slot};
use crate::{CloseHelmetDesign, GenerateError};

pub(super) const HALF_WIDTH_MM: f32 = 160.0;
pub(super) const HEIGHT_MM: f32 = 100.0;
pub(super) const SIGHT_CENTER_MM: f64 = 30.0;
const SAMPLE_SPACING_MM: f64 = 5.0;

pub(super) struct VisorDomain {
    pub points: Vec<[f64; 2]>,
    pub indices: Vec<u32>,
    pub relief: Vec<f32>,
}

impl VisorDomain {
    pub fn new(design: &CloseHelmetDesign) -> Result<Self, GenerateError> {
        let mut outer = vec![
            [-160.0, 12.0],
            [-145.0, 4.0],
            [-100.0, 0.0],
            [0.0, 0.0],
            [100.0, 0.0],
            [145.0, 4.0],
            [160.0, 12.0],
        ];
        // A shield-shaped lower edge rises continuously into the pivot wings.
        for column in (-16_i32..=16).rev() {
            let x = f64::from(column) * 10.0;
            outer.push([x, 100.0 - 70.0 * (x.abs() / 160.0).powf(1.5)]);
        }
        let holes = openings(design);
        let PiercedDomain { points, indices } =
            PiercedDomain::new(&outer, &holes, interior_samples(design))?;
        let relief = points
            .iter()
            .map(|p| {
                design.visor_fluting.as_ref().map_or(0.0, |pattern| {
                    let v = 1.0 - p[1] as f32 / HEIGHT_MM;
                    let u = pattern.unfan_coordinate((p[0] as f32 / HALF_WIDTH_MM + 1.0) * 0.5, v);
                    let distance = holes
                        .iter()
                        .map(|h| edge_distance(*p, h))
                        .fold(edge_distance(*p, &outer), f64::min);
                    let margin = (distance / MINIMUM_WEB_MM).clamp(0.0, 1.0) as f32;
                    let relief = pattern.relief(u, v) * margin * margin * (3.0 - 2.0 * margin);
                    // A raised flute beside a steep ledge must not cross a pierced return.
                    relief.min(distance as f32 * 0.001 * 0.35)
                })
            })
            .collect();
        Ok(Self {
            points,
            indices,
            relief,
        })
    }
}

fn interior_samples(design: &CloseHelmetDesign) -> Vec<[f64; 2]> {
    let mut points = Vec::new();
    let mut rows = (1..20)
        .map(|y| f64::from(y) * SAMPLE_SPACING_MM)
        .collect::<Vec<_>>();
    // The ocular ledge and narrow central bridge need their own sampling.
    rows.extend((20..=55).map(f64::from));
    if let Some(bellows) = design.bellows {
        rows.extend(
            bellows
                .rows()
                .map(|row| row * f64::from(HEIGHT_MM) / 1000.0),
        );
    }
    rows.sort_by(f64::total_cmp);
    rows.dedup_by(|a, b| (*a - *b).abs() < 0.05);
    for y in rows {
        let v = 1.0 - y as f32 / HEIGHT_MM;
        let mut columns = design.visor_fluting.as_ref().map_or_else(
            || {
                (-31..32)
                    .map(|x| f64::from(x) * SAMPLE_SPACING_MM)
                    .collect::<Vec<_>>()
            },
            |pattern| {
                pattern
                    .columns(64)
                    .into_iter()
                    .map(|u| f64::from((pattern.fan_coordinate(u, v) * 2.0 - 1.0) * HALF_WIDTH_MM))
                    .collect()
            },
        );
        if (20.0..=55.0).contains(&y) {
            columns.extend((-15..=15).map(f64::from));
            columns.sort_by(f64::total_cmp);
            columns.dedup_by(|a, b| (*a - *b).abs() < 0.1);
        }
        for x in columns {
            let p = [x, y];
            points.push(p);
        }
    }
    points
}

fn openings(d: &CloseHelmetDesign) -> Vec<Vec<[f64; 2]>> {
    let mut holes = Vec::new();
    let span = f64::from(d.sight_span.0);
    let bridge = f64::from(d.sight_bridge.0);
    if bridge == 0.0 {
        holes.push(rounded_slot(
            [0.0, SIGHT_CENTER_MM],
            span,
            f64::from(d.sight_gap.0),
            0.0,
            0.25,
        ));
    } else {
        for side in [-1.0, 1.0] {
            holes.push(rounded_slot(
                [side * (span + bridge) / 4.0, SIGHT_CENTER_MM],
                (span - bridge) / 2.0,
                f64::from(d.sight_gap.0),
                0.0,
                0.25,
            ));
        }
    }
    holes.extend(d.breaths.openings(f64::from(HEIGHT_MM)));
    holes
}
