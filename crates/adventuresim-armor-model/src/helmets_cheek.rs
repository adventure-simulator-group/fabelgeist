//! Rounded burgonet cheek plates with a fan rising from the lower front.
use super::{BurgonetDesign, geometry::Surface};
use crate::{GenerateError, PartMesh};
use std::f32::consts::TAU;

pub(super) fn generate(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &BurgonetDesign,
) -> Result<PartMesh, GenerateError> {
    const RADIAL_ROWS: usize = 16;
    let width = radii[2] * 0.38 * d.cheek_width.unit();
    let height = half_height * 0.46 * d.cheek_depth.unit();
    let center = [brow - half_height * 0.40, radii[2] * 0.10];
    let root = [center[0] - height * 0.65, center[1] + width * 0.55];
    let mut columns = d.cheek_fluting.as_ref().map_or_else(
        || (0..=48).map(|i| i as f32 / 48.0).collect(),
        |p| p.columns(48),
    );
    columns.pop();
    let mut surface = Surface::default();
    let point = |y: f32, z: f32| {
        let depth = (z / radii[2]).clamp(-0.98, 0.98);
        let lower = ((brow - y) / half_height).clamp(0.0, 1.0);
        [
            radii[0] * (1.0 - (1.0 - d.cheek_taper.unit()) * lower) * (1.0 - depth * depth).sqrt(),
            y,
            z,
        ]
    };
    let pole = surface.vertex(point(root[0], root[1]));
    // The cheek laps outside the rear skull sheet by one gauge of air.
    let separation = d.fit.wall_thickness.metres() * 2.0;
    surface.relief[pole as usize] = separation;
    let mut previous: Vec<u32> = Vec::new();
    for row in 1..=RADIAL_ROWS {
        let v = row as f32 / RADIAL_ROWS as f32;
        let mut ring = Vec::new();
        for u in &columns {
            let mapped = d
                .cheek_fluting
                .as_ref()
                .map_or(*u, |pattern| pattern.fan_coordinate(*u, v));
            let angle = (mapped - 0.5) * TAU;
            let tab = d.chin_tab.metres() * (angle + 3.0 * TAU / 8.0).cos().max(0.0).powi(10);
            let edge = [
                center[0] + height * angle.cos() - tab * 0.8,
                center[1] - width * angle.sin() + tab * 0.6,
            ];
            let p = [
                root[0] + (edge[0] - root[0]) * v,
                root[1] + (edge[1] - root[1]) * v,
            ];
            let id = surface.vertex(point(p[0], p[1]));
            surface.relief[id as usize] = separation
                + d.cheek_fluting
                    .as_ref()
                    .map_or(0.0, |pattern| pattern.relief(*u, v));
            ring.push(id);
        }
        if previous.is_empty() {
            for i in 0..ring.len() {
                surface
                    .indices
                    .extend([pole, ring[(i + 1) % ring.len()], ring[i]]);
            }
        } else {
            for i in 0..ring.len() {
                let next = (i + 1) % ring.len();
                surface.indices.extend([
                    previous[i],
                    previous[next],
                    ring[next],
                    previous[i],
                    ring[next],
                    ring[i],
                ]);
            }
        }
        previous = ring;
    }
    surface.shell(d.fit.wall_thickness.metres(), crate::ShellExtrusion::Normal)
}
