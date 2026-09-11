//! Skull, peak, hinged cheek defenses and a separate lower neck guard.
use super::geometry::{AROUND, Surface};
use crate::{GenerateError, PartMesh};
use std::f32::consts::{FRAC_PI_3, TAU};
const SKIRT_ROWS: usize = 8;

pub(super) fn generate(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &super::BurgonetDesign,
) -> Result<PartMesh, GenerateError> {
    let mut skull = Surface::default();
    let rim = skull.styled_dome(radii, brow, &d.crown, d.comb_height.metres());
    const NAPE_START: usize = AROUND * 7 / 24;
    let mut previous = rim[NAPE_START..=AROUND - NAPE_START].to_vec();
    let join = 1.0 - d.neck_guard_fraction.unit();
    let gauge = d.fit.wall_thickness.metres();
    for row in 1..=SKIRT_ROWS {
        let v = row as f32 / SKIRT_ROWS as f32;
        let ring = (NAPE_START..=AROUND - NAPE_START)
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * TAU;
                let mut p = nape_point(radii, brow, half_height, d, join * v, angle);
                p[0] += gauge * 2.0 * v * v * angle.sin();
                p[2] += gauge * 2.0 * v * v * angle.cos();
                skull.vertex(p)
            })
            .collect::<Vec<_>>();
        skull.connect(&previous, &ring, false);
        previous = ring;
    }
    const PEAK_HALF_COLUMNS: usize = AROUND / 6;
    let mut previous = (0..=2 * PEAK_HALF_COLUMNS)
        .map(|i| rim[(AROUND - PEAK_HALF_COLUMNS + i) % AROUND])
        .collect::<Vec<_>>();
    for row in 1..=4 {
        let t = row as f32 / 4.0;
        let next = (0..=2 * PEAK_HALF_COLUMNS)
            .map(|i| {
                let angle =
                    -FRAC_PI_3 + 2.0 * FRAC_PI_3 * i as f32 / (2 * PEAK_HALF_COLUMNS) as f32;
                let reach = d.peak_length.metres() * angle.cos() * t;
                skull.vertex([
                    (radii[0] + reach) * angle.sin(),
                    brow - d.peak_drop.metres() * angle.cos() * t,
                    (radii[2] + reach) * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        skull.connect(&previous, &next, false);
        previous = next;
    }
    let mut mesh = skull.shell(d.fit.wall_thickness.metres(), crate::ShellExtrusion::Normal)?;
    let mut guard = Surface::default();
    let mut previous = Vec::new();
    const NECK_LAP_FRACTION: f32 = 0.05;
    for row in 0..=SKIRT_ROWS {
        let t = join - NECK_LAP_FRACTION
            + (1.0 - join + NECK_LAP_FRACTION) * row as f32 / SKIRT_ROWS as f32;
        let ring = (NAPE_START..=AROUND - NAPE_START)
            .map(|i| {
                guard.vertex(nape_point(
                    radii,
                    brow,
                    half_height,
                    d,
                    t,
                    i as f32 / AROUND as f32 * TAU,
                ))
            })
            .collect::<Vec<_>>();
        if row > 0 {
            guard.connect(&previous, &ring, false);
        }
        previous = ring;
    }
    mesh.append(PartMesh::from_relief_surface(
        guard.positions,
        guard.indices,
        gauge,
        crate::BoundaryNormals::Smooth,
        crate::ShellExtrusion::Radial {
            origin: [0.0; 3],
            axis: [0.0, 1.0, 0.0],
        },
        None,
    )?);
    for side in [-1.0, 1.0] {
        let cheek = super::cheek::generate(radii, brow, half_height, d)?;
        mesh.append(if side < 0.0 { mirror(cheek) } else { cheek });
    }
    Ok(mesh)
}

fn mirror(mesh: PartMesh) -> PartMesh {
    mesh.transformed(&crate::PartFrame {
        origin: [0.0; 3],
        axes: [[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents: [1.0; 3],
    })
}

fn nape_point(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &super::BurgonetDesign,
    t: f32,
    angle: f32,
) -> [f32; 3] {
    const FLARE_START: f32 = 0.75;
    let taper = 1.0 - (1.0 - d.nape_taper.unit()) * t * t;
    let neck = (t / FLARE_START).min(1.0);
    let recession = d.nape_recession.metres() * neck * neck * (3.0 - 2.0 * neck);
    let flare = d.neck_flare.metres() * ((t - FLARE_START) / (1.0 - FLARE_START)).max(0.0);
    let rear = (-angle.cos()).max(0.0);
    let depth = half_height * d.nape_depth.unit() * (0.65 + 0.35 * rear * rear);
    let back = angle.cos() * (1.0 - t * t) - rear.powf(0.65) * t * t;
    [
        (radii[0] * taper + flare * 0.4) * angle.sin(),
        brow - depth * t,
        (radii[2] - recession + flare) * back,
    ]
}
