//! Flat spiral strips follow the shaft taper and lift over crossing strips.
use super::*;
use std::f64::consts::{PI, TAU};

pub(super) fn parts(
    shaft: &Shaft,
    id: &str,
    label: &str,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let mut parts = Vec::new();
    for (index, wrapping) in shaft.wrappings.iter().flatten().enumerate() {
        if let Some(underlay) = &wrapping.underlay {
            let heights = [
                wrapping.start.get(),
                wrapping.start.get() + wrapping.length.get(),
            ];
            let inner = heights.map(|y| shaft.radius_at(y));
            let profile = std::array::from_fn::<_, 2, _>(|i| {
                [heights[i], inner[i] + underlay.thickness.get()]
            });
            let largest = shaft.profile().iter().map(|p| p[1]).fold(0.0, f64::max);
            let sides =
                detail.lathe_radial(largest, shaft.segments.map_or(16, |n| n.0 as usize), true);
            parts.push(PartSource::new(
                Solid::hollow_profile(&profile, &inner, sides),
                underlay.material,
                &format!("{label} wrapping {} underlay", index + 1),
                id,
            ));
        }
        let directions: &[f64] = match wrapping.pattern {
            WrappingPattern::RightHanded => &[1.0],
            WrappingPattern::LeftHanded => &[-1.0],
            WrappingPattern::Crossed => &[1.0, -1.0],
        };
        for &direction in directions {
            parts.push(PartSource::new(
                strip(shaft, wrapping, direction, detail)?,
                wrapping.material,
                &format!("{label} wrapping {} {direction}", index + 1),
                id,
            ));
        }
    }
    Ok(parts)
}

fn strip(
    shaft: &Shaft,
    wrap: &ShaftWrapping,
    direction: f64,
    detail: Detail,
) -> Result<Solid, String> {
    let start = wrap.start.get() + wrap.width.get() / 2.0;
    let end = wrap.start.get() + wrap.length.get() - wrap.width.get() / 2.0;
    let turns = (end - start) / wrap.pitch.get();
    let segments = (turns * detail.radial(shaft.radius.get(), 24) as f64)
        .ceil()
        .max(4.0) as usize;
    let largest = shaft.profile().iter().map(|p| p[1]).fold(0.0, f64::max);
    let shaft_sides =
        detail.lathe_radial(largest, shaft.segments.map_or(16, |n| n.0 as usize), true);
    let sector = TAU / shaft_sides as f64;
    let mut stations: Vec<_> = (0..=segments).map(|n| n as f64 / segments as f64).collect();
    let phase = wrap.phase.get().to_radians();
    let last_angle = phase + direction * turns * TAU;
    for corner in (phase.min(last_angle) / sector).ceil() as i64
        ..=(phase.max(last_angle) / sector).floor() as i64
    {
        stations.push((corner as f64 * sector - phase) / (direction * turns * TAU));
    }
    stations.sort_by(f64::total_cmp);
    stations.dedup_by(|a, b| (*a - *b).abs() < 1e-8);
    construction_budget(stations.len() as f64 * 8.0 + 4.0)?;
    let maximum_step = stations
        .windows(2)
        .map(|s| (s[1] - s[0]) * (end - start))
        .fold(0.0, f64::max);
    let cell_phase_span = TAU / wrap.pitch.get() * (2.0 * maximum_step + wrap.width.get());
    let receiver_apothem = (sector / 2.0).cos();
    let ring = |progress: f64| -> [Point; 4] {
        let center = start + (end - start) * progress;
        let angle =
            wrap.phase.get().to_radians() + direction * TAU * (center - start) / wrap.pitch.get();
        [[-0.5, 0.0], [0.5, 0.0], [0.5, 1.0], [-0.5, 1.0]].map(|[axial, radial]| {
            let y = center + axial * wrap.width.get();
            let lift = if wrap.pattern == WrappingPattern::Crossed && direction < 0.0 {
                crossing_lift(wrap, start, y, angle, cell_phase_span) / receiver_apothem
            } else {
                0.0
            };
            // Same polygonal section as the emitted shaft. Sampling every face
            // boundary makes each unlifted inner quad lie on its receiving face.
            let local_angle = angle.rem_euclid(sector) - sector / 2.0;
            let base_radius =
                shaft.radius_at(y) + wrap.underlay.as_ref().map_or(0.0, |u| u.thickness.get());
            let surface = base_radius * receiver_apothem / local_angle.cos();
            let radius = surface + lift + radial * wrap.thickness.get();
            [radius * angle.cos(), y, radius * angle.sin()]
        })
    };
    let mut solid = Solid::default();
    for pair in stations.windows(2) {
        let a = ring(pair[0]);
        let b = ring(pair[1]);
        for side in 0..4 {
            let next = (side + 1) % 4;
            solid.quad(a[side], b[side], b[next], a[next], side as u32 + 1);
        }
    }
    let first = ring(0.0);
    let last = ring(1.0);
    solid.quad(first[0], first[1], first[2], first[3], 5);
    solid.quad(last[3], last[2], last[1], last[0], 6);
    Ok(solid.positive())
}

fn crossing_lift(
    wrap: &ShaftWrapping,
    start: f64,
    y: f64,
    angle: f64,
    cell_phase_span: f64,
) -> f64 {
    let first_angle = wrap.phase.get().to_radians() + TAU * (y - start) / wrap.pitch.get();
    let separation = ((angle - first_angle + PI).rem_euclid(TAU) - PI).abs();
    let contact = PI * wrap.width.get() / wrap.pitch.get();
    // Any cell meeting the lower strip must keep all four corners above it.
    // Phase is linear across a cell, so this span bounds every interior point.
    let progress = ((separation - contact - cell_phase_span) / contact).clamp(0.0, 1.0);
    wrap.thickness.get() * (1.0 - progress * progress * (3.0 - 2.0 * progress))
}
