//! Resample the fitted torso chart into separate overlapping closed courses.
use super::*;
use crate::BreastplateConstruction;

const COURSE_ROWS: usize = 8;
const UPPER_PLATE_ROWS: usize = 24;

pub(super) fn articulate(
    source: MidMesh,
    rear: bool,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
    eligible: &[usize],
) -> Result<MidMesh, GenerateError> {
    let BreastplateConstruction::Anime(anime) = &design.construction else {
        return Ok(source);
    };
    let samples = carrier_samples(&source, wearer, eligible);
    let normals = source
        .extrusion_normals
        .clone()
        .map(Ok)
        .unwrap_or_else(|| vertex_normals(&source.positions, &source.faces))?;
    let columns = source.main_columns;
    let row_order = (V_SAMPLES..V_SAMPLES + SKIRT_SAMPLES - 1)
        .rev()
        .chain(0..V_SAMPLES)
        .collect::<Vec<_>>();
    let slope = if rear {
        anime.rear_chevron_slope.unit()
    } else {
        anime.chevron_slope.unit()
    };
    let level = |index: usize| {
        let p = local(source.positions[index], wearer.frame);
        p[1] - slope * (p[0] - wearer.lateral_origin).abs()
    };
    let floor = (0..columns)
        .map(|c| level(row_order[0] * columns + c))
        .fold(f32::NEG_INFINITY, f32::max);
    let ceiling = (0..columns)
        .map(|c| level((V_SAMPLES - 1) * columns + c))
        .fold(f32::INFINITY, f32::min);
    let course_top = floor + (ceiling - floor) * anime.articulated_height.unit();
    let count = usize::from(anime.lame_count);
    let height = (course_top - floor) / count as f32;
    if height <= anime.overlap.metres() * 2.0 {
        return Err(GenerateError::InvalidSurface);
    }
    let mut mesh = MidMesh {
        main_columns: columns,
        ..MidMesh::default()
    };
    let (mut extrusion, mut attached) = (Vec::new(), Vec::new());
    for course in 0..=count {
        let lower = floor + height * course as f32 - anime.overlap.metres();
        let upper = floor + height * (course + 1) as f32;
        let rows = if course == count {
            UPPER_PLATE_ROWS
        } else {
            COURSE_ROWS
        };
        let mut ids = Vec::new();
        for row in 0..=rows {
            let t = row as f32 / rows as f32;
            let mut ring = Vec::new();
            for c in 0..columns {
                let low = if course == 0 { floor } else { lower };
                let high = if course == count {
                    level((V_SAMPLES - 1) * columns + c)
                } else {
                    upper
                };
                let q = low + (high - low) * t;
                let (a, b, blend) = sample_column(c, columns, &row_order, q, &level);
                let normal = normalized(add(
                    scale(normals[a], 1.0 - blend),
                    scale(normals[b], blend),
                ))?;
                let lift = if course == 0 {
                    0.0
                } else {
                    anime.lap_lift.metres() * (1.0 - t)
                };
                ring.push(mesh.positions.len() as u32);
                mesh.positions.push(add(
                    add(
                        scale(source.positions[a], 1.0 - blend),
                        scale(source.positions[b], blend),
                    ),
                    scale(normal, lift),
                ));
                extrusion.push(normal);
                attached.push(blend_carrier_samples(&samples[a], &samples[b], blend));
                if !source.medial_crease.is_empty() {
                    mesh.medial_crease.push(source.medial_crease[a]);
                    mesh.crease_right.push(source.crease_right[a]);
                }
            }
            ids.push(ring);
        }
        append_grid_faces(&mut mesh, &ids, rear);
    }
    mesh.extrusion_normals = Some(extrusion);
    mesh.morph_samples = Some(attached);
    Ok(mesh)
}

fn blend_carrier_samples(a: &MorphSample, b: &MorphSample, blend: f32) -> MorphSample {
    MorphSample {
        endpoints: [
            a.endpoints[0],
            a.endpoints[1],
            b.endpoints[0],
            b.endpoints[1],
        ],
        weights: [
            a.weights[0] * (1.0 - blend),
            a.weights[1] * (1.0 - blend),
            b.weights[0] * blend,
            b.weights[1] * blend,
        ],
    }
}

fn sample_column(
    column: usize,
    columns: usize,
    rows: &[usize],
    target: f32,
    level: &impl Fn(usize) -> f32,
) -> (usize, usize, f32) {
    for pair in rows.windows(2) {
        let a = pair[0] * columns + column;
        let b = pair[1] * columns + column;
        let low = level(a);
        let high = level(b);
        if target <= high {
            return (a, b, ((target - low) / (high - low)).clamp(0.0, 1.0));
        }
    }
    let end = rows[rows.len() - 1] * columns + column;
    (end, end, 0.0)
}
