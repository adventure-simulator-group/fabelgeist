//! Paired tasset suspension: short leather hangers from the fauld to each panel.
use super::{finish, mesh};
use adventuresim_armor_model::{BoundaryNormals, Millimeters, PartMesh, ShellExtrusion};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuspensionDesign {
    pub width: Millimeters,
    pub thickness: Millimeters,
    pub count_per_panel: u8,
    pub fauld_inset: Millimeters,
    pub tasset_inset: Millimeters,
    pub leather_color: [u8; 3],
}

impl SuspensionDesign {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (12..=24).contains(&self.width.0) && (1..=3).contains(&self.thickness.0),
            "invalid hanger width or gauge"
        );
        ensure!(
            (1..=3).contains(&self.count_per_panel),
            "tassets require 1–3 hangers per panel"
        );
        ensure!(
            (40..=100).contains(&self.fauld_inset.0) && (20..=60).contains(&self.tasset_inset.0),
            "invalid suspension anchor inset"
        );
        Ok(())
    }

    pub fn generate(&self, tassets: &PartMesh, fauld: &PartMesh) -> Result<PartMesh> {
        self.validate()?;
        let extent = tassets
            .positions
            .iter()
            .map(|p| p[0].abs())
            .fold(0.0_f32, f32::max);
        let mut leather = PartMesh::new();
        let mut metal = PartMesh::new();
        for side in [-1.0, 1.0] {
            for i in 0..self.count_per_panel {
                let fraction = if self.count_per_panel == 1 {
                    0.55
                } else {
                    0.3 + 0.5 * f32::from(i) / f32::from(self.count_per_panel - 1)
                };
                let x = extent * fraction * side;
                let upper = vertical_extent(fauld, x)?.0 + self.fauld_inset.metres();
                let lower = (vertical_extent(tassets, x)?.1 - self.tasset_inset.metres())
                    .min(vertical_extent(fauld, x)?.0 - self.width.metres() * 2.0);
                ensure!(
                    upper - lower >= self.width.metres() * 2.0,
                    "suspension anchors are too close for the buckle"
                );
                let (strap, buckle) = hanger(self, tassets, fauld, x, upper, lower)?;
                leather.append(strap);
                metal.append(buckle);
            }
        }
        Ok(finish(leather, metal, self.leather_color))
    }
}

const HANGER_ROWS: usize = 64;
const SURFACE_GAP_M: f32 = 0.0055;

fn hanger(
    d: &SuspensionDesign,
    tassets: &PartMesh,
    fauld: &PartMesh,
    x: f32,
    upper: f32,
    lower: f32,
) -> Result<(PartMesh, PartMesh)> {
    let width = d.width.metres();
    let thickness = d.thickness.metres();
    let start = front_depth(fauld, x, upper).context("missing fauld suspension surface")?;
    let end = front_depth(tassets, x, lower).context("missing tasset buckle surface")?;
    let mut profile = Vec::new();
    for row in 0..=HANGER_ROWS {
        let t = row as f32 / HANGER_ROWS as f32;
        let y = upper + (lower - upper) * t;
        let z = [-0.5, 0.0, 0.5]
            .into_iter()
            .flat_map(|edge| {
                [
                    front_depth(fauld, x + edge * width, y),
                    front_depth(tassets, x + edge * width, y),
                ]
            })
            .flatten()
            .fold(start + (end - start) * t, f32::max);
        profile.push([y, z + SURFACE_GAP_M + thickness]);
    }
    // A tensioned strip bridges plate steps rather than reproducing every flute.
    for _ in 0..32 {
        let previous = profile.clone();
        for i in 1..HANGER_ROWS {
            profile[i][1] = profile[i][1].max((previous[i - 1][1] + previous[i + 1][1]) * 0.5);
        }
    }
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (i, &[y, z]) in profile.iter().enumerate() {
        for edge in [-0.5, 0.5] {
            vertices.push([x + edge * width, y, z]);
        }
        if i > 0 {
            let a = (i as u32 - 1) * 2;
            indices.extend([a, a + 2, a + 3, a, a + 3, a + 1]);
        }
    }
    let leather = PartMesh::from_surface(
        vertices,
        indices,
        thickness,
        BoundaryNormals::Separate,
        ShellExtrusion::Along {
            direction: [0.0, 0.0, 1.0],
        },
    )?;
    let buckle_y = lower + width * 0.8;
    let mut buckle = mesh::buckle_shape(width);
    let mut mounting_tab = mesh::cuboid([width * 0.28, width * 0.35, thickness * 0.25]);
    for p in &mut mounting_tab.positions {
        p[0] += width * 0.65;
        p[2] -= thickness * 0.25;
    }
    buckle.append(mounting_tab);
    for p in &mut buckle.positions {
        let y = buckle_y - p[0];
        let t = ((upper - y) / (upper - lower)).clamp(0.0, 1.0) * HANGER_ROWS as f32;
        let i = (t.floor() as usize).min(HANGER_ROWS - 1);
        let z = profile[i][1] + (profile[i + 1][1] - profile[i][1]) * (t - i as f32);
        *p = [x + p[1], y, z + p[2]];
    }
    for y in [upper - width * 0.35, buckle_y - width * 0.80] {
        let t = ((upper - y) / (upper - lower)).clamp(0.0, 1.0) * HANGER_ROWS as f32;
        let i = (t.floor() as usize).min(HANGER_ROWS - 1);
        let z = profile[i][1] + (profile[i + 1][1] - profile[i][1]) * (t - i as f32);
        let mut head = mesh::rivet_shape(width * 0.16);
        for p in &mut head.positions {
            *p = [x + p[0], y + p[1], z + p[2]];
        }
        buckle.append(head);
    }
    Ok((leather, buckle))
}

fn vertical_extent(mesh: &PartMesh, x: f32) -> Result<(f32, f32)> {
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for face in mesh.indices.as_chunks::<3>().0 {
        let p = face.map(|i| mesh.positions[i as usize]);
        for i in 0..3 {
            let a = p[i];
            let b = p[(i + 1) % 3];
            if (a[0] <= x && b[0] > x) || (b[0] <= x && a[0] > x) {
                let y = a[1] + (b[1] - a[1]) * (x - a[0]) / (b[0] - a[0]);
                lo = lo.min(y);
                hi = hi.max(y);
            }
        }
    }
    ensure!(
        lo.is_finite() && hi.is_finite(),
        "suspension anchor misses its plate"
    );
    Ok((lo, hi))
}

fn front_depth(mesh: &PartMesh, x: f32, y: f32) -> Option<f32> {
    mesh.indices
        .as_chunks::<3>()
        .0
        .iter()
        .filter_map(|face| {
            let [a, b, c] = face.map(|i| mesh.positions[i as usize]);
            let determinant = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
            if determinant.abs() < 1e-10 {
                return None;
            }
            let u = ((x - a[0]) * (c[1] - a[1]) - (y - a[1]) * (c[0] - a[0])) / determinant;
            let v = ((b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0])) / determinant;
            (u >= 0.0 && v >= 0.0 && u + v <= 1.0)
                .then_some(a[2] + u * (b[2] - a[2]) + v * (c[2] - a[2]))
        })
        .reduce(f32::max)
}
