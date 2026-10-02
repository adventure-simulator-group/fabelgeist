//! A lathed fitting seated outside the actual quillon end plane.
use super::*;

pub(super) fn terminal_profile(
    p: &TerminalProfile,
    tangent: Point,
    end: Point,
    receiver: &Solid,
    detail: Detail,
) -> Result<Solid, String> {
    let axis = normalize(tangent);
    let mut radius = 0.0_f64;
    for &vertex in &receiver.positions {
        let delta = sub(vertex, end);
        let axial = dot(delta, axis);
        if axial > 1e-9 {
            return Err("quillon crosses the terminal joint plane".into());
        }
        if axial.abs() < 1e-9 {
            radius = radius.max(magnitude(sub(delta, mul(axis, axial))));
        }
    }
    let points: Vec<_> = p.stations.iter().map(|s| s.map(Metres::get)).collect();
    let largest = points.iter().map(|p| p[1]).fold(0.0, f64::max);
    let segments = detail.radial(largest, 16);
    if radius <= 0.0 || points[0][1] * (PI / segments as f64).cos() < radius {
        return Err("terminal base must cover its receiving quillon section".into());
    }
    let mut sampled = Vec::new();
    let mut start = 0;
    while start < points.len() - 1 {
        let mut end = start + 1;
        while end < points.len() && points[end][0] > points[end - 1][0] {
            end += 1;
        }
        let run = &points[start..end];
        if run.len() > 1 && p.interpolation == ProfileInterpolation::Smooth {
            let curve = SmoothProfile::new(run.to_vec())?;
            for pair in run.windows(2) {
                sampled.extend(adaptive_curve(
                    |u| {
                        let y = pair[0][0] + u * (pair[1][0] - pair[0][0]);
                        [y, curve.value(y)]
                    },
                    CurveQuality {
                        minimum_segments: 2,
                        max_chord: 0.003,
                        max_deviation: 0.000025,
                    },
                    detail,
                ));
            }
        } else {
            sampled.extend_from_slice(run);
        }
        if end == points.len() {
            break;
        }
        start = end;
    }
    if sampled.last() != points.last() {
        sampled.push(*points.last().unwrap());
    }
    sampled.dedup();
    let rotation = [
        tangent[2].atan2(tangent[0].hypot(tangent[1])).to_degrees(),
        0.0,
        (-tangent[0]).atan2(tangent[1]).to_degrees(),
    ];
    Ok(Solid::lathe(&sampled, 16, 1.0, false, detail)?.transform(rotation, [0.0; 3]))
}
