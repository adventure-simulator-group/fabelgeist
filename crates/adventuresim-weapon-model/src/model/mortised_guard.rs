//! Transverse section loft with an open, materially subtracted blade mortise.
use super::*;

const GUARD_TRANSVERSE_SAMPLES: usize = 32;
const MIN_GUARD_TRANSVERSE_SAMPLES: usize = 16;
const MORTISE_POSITION_TOLERANCE: f64 = 1e-10;

/// A mounted blade must retain the actual constant section cut from the guard.
pub(super) fn check_mating(
    child: &ResolvedComponent,
    parent: &ResolvedComponent,
) -> Result<(), String> {
    let (Shape::LoftedBlade(blade), Shape::MortisedGuard(guard)) =
        (&child.component.shape, &parent.component.shape)
    else {
        return Ok(());
    };
    let section_matches = blade.width == guard.mortise.width
        && blade.thickness == guard.mortise.thickness
        && blade.plan == BladePlan::Straight
        && blade.single_edge.get() == 0.0
        && blade.ricasso.get() >= guard.shoulder_height.get()
        && blade.fuller.as_ref().is_some_and(|fuller| {
            fuller.bevel_width_ratio == guard.mortise.bevel_width_ratio
                && fuller
                    .grooves
                    .iter()
                    .all(|g| g.start.get() >= guard.shoulder_height.get())
        });
    let expected = add(
        parent.offset,
        rotate([0.0, guard.height.get(), 0.0], parent.rotation),
    );
    if !section_matches
        || child.rotation != parent.rotation
        || magnitude(sub(child.offset, expected)) > MORTISE_POSITION_TOLERANCE
    {
        return Err(
            "mounted blade must match the guard mortise section, engagement and pose".into(),
        );
    }
    Ok(())
}

pub(super) fn construct(p: &MortisedGuardParameters, detail: Detail) -> Result<Solid, String> {
    let half = p.width.get() / 2.0;
    let blade = p.mortise.width.get() / 2.0;
    let flat = blade * (1.0 - p.mortise.bevel_width_ratio.get());
    let count = detail.samples(GUARD_TRANSVERSE_SAMPLES, MIN_GUARD_TRANSVERSE_SAMPLES);
    let mut stations: Vec<_> = (0..=count)
        .map(|i| -half + p.width.get() * i as f64 / count as f64)
        .collect();
    stations.extend([-blade, -flat, 0.0, flat, blade]);
    stations.sort_by(f64::total_cmp);
    stations.dedup();
    let rings: Vec<_> = stations.iter().map(|&x| section(p, x)).collect();
    let mut solid = Solid::default();
    for pair in rings.windows(2) {
        for side in 0..pair[0].len() {
            let next = (side + 1) % pair[0].len();
            face(
                &mut solid,
                [pair[0][side], pair[0][next], pair[1][next], pair[1][side]],
                side as u32 + 1,
            );
        }
    }
    for (ring, reverse) in [
        (rings.first().unwrap(), true),
        (rings.last().unwrap(), false),
    ] {
        let mut outline: Vec<_> = ring.iter().map(|p| [p[1], p[2]]).collect();
        outline.dedup();
        let region = Region::triangulate(&outline, true)?;
        for [a, b, c] in region.triangles {
            let point = |i: usize| [ring[0][0], region.points[i][0], region.points[i][1]];
            if reverse {
                solid.triangle(point(a), point(c), point(b), 0);
            } else {
                solid.triangle(point(a), point(b), point(c), 0);
            }
        }
    }
    Ok(solid.positive())
}

fn section(p: &MortisedGuardParameters, x: f64) -> Vec<Point> {
    let half = p.width.get() / 2.0;
    let blade = p.mortise.width.get() / 2.0;
    let arm = ((x.abs() - blade) / (half - blade)).clamp(0.0, 1.0);
    let flare = 1.0 + (p.terminal_scale.get() - 1.0) * arm * arm;
    let bottom = p.sweep.get() * arm * arm;
    let top = bottom
        + p.height.get() * flare
        + p.shoulder_height.get() * (1.0 - x.abs() / blade).max(0.0);
    let slot = p.mortise.thickness.get() / 2.0
        * ((blade - x.abs()) / (blade * p.mortise.bevel_width_ratio.get())).clamp(0.0, 1.0);
    let floor = if x.abs() < blade { p.height.get() } else { top };
    let depth = p.thickness.get() / 2.0;
    let bevel = p.edge_bevel.get();
    [
        [bottom + bevel, -depth],
        [top - bevel, -depth],
        [top, -depth + bevel],
        [top, -slot],
        [floor, -slot],
        [floor, slot],
        [top, slot],
        [top, depth - bevel],
        [top - bevel, depth],
        [bottom + bevel, depth],
        [bottom, depth - bevel],
        [bottom, -depth + bevel],
    ]
    .into_iter()
    .map(|[y, z]| [x, y, z])
    .collect()
}

fn face(solid: &mut Solid, points: [Point; 4], surface: u32) {
    let mut unique = Vec::new();
    for point in points {
        if !unique.contains(&point) {
            unique.push(point);
        }
    }
    for index in 1..unique.len().saturating_sub(1) {
        solid.triangle(unique[0], unique[index], unique[index + 1], surface);
    }
}
