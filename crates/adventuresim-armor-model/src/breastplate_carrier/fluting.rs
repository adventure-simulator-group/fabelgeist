//! Feature-aligned front sampling and relief on the fitted smooth carrier.
use super::*;

const FLUTE_PROFILE_SEGMENTS: usize = 8;
const CHART_MERGE_TOLERANCE: f32 = 5e-4;

pub(super) fn chart_columns(rear: bool, design: &BreastplateDesign) -> Vec<f32> {
    let mut columns: Vec<_> = (0..U_SAMPLES)
        .map(|i| -1.0 + 2.0 * i as f32 / (U_SAMPLES - 1) as f32)
        .collect();
    if let Some(pattern) = design.fluting.as_ref().filter(|_| !rear) {
        let pitch = 2.0 * pattern.spread.unit() / f32::from(pattern.count.0);
        let half_width = pitch * pattern.width.unit() * 0.5;
        for flute in 0..pattern.count.0 {
            let center = -pattern.spread.unit() + (f32::from(flute) + 0.5) * pitch;
            for sample in 0..=FLUTE_PROFILE_SEGMENTS {
                columns.push(
                    center
                        + half_width * (2.0 * sample as f32 / FLUTE_PROFILE_SEGMENTS as f32 - 1.0),
                );
            }
        }
        columns.sort_by(f32::total_cmp);
        columns.dedup_by(|a, b| (*a - *b).abs() < CHART_MERGE_TOLERANCE);
    }
    columns
}

/// Fan only the central chart, retaining both side edges and the top boundary.
pub(super) fn fan_coordinate(u: f32, t: f32, rear: bool, design: &BreastplateDesign) -> f32 {
    let Some(pattern) = design.fluting.as_ref().filter(|_| !rear) else {
        return u;
    };
    let span = pattern.spread.unit();
    let fan = pattern.lower_spread.unit() + (1.0 - pattern.lower_spread.unit()) * smoothstep(t);
    let absolute = u.abs();
    let mapped = if absolute <= span {
        absolute * fan
    } else {
        span * fan + (absolute - span) * (1.0 - span * fan) / (1.0 - span)
    };
    u.signum() * mapped
}

pub(super) fn apply_fluting(
    mesh: &mut MidMesh,
    design: &BreastplateDesign,
) -> Result<(), GenerateError> {
    let Some(pattern) = &design.fluting else {
        return Ok(());
    };
    let columns = chart_columns(false, design);
    let smooth_normals = mesh
        .extrusion_normals
        .clone()
        .ok_or(GenerateError::InvalidSurface)?;
    let pitch = 2.0 * pattern.spread.unit() / f32::from(pattern.count.0);
    let half_width = pitch * pattern.width.unit() * 0.5;
    for row in 0..V_SAMPLES {
        let t = row as f32 / (V_SAMPLES - 1) as f32;
        let fade = smoothstep((t - pattern.start.unit()) / pattern.fade.unit())
            * smoothstep((pattern.end.unit() - t) / pattern.fade.unit());
        for (column, u) in columns.iter().enumerate() {
            let slot = ((u + pattern.spread.unit()) / pitch).floor();
            if slot < 0.0 || slot >= f32::from(pattern.count.0) {
                continue;
            }
            let center = -pattern.spread.unit() + (slot + 0.5) * pitch;
            let distance = (u - center).abs() / half_width;
            if distance >= 1.0 {
                continue;
            }
            let relief = (1.0 + (std::f32::consts::PI * distance).cos()) * 0.5;
            let index = row * columns.len() + column;
            mesh.positions[index] = add(
                mesh.positions[index],
                scale(
                    smooth_normals[index],
                    pattern.depth.metres() * relief * fade,
                ),
            );
        }
    }
    // Offset both corrugated surfaces along the smooth carrier normal. Using
    // the relief normal here can fold the outer wall at narrow flute troughs.
    // Gauge is measured along this carrier direction, not the flute normal.
    mesh.extrusion_normals = Some(smooth_normals);
    Ok(())
}

/// Lay out the flute fan in lateral planes, so changes in the carrier's
/// cross-section do not bend the channels back inward near their upper ends.
fn flute_sample_coordinate(
    coarse: &MidMesh,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
    u: f32,
    row: usize,
) -> f32 {
    let pattern = design.fluting.as_ref().unwrap();
    let t = row as f32 / (V_SAMPLES - 1) as f32;
    let baseline = (fan_coordinate(u, t, false, design) + 1.0) * 0.5;
    let active =
        smoothstep(t / pattern.start.unit()) * smoothstep((1.0 - t) / (1.0 - pattern.end.unit()));
    if active == 0.0 || u == 0.0 {
        return baseline;
    }
    let lateral = |point| local(point, wearer.frame)[0];
    let reference_row = V_SAMPLES / 2 * U_SAMPLES;
    let left = lateral(coarse.positions[reference_row]);
    let right = lateral(coarse.positions[reference_row + U_SAMPLES - 1]);
    let current = &coarse.positions[row * U_SAMPLES..(row + 1) * U_SAMPLES];
    let current_left = lateral(current[0]);
    let current_right = lateral(current[U_SAMPLES - 1]);
    let center = lateral(current[U_SAMPLES / 2]);
    let half_width = ((right - left) * 0.5)
        .min(center - current_left)
        .min(current_right - center);
    let span = pattern.spread.unit();
    let fan = pattern.lower_spread.unit() + (1.0 - pattern.lower_spread.unit()) * smoothstep(t);
    let target = center + u.clamp(-span, span) * half_width * fan;
    let mut along = baseline;
    for (column, pair) in current.windows(2).enumerate() {
        let a = lateral(pair[0]);
        let b = lateral(pair[1]);
        if a <= target && target <= b && b - a > 1e-8 {
            along = (column as f32 + (target - a) / (b - a)) / (U_SAMPLES - 1) as f32;
            break;
        }
    }
    if u.abs() > span {
        let endpoint = if u < 0.0 { 0.0 } else { 1.0 };
        along += (endpoint - along) * (u.abs() - span) / (1.0 - span);
    }
    baseline * (1.0 - active) + along * active
}

/// Refine an already fitted and solidifiable carrier. Interpolate its extrusion
/// vectors as well as positions so density cannot change the original shell.
pub(super) fn refine_front(
    coarse: MidMesh,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<MidMesh, GenerateError> {
    if design.fluting.is_none() {
        return Ok(coarse);
    }
    let normals = match &coarse.extrusion_normals {
        Some(normals) => normals.clone(),
        None => vertex_normals(&coarse.positions, &coarse.faces)?,
    };
    let columns = chart_columns(false, design);
    let mut mesh = build_mid(false, wearer, design)?;
    let mut samples = Vec::with_capacity(mesh.positions.len());
    for row in 0..V_SAMPLES {
        for u in &columns {
            samples.push((
                row * U_SAMPLES,
                U_SAMPLES,
                flute_sample_coordinate(&coarse, wearer, design, *u, row),
            ));
        }
    }
    let coarse_skirt = U_SAMPLES * V_SAMPLES;
    for row in 0..SKIRT_SAMPLES - 1 {
        for u in &columns {
            samples.push((
                coarse_skirt + row * U_SAMPLES,
                U_SAMPLES,
                (fan_coordinate(*u, 0.0, false, design) + 1.0) * 0.5,
            ));
        }
    }
    let mut extrusion = Vec::with_capacity(samples.len());
    let mut carrier_samples = Vec::with_capacity(samples.len());
    for (point, (start, count, along)) in mesh.positions.iter_mut().zip(samples) {
        let sample = along.clamp(0.0, 1.0) * (count - 1) as f32;
        let lower = (sample.floor() as usize).min(count - 2);
        let t = sample - lower as f32;
        carrier_samples.push((start + lower, t));
        *point = add(
            scale(coarse.positions[start + lower], 1.0 - t),
            scale(coarse.positions[start + lower + 1], t),
        );
        extrusion.push(add(
            scale(normals[start + lower], 1.0 - t),
            scale(normals[start + lower + 1], t),
        ));
    }
    mesh.extrusion_normals = Some(extrusion);
    mesh.morph_carrier = Some(MorphCarrier {
        positions: coarse.positions,
        samples: carrier_samples,
    });
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_relief_has_requested_peak_count_width_and_taper() {
        for count in [2, 9, 24] {
            let mut spans = Vec::new();
            for width in [350, 850] {
                let mut design = BreastplateDesign::fluted();
                let pattern = design.fluting.as_mut().unwrap();
                pattern.count.0 = count;
                pattern.width.0 = width;
                let columns = chart_columns(false, &design);
                let grid: Vec<_> = (0..V_SAMPLES)
                    .map(|row| {
                        columns
                            .iter()
                            .map(|u| [*u, row as f32 / (V_SAMPLES - 1) as f32, 0.0])
                            .collect()
                    })
                    .collect();
                let mut mesh = MidMesh {
                    main_columns: columns.len(),
                    ..MidMesh::default()
                };
                append_grid(&mut mesh, &grid, false);
                mesh.extrusion_normals =
                    Some(vertex_normals(&mesh.positions, &mesh.faces).unwrap());
                apply_fluting(&mut mesh, &design).unwrap();
                let middle = &mesh.positions
                    [(V_SAMPLES / 2) * columns.len()..(V_SAMPLES / 2 + 1) * columns.len()];
                let peaks = middle
                    .windows(3)
                    .filter(|p| p[1][2] > p[0][2] && p[1][2] > p[2][2])
                    .count();
                assert_eq!(peaks, usize::from(count));
                let area: f32 = middle
                    .windows(2)
                    .map(|p| (p[1][0] - p[0][0]) * (p[0][2] + p[1][2]) * 0.5)
                    .sum();
                spans.push(area);
                assert!(mesh.positions[..columns.len()].iter().all(|p| p[2] == 0.0));
                assert!(
                    mesh.positions[(V_SAMPLES - 1) * columns.len()..]
                        .iter()
                        .all(|p| p[2] == 0.0)
                );
            }
            assert!(
                spans[1] > spans[0] * 2.0,
                "width must change relief independently of count"
            );
        }
    }
}
