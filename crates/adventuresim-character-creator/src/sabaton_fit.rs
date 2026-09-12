//! Shared longitudinal foot sections keep toe and instep laps registered.
use super::{FitRegion, Side, Wearer, local};
use adventuresim_armor_model::{FootArmorDesign, GenerateError, PartFrame, PartMesh};
use anyhow::Result;
#[path = "foot_profile.rs"]
mod foot_profile;
use foot_profile::FootProfile;

pub(super) fn fit(
    mesh: PartMesh,
    d: &FootArmorDesign,
    wearer: &Wearer<'_>,
    side: Side,
    frame: &PartFrame,
) -> Result<PartMesh> {
    let points = wearer
        .support_indices(FitRegion::Foot(side))?
        .into_iter()
        .map(|i| local(frame, wearer.positions[i]))
        .collect::<Vec<_>>();
    let [width, height, length] = frame.half_extents;
    let available_span_m = length * 0.70;
    // Match generation at the exact trim boundary despite f32 roundoff.
    if length <= d.ankle_cutaway.metres() / 0.70 {
        return Err(GenerateError::SabatonTrimExceedsFoot {
            cutaway_m: d.ankle_cutaway.metres(),
            available_span_m,
        }
        .into());
    }
    let gauge = d.gauge.thickness.metres();
    let gap = d.gauge.clearance.metres() + gauge + 0.002;
    let sole = -height + gauge;
    let front = points
        .iter()
        .map(|p| p[2])
        .fold(f32::NEG_INFINITY, f32::max);
    let end = (length + d.toe_extension.metres()).max(front + gap + d.toe_extension.metres());
    let ankle_start = -length * 0.05 + d.ankle_cutaway.metres();
    let profile = FootProfile::new(&points, ankle_start, front - 0.004);
    let mut carrier = 0;
    Ok(mesh.refit_surfaces(|surface, _| {
        let toe = carrier == usize::from(d.lame_count);
        for point in surface {
            let mut p = local(frame, *point);
            let original_z = p[2];
            let (authored_width, authored_height) = if toe {
                let t = ((original_z - length * 0.60) / (length * 0.40 + d.toe_extension.metres()))
                    .clamp(0.0, 1.0);
                let cosine = (1.0 - t * t).sqrt();
                p[2] = length * 0.60 + (end - length * 0.60) * t;
                (
                    (width * d.toe_width.unit() + d.gauge.clearance.metres() + gauge)
                        * cosine.powf(d.toe_roundness.unit()),
                    height * 1.1 * d.instep_height.unit() * cosine,
                )
            } else {
                let t =
                    ((original_z - ankle_start) / (length * 0.65 - ankle_start)).clamp(0.0, 1.0);
                let smooth = |x: f32| x * x * (3.0 - 2.0 * x);
                let taper = if t < 0.55 {
                    0.63 + 0.37 * smooth(t / 0.55)
                } else {
                    1.0 + (d.toe_width.unit() - 1.0) * smooth((t - 0.55) / 0.45)
                };
                (
                    width * taper + d.gauge.clearance.metres() + gauge,
                    height * d.instep_height.unit() * (1.75 - 0.65 * smooth(t)),
                )
            };
            if authored_width < 1e-6 || authored_height < 1e-6 {
                *point = frame.point([0.0, sole, end]);
                continue;
            }
            let angle = (p[0] / authored_width).atan2((p[1] - sole) / authored_height);
            let [center, half_width, top] = profile.at(p[2]);
            let nose = ((p[2] - (front + gap)) / (end - front - gap).max(0.001)).clamp(0.0, 1.0);
            let return_scale = (1.0 - nose * nose).sqrt();
            let toe_blend = ((p[2] / length - 0.15) / 0.5).clamp(0.0, 1.0);
            let breadth = half_width + gap + width * 0.8 * (d.toe_width.unit() - 0.8) * toe_blend;
            // A low, rounded rectangular section encloses the toe corners.
            let rounded = |a: f32| a.signum() * a.abs().powf(0.60);
            p[0] = center * return_scale
                + breadth * return_scale.powf(d.toe_roundness.unit()) * rounded(angle.sin());
            p[1] = sole
                + (top - sole + gap + height * 0.3 * (d.instep_height.unit() - 0.85))
                    * return_scale
                    * rounded(angle.cos());
            *point = frame.point(p);
        }
        carrier += 1;
    })?)
}
