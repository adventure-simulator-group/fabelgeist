//! Radial flanges and their turned structural core.
use super::*;

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
    let mut parts = vec![PartSource::new(
        Solid::lathe(
            &core,
            p.segments.map_or(12, |n| n.0 as usize),
            1.0,
            false,
            detail,
        )?,
        material,
        &r.label,
        &r.id,
    )];
    let outer = flange_outer(p, detail);
    let scale = p.flange_root_scale.map_or(0.55, Ratio::get);
    let mut outline = outer;
    outline.extend([[shoulder * scale, half], [root * scale, -half]]);
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

pub(super) fn flange_outer(p: &MaceParameters, detail: Detail) -> Vec<PlanarPoint> {
    let length = p.length.get();
    let half = length / 2.0;
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
