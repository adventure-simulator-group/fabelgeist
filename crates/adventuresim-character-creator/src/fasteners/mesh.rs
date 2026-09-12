//! Closed leather bands, buckle frames, tongues, and terminal rivet heads.
use super::{StrapDesign, section::ClosureSection};
use adventuresim_armor_model::PartMesh;
use anyhow::Result;
use std::f32::consts::PI;

const BAND_SEGMENTS: usize = 192;
const SEATING_GAP_M: f32 = 0.0025;
const BAR_GAUGE_M: f32 = 0.0025;

struct StripSample {
    point: [f32; 2],
    normal: [f32; 2],
}

impl StripSample {
    fn new(point: [f32; 2], tangent: [f32; 2]) -> Self {
        let length = tangent[0].hypot(tangent[1]);
        Self {
            point,
            normal: [-tangent[1] / length, tangent[0] / length],
        }
    }
}

pub(super) fn closure(
    section: &ClosureSection,
    height: f32,
    design: &StrapDesign,
) -> Result<(PartMesh, PartMesh)> {
    let width = design.width.metres();
    let thickness = design.thickness.metres();
    let start = design.start_angle.radians();
    let end = design.end_angle.radians();
    let buckle_angle = start + (end - start) * design.buckle_position.unit();
    let buckle_radius = section.radius(buckle_angle)?;
    let buckle_offset = SEATING_GAP_M * 2.0 + thickness * 2.0;
    let fixed_end = buckle_angle - width * 0.52 / buckle_radius;
    let free_tip = buckle_angle - width * 1.3 / buckle_radius;
    anyhow::ensure!(
        fixed_end > start && end - buckle_angle > width / buckle_radius,
        "strap arc is too short for its buckle and return fold on this wearer"
    );
    let mut strap = fixed_loop(
        section,
        height,
        width,
        thickness,
        start,
        fixed_end,
        buckle_offset,
    )?;
    strap.append(band(
        section,
        height,
        width,
        thickness,
        free_tip,
        end,
        |angle| {
            let distance = (angle - buckle_angle) * buckle_radius;
            let transition = ((distance - width * 0.3) / width).clamp(0.0, 1.0);
            let fold_end =
                -width * 0.52 + BAR_GAUGE_M * 0.5 + thickness * 1.5 + SEATING_GAP_M * 2.0;
            let threaded = ((distance - fold_end) / (width * 0.24)).clamp(0.0, 1.0);
            let over_fold = (thickness * 2.5 + BAR_GAUGE_M + SEATING_GAP_M * 2.0)
                * (1.0 - threaded * threaded * (3.0 - 2.0 * threaded));
            over_fold
                + (thickness + SEATING_GAP_M)
                    * (1.0 - transition * transition * (3.0 - 2.0 * transition))
        },
    )?);
    let radius = buckle_radius + buckle_offset;
    let mut hardware = buckle(width, radius, height, buckle_angle);
    // Keep mounted metal parallel to the actual support rather than a circular
    // approximation that can put one edge through an eccentric limb section.
    conform_hardware(&mut hardware, section, buckle_radius)?;
    for angle in [start + 0.04, end - 0.04] {
        let radius = section.radius(angle)? + SEATING_GAP_M + thickness;
        let mut head = rivet(angle, radius, height, width * 0.18);
        conform_hardware(&mut head, section, section.radius(angle)?)?;
        hardware.append(head);
    }
    Ok((strap, hardware))
}

fn conform_hardware(mesh: &mut PartMesh, section: &ClosureSection, reference: f32) -> Result<()> {
    for p in &mut mesh.positions {
        let angle = p[0].atan2(p[2]);
        let radius = p[0].hypot(p[2]) + section.radius(angle)? - reference;
        *p = radial_point(angle, radius, p[1]);
    }
    Ok(())
}

/// One continuous strip bends around the buckle's root bar and folds back.
fn fixed_loop(
    section: &ClosureSection,
    height: f32,
    width: f32,
    thickness: f32,
    start: f32,
    end: f32,
    buckle_offset: f32,
) -> Result<PartMesh> {
    const FOLD_SEGMENTS: usize = 16;
    let radius = section.radius(end)?;
    let bend_radius = BAR_GAUGE_M * 0.5 + thickness + SEATING_GAP_M;
    let center = buckle_offset + BAR_GAUGE_M * 0.5;
    let bottom = center - bend_radius;
    let base = SEATING_GAP_M + thickness * 0.5;
    let mut path = Vec::new();
    for i in 0..=BAND_SEGMENTS {
        let t = i as f32 / BAND_SEGMENTS as f32;
        let x = (start - end) * radius * (1.0 - t);
        let rise = ((x + width) / width).clamp(0.0, 1.0);
        let slope = (bottom - base) * 6.0 * rise * (1.0 - rise) / width;
        path.push(StripSample::new(
            [x, base + (bottom - base) * rise * rise * (3.0 - 2.0 * rise)],
            [1.0, slope],
        ));
    }
    for i in 1..=FOLD_SEGMENTS {
        let angle = PI * i as f32 / FOLD_SEGMENTS as f32;
        path.push(StripSample::new(
            [
                bend_radius * angle.sin(),
                center - bend_radius * angle.cos(),
            ],
            [angle.cos(), angle.sin()],
        ));
    }
    path.push(StripSample::new(
        [-width * 0.7, center + bend_radius],
        [-1.0, 0.0],
    ));
    let mut mesh = PartMesh::new();
    // Exact tangents retain wall separation where densely sampled straight
    // leather meets the coarser bend. A neighbor chord tilts that junction's
    // normal and can fold the outer wall back onto its preceding sample.
    for StripSample { point: p, normal } in &path {
        for (edge, wall) in [(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
            let angle = end + (p[0] + normal[0] * thickness * wall) / radius;
            mesh.positions.push(radial_point(
                angle,
                section.radius(angle)? + p[1] + normal[1] * thickness * wall,
                height + edge * width,
            ));
        }
    }
    close_strip(&mut mesh, path.len());
    Ok(mesh)
}

fn close_strip(mesh: &mut PartMesh, rows: usize) {
    for i in 1..rows {
        let a = (i as u32 - 1) * 4;
        for [p, q] in [[0, 1], [1, 3], [3, 2], [2, 0]] {
            quad(mesh, [a + p, a + q, a + q + 4, a + p + 4]);
        }
    }
    quad(mesh, [0, 2, 3, 1]);
    let end = (rows as u32 - 1) * 4;
    quad(mesh, [end, end + 1, end + 3, end + 2]);
    outward(mesh);
}

fn band(
    section: &ClosureSection,
    height: f32,
    width: f32,
    thickness: f32,
    start: f32,
    end: f32,
    lift: impl Fn(f32) -> f32,
) -> Result<PartMesh> {
    let mut strap = PartMesh::new();
    for i in 0..=BAND_SEGMENTS {
        let angle = start + (end - start) * i as f32 / BAND_SEGMENTS as f32;
        let radius = section.radius(angle)? + SEATING_GAP_M + lift(angle);
        for (edge, outside) in [(-0.5, 0.0), (0.5, 0.0), (-0.5, 1.0), (0.5, 1.0)] {
            strap.positions.push(radial_point(
                angle,
                radius + outside * thickness,
                height + edge * width,
            ));
        }
        if i > 0 {
            let a = (i as u32 - 1) * 4;
            for [p, q] in [[0, 1], [1, 3], [3, 2], [2, 0]] {
                quad(&mut strap, [a + p, a + q, a + q + 4, a + p + 4]);
            }
        }
    }
    quad(&mut strap, [0, 2, 3, 1]);
    let end = BAND_SEGMENTS as u32 * 4;
    quad(&mut strap, [end, end + 1, end + 3, end + 2]);
    outward(&mut strap);
    Ok(strap)
}

fn buckle(width: f32, radius: f32, height: f32, angle: f32) -> PartMesh {
    let mut mesh = buckle_shape(width);
    for p in &mut mesh.positions {
        *p = buckle_point(*p, radius, height, angle);
    }
    mesh
}

pub(super) fn buckle_shape(width: f32) -> PartMesh {
    let mut mesh = PartMesh::new();
    let half_width = width * 0.65;
    let half_length = width * 0.52;
    for depth in [0.0, BAR_GAUGE_M] {
        for inset in [0.0, BAR_GAUGE_M] {
            for [x, y] in [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]] {
                mesh.positions
                    .push([x * (half_length - inset), y * (half_width - inset), depth]);
            }
        }
    }
    for i in 0..4 {
        let j = (i + 1) % 4;
        quad(&mut mesh, [i, j, j + 4, i + 4]);
        quad(&mut mesh, [i + 8, i + 12, j + 12, j + 8]);
        quad(&mut mesh, [i, i + 8, j + 8, j]);
        quad(&mut mesh, [i + 4, j + 4, j + 12, i + 12]);
    }
    outward(&mut mesh);
    // Tongue sits within the frame; its root and tip contact the frame.
    let mut tongue = cuboid([half_length, BAR_GAUGE_M * 0.35, BAR_GAUGE_M * 0.35]);
    for p in &mut tongue.positions {
        p[2] += BAR_GAUGE_M * 0.5;
    }
    mesh.append(tongue);
    mesh
}

fn buckle_point(p: [f32; 3], radius: f32, height: f32, angle: f32) -> [f32; 3] {
    let angle = angle + p[0] / radius;
    radial_point(angle, radius + p[2], height + p[1])
}

fn rivet(angle: f32, radius: f32, height: f32, size: f32) -> PartMesh {
    let mut mesh = rivet_shape(size);
    for p in &mut mesh.positions {
        *p = buckle_point(*p, radius, height, angle);
    }
    mesh
}

pub(super) fn rivet_shape(size: f32) -> PartMesh {
    const SIDES: usize = 12;
    const HEAD_HEIGHT_M: f32 = 0.0015;
    let mut mesh = PartMesh::new();
    for (depth, scale) in [(0.0, 1.0), (HEAD_HEIGHT_M, 0.75)] {
        for i in 0..SIDES {
            let phi = 2.0 * PI * i as f32 / SIDES as f32;
            mesh.positions
                .push([phi.cos() * size * scale, phi.sin() * size * scale, depth]);
        }
    }
    for i in 0..SIDES as u32 {
        let j = (i + 1) % SIDES as u32;
        quad(&mut mesh, [i, j, j + SIDES as u32, i + SIDES as u32]);
    }
    for i in 1..SIDES as u32 - 1 {
        mesh.indices.extend([
            0,
            i + 1,
            i,
            SIDES as u32,
            SIDES as u32 + i,
            SIDES as u32 + i + 1,
        ]);
    }
    outward(&mut mesh);
    mesh
}

fn radial_point(angle: f32, radius: f32, height: f32) -> [f32; 3] {
    [angle.sin() * radius, height, angle.cos() * radius]
}

pub(super) fn cuboid(half: [f32; 3]) -> PartMesh {
    let mut mesh = PartMesh::new();
    for [x, y, z] in [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
    ] {
        mesh.positions.push([x * half[0], y * half[1], z * half[2]]);
    }
    for face in [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [3, 7, 6, 2],
        [0, 4, 7, 3],
        [1, 2, 6, 5],
    ] {
        quad(&mut mesh, face);
    }
    mesh
}

fn quad(mesh: &mut PartMesh, [a, b, c, d]: [u32; 4]) {
    mesh.indices.extend([a, b, c, a, c, d]);
}

fn outward(mesh: &mut PartMesh) {
    let volume: f32 = mesh
        .indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|face| {
            let [a, b, c] = face.map(|i| mesh.positions[i as usize]);
            a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0])
        })
        .sum();
    if volume < 0.0 {
        for triangle in mesh.indices.as_chunks_mut::<3>().0 {
            triangle.swap(1, 2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::Millimeters;

    #[test]
    fn narrow_wrist_closures_keep_both_walls_advancing_into_the_buckle_bend() {
        let design = StrapDesign {
            width: Millimeters(12),
            thickness: Millimeters(2),
            ..StrapDesign::default()
        };
        let mut topology = None;
        for radius in [0.035, 0.045, 0.055] {
            let plate = cuboid([radius, 0.1, radius * 0.8]);
            let section = ClosureSection::new(
                &[],
                &[],
                &plate.positions,
                plate.indices.as_chunks::<3>().0,
                0.0,
                0.04,
                0.006,
            )
            .unwrap();
            let (leather, _) = closure(&section, 0.0, &design).unwrap();
            leather
                .normals()
                .expect("closed thickness must not collapse at the bend");
            for wall in [0, 2] {
                let angles = (0..=BAND_SEGMENTS)
                    .map(|row| leather.positions[row * 4 + wall])
                    .map(|p| p[0].atan2(p[2]))
                    .collect::<Vec<_>>();
                assert!(
                    angles.windows(2).all(|pair| pair[1] > pair[0]),
                    "straight strap wall folded backward at the buckle junction"
                );
            }
            if let Some(expected) = &topology {
                assert_eq!(
                    &leather.indices, expected,
                    "wearer changed closure connectivity"
                );
            }
            topology = Some(leather.indices);
        }
    }
}
