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
    let section = wrapping_section(wrap, detail);
    let mut support_height = 0.0;
    let mut support_cover = 0.0;
    let mut parent = wrap.on_wrapping;
    while let Some(index) = parent {
        let support = &shaft.wrappings.as_ref().unwrap()[index.0 as usize];
        support_height += support.thickness.get();
        support_cover += support.underlay.as_ref().map_or(0.0, |p| p.thickness.get());
        parent = support.on_wrapping;
    }
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
    construction_budget(
        stations.len() as f64 * section.len() as f64 * 2.0 + section.len() as f64 * 2.0,
    )?;
    let maximum_step = stations
        .windows(2)
        .map(|s| (s[1] - s[0]) * (end - start))
        .fold(0.0, f64::max);
    let cell_phase_span = TAU / wrap.pitch.get() * (2.0 * maximum_step + wrap.width.get());
    let receiver_apothem = (sector / 2.0).cos();
    let ring = |progress: f64| -> Vec<Point> {
        let center = start + (end - start) * progress;
        let angle =
            wrap.phase.get().to_radians() + direction * TAU * (center - start) / wrap.pitch.get();
        section
            .iter()
            .map(|&[axial, radial]| {
                let y = center + axial * wrap.width.get();
                let lift = if wrap.pattern == WrappingPattern::Crossed && direction < 0.0 {
                    crossing_lift(wrap, start, y, angle, cell_phase_span) / receiver_apothem
                } else {
                    0.0
                };
                // Same polygonal section as the emitted shaft. Sampling every face
                // boundary makes each unlifted inner quad lie on its receiving face.
                let local_angle = angle.rem_euclid(sector) - sector / 2.0;
                let base_radius = shaft.radius_at(y)
                    + support_cover
                    + wrap.underlay.as_ref().map_or(0.0, |u| u.thickness.get());
                let surface = base_radius * receiver_apothem / local_angle.cos();
                let radius = if wrap.section.is_some() || wrap.on_wrapping.is_some() {
                    // Rounded compressed cord has planar crest/underside footprints
                    // on the actual receiving polygon, including supported layers.
                    surface
                        + lift
                        + (support_height + radial * wrap.thickness.get()) / local_angle.cos()
                } else {
                    surface + lift + radial * wrap.thickness.get()
                };
                [radius * angle.cos(), y, radius * angle.sin()]
            })
            .collect()
    };
    let mut solid = Solid::default();
    for pair in stations.windows(2) {
        let a = ring(pair[0]);
        let b = ring(pair[1]);
        for side in 0..section.len() {
            let next = (side + 1) % section.len();
            let group = if wrap.section.is_some() {
                u32::from(side != 0) + 1
            } else {
                side as u32 + 1
            };
            solid.quad(a[side], b[side], b[next], a[next], group);
        }
    }
    let first = ring(0.0);
    let last = ring(1.0);
    solid.cap_wrapping(&first, &last, wrap.section.as_ref());
    Ok(solid.positive())
}

fn wrapping_section(wrap: &ShaftWrapping, detail: Detail) -> Vec<PlanarPoint> {
    let Some(WrappingSection::Rounded { crest_fraction }) = &wrap.section else {
        return vec![[-0.5, 0.0], [0.5, 0.0], [0.5, 1.0], [-0.5, 1.0]];
    };
    let crest = crest_fraction.get() / 2.0;
    let shoulder = 0.5 - crest;
    let samples = detail.samples(6, 4);
    let mut section = vec![[-0.5, 0.0], [0.5, 0.0]];
    for step in 1..=samples {
        let angle = PI / 2.0 * step as f64 / samples as f64;
        section.push([crest + shoulder * angle.cos(), angle.sin()]);
    }
    section.push([-crest, 1.0]);
    for step in 1..samples {
        let angle = PI / 2.0 + PI / 2.0 * step as f64 / samples as f64;
        section.push([-crest + shoulder * angle.cos(), angle.sin()]);
    }
    section
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

impl Solid {
    fn cap_wrapping(&mut self, first: &[Point], last: &[Point], section: Option<&WrappingSection>) {
        if section.is_some() {
            for (ring, reverse, group) in [(first, false, 5), (last, true, 6)] {
                let center = mul(
                    ring.iter().copied().fold([0.0; 3], add),
                    1.0 / ring.len() as f64,
                );
                for side in 0..ring.len() {
                    let a = ring[side];
                    let b = ring[(side + 1) % ring.len()];
                    if reverse {
                        self.triangle(center, b, a, group);
                    } else {
                        self.triangle(center, a, b, group);
                    }
                }
            }
        } else {
            self.quad(first[0], first[1], first[2], first[3], 5);
            self.quad(last[3], last[2], last[1], last[0], 6);
        }
    }
}
