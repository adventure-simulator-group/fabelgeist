//! Radial flanges and their turned structural core.
use super::*;
use std::f64::consts::PI;

pub(super) fn mace(
    r: &ResolvedComponent,
    p: &MaceParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let length = p.length.get();
    let half = length / 2.0;
    let root = p.root_radius.get();
    let shoulder = p.shoulder_radius.get();
    let crown = p.crown_length.map_or(0.0, Metres::get);
    let material = r.component.material.unwrap_or(Material::Steel);
    let mut core = vec![[-half, root], [half * 0.72, root * 0.9], [half, shoulder]];
    if crown > 0.0 {
        core.push([half + crown, 0.001]);
    }
    if let Some(profile) = &p.core_profile {
        core = profile.iter().map(|p| p.map(Metres::get)).collect();
    }
    let sides = p.segments.map_or(p.flanges.0 as usize, |n| n.0 as usize);
    if !sides.is_multiple_of(p.flanges.0 as usize) {
        return Err("mace core sides must be a multiple of its flange count".into());
    }
    let angle = PI / sides as f64;
    let outer = flange_outer(p, detail);
    let inner = receiving_profile(&core, &outer, p.flange_thickness.get(), angle)?;
    let mut parts = vec![PartSource::new(
        Solid::faceted_lathe(&core, sides, detail)?
            .transform([0.0, -180.0 / sides as f64, 0.0], [0.0; 3]),
        material,
        &r.label,
        &r.id,
    )];
    let mut outline = outer;
    outline.extend(inner.into_iter().rev());
    let coincident = |a: &PlanarPoint, b: &PlanarPoint| (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-10;
    outline.dedup_by(|a, b| coincident(a, b));
    if coincident(outline.first().unwrap(), outline.last().unwrap()) {
        outline.pop();
    }
    let flange = Solid::prism(&outline, p.flange_thickness.get(), detail)?;
    for index in 0..p.flanges.0 {
        parts.push(PartSource::new(
            flange.clone().transform(
                [0.0, -(index as f64 / p.flanges.0 as f64) * 360.0, 0.0],
                [0.0; 3],
            ),
            material,
            &format!("{} flange", r.label),
            &r.id,
        ));
    }
    Ok(parts)
}

/// The inner flange edge shares the actual polygon face, including each change
/// in core slope. Checking the union of both polylines' stations proves that
/// their piecewise-linear boundaries never cross between stations.
fn receiving_profile(
    core: &[PlanarPoint],
    outer: &[PlanarPoint],
    thickness: f64,
    angle: f64,
) -> Result<Vec<PlanarPoint>, String> {
    if core.windows(2).any(|p| p[1][0] <= p[0][0]) {
        return Err("mace core stations must have strictly increasing heights".into());
    }
    let bottom = outer[0][1];
    let top = outer.last().unwrap()[1];
    let interpolate = |profile: &[PlanarPoint], height: f64| {
        let span = profile
            .windows(2)
            .find(|p| p[1][0] > p[0][0] && height <= p[1][0])
            .unwrap();
        let t = (height - span[0][0]) / (span[1][0] - span[0][0]);
        span[0][1] + (span[1][1] - span[0][1]) * t
    };
    let silhouette: Vec<_> = outer.iter().map(|p| [p[1], p[0]]).collect();
    for &[radius, height] in outer {
        if interpolate(core, height) * angle.cos() >= radius {
            return Err("mace core face reaches outside the flange outline".into());
        }
    }
    let mut heights: Vec<_> = outer
        .iter()
        .map(|p| p[1])
        .chain(core.iter().map(|p| p[0]).filter(|&y| y > bottom && y < top))
        .collect();
    heights.sort_by(f64::total_cmp);
    heights.dedup();
    let mut inner = Vec::new();
    for height in heights {
        let radius = interpolate(core, height);
        if thickness >= 2.0 * radius * angle.sin() {
            return Err("mace flange thickness exceeds its receiving core face".into());
        }
        let seat = radius * angle.cos();
        if seat >= interpolate(&silhouette, height) {
            return Err("mace core face reaches outside the flange outline".into());
        }
        inner.push([seat, height]);
    }
    Ok(inner)
}

pub(super) fn flange_outer(p: &MaceParameters, detail: Detail) -> Vec<PlanarPoint> {
    let length = p.length.get();
    let half = length / 2.0;
    if let Some(profile) = &p.flange_profile {
        return profile
            .iter()
            .map(|s| {
                [
                    s.radius.get(),
                    if s.at.get() == 1.0 {
                        half
                    } else {
                        -half + length * s.at.get()
                    },
                ]
            })
            .collect();
    }
    let root = p.root_radius.get();
    let shoulder = p.shoulder_radius.get();
    let cusp = p.cusp_radius.get();
    let cusp_y = -half + length * p.cusp_height.map_or(0.58, Ratio::get);
    let exponent = 1.03 + p.concavity.map_or(0.0, Ratio::get).clamp(0.0, 0.98) * 2.97;
    let quality = CurveQuality {
        minimum_segments: p.profile_samples.map_or(10, |n| n.0 as usize),
        max_chord: length / 22.0,
        max_deviation: (cusp - root.min(shoulder)) / 150.0,
    };
    let mut outer = adaptive_curve(
        |t| {
            [
                root + (cusp - root) * t.powf(exponent),
                -half + (cusp_y + half) * t,
            ]
        },
        quality,
        detail,
    );
    outer.extend(
        adaptive_curve(
            |t| {
                if t == 1.0 {
                    return [shoulder, half];
                }
                [
                    shoulder + (cusp - shoulder) * (1.0 - t).powf(exponent),
                    cusp_y + (half - cusp_y) * t,
                ]
            },
            quality,
            detail,
        )
        .into_iter()
        .skip(1),
    );
    outer
}
