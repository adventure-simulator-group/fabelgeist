//! Whole-body horizontal support sections clipped to occupied main material.

use serde::Serialize;

pub(crate) const SECTION_INTERVALS: usize = 256;
const PLANE_EPSILON_M: f64 = 1e-14;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct FrontSection {
    pub height_m: f64,
    pub floor_m: f64,
    pub raw_body_z_m: f64,
    pub body_face: usize,
    pub x_m: f64,
    pub lateral_limit_m: f64,
    pub clipped_segments: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct FrontSupport {
    pub height_m: f64,
    pub floor_m: f64,
    pub origin: &'static str,
}

pub(crate) fn sections(
    vertices: &[[f64; 3]],
    faces: &[[u32; 3]],
    outline: &[[f64; 2]],
    heights: [f64; 2],
    padding_m: f64,
) -> Result<Vec<FrontSection>, String> {
    if faces.is_empty()
        || faces
            .iter()
            .flatten()
            .any(|i| *i as usize >= vertices.len())
        || vertices.iter().flatten().any(|v| !v.is_finite())
        || outline.len() < 3
        || outline.iter().flatten().any(|v| !v.is_finite())
        || heights.iter().any(|v| !v.is_finite())
        || !padding_m.is_finite()
        || padding_m < 0.
        || heights[1] <= heights[0]
    {
        return Err("Invalid full-body front-envelope domain".into());
    }
    (0..=SECTION_INTERVALS)
        .map(|i| {
            let y = heights[0] + (heights[1] - heights[0]) * i as f64 / SECTION_INTERVALS as f64;
            let intervals = material_intervals(outline, y);
            if intervals.is_empty() {
                return Err(format!("No occupied front section at height {y}"));
            }
            let limit = intervals
                .iter()
                .flatten()
                .map(|x| x.abs())
                .fold(0., f64::max);
            let mut best = None::<FrontSection>;
            let mut count = 0;
            for (index, face) in faces.iter().enumerate() {
                let triangle = face.map(|i| vertices[i as usize]);
                let mut crossings = Vec::new();
                let mut segments = Vec::new();
                for (a, b) in [(0, 1), (1, 2), (2, 0)] {
                    let (a, b) = (triangle[a], triangle[b]);
                    if (a[1] - y).abs() <= PLANE_EPSILON_M && (b[1] - y).abs() <= PLANE_EPSILON_M {
                        segments.push(([a[0], a[2]], [b[0], b[2]]));
                    }
                    if y >= a[1].min(b[1])
                        && y <= a[1].max(b[1])
                        && (b[1] - a[1]).abs() > PLANE_EPSILON_M
                    {
                        let t = (y - a[1]) / (b[1] - a[1]);
                        crossings.push([a[0] + t * (b[0] - a[0]), a[2] + t * (b[2] - a[2])]);
                    }
                }
                if crossings.len() >= 2 {
                    crossings.sort_by(|a, b| a[0].total_cmp(&b[0]));
                    segments.push((crossings[0], *crossings.last().unwrap()));
                }
                for (a, b) in segments {
                    let (a, b) = if a[0] <= b[0] { (a, b) } else { (b, a) };
                    for &[left, right] in &intervals {
                        if b[0] < left || a[0] > right {
                            continue;
                        }
                        count += 1;
                        for x in [a[0].max(left), b[0].min(right)] {
                            let z = if (b[0] - a[0]).abs() < PLANE_EPSILON_M {
                                a[1].max(b[1])
                            } else {
                                a[1] + (b[1] - a[1]) * (x - a[0]) / (b[0] - a[0])
                            };
                            if best.as_ref().is_none_or(|p| z > p.raw_body_z_m) {
                                best = Some(FrontSection {
                                    height_m: y,
                                    floor_m: z + padding_m,
                                    raw_body_z_m: z,
                                    body_face: index,
                                    x_m: x,
                                    lateral_limit_m: limit,
                                    clipped_segments: 0,
                                });
                            }
                        }
                    }
                }
            }
            let mut best =
                best.ok_or_else(|| format!("No body intersects occupied front section {y}"))?;
            best.clipped_segments = count;
            Ok(best)
        })
        .collect()
}

fn material_intervals(outline: &[[f64; 2]], y: f64) -> Vec<[f64; 2]> {
    let mut crossings = outline
        .iter()
        .zip(outline.iter().cycle().skip(1))
        .take(outline.len())
        .flat_map(|(a, b)| {
            if y < a[1].min(b[1]) || y > a[1].max(b[1]) {
                return Vec::new();
            }
            if (b[1] - a[1]).abs() <= PLANE_EPSILON_M {
                return vec![a[0], b[0]];
            }
            vec![a[0] + (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1])]
        })
        .collect::<Vec<_>>();
    crossings.sort_by(f64::total_cmp);
    crossings.dedup_by(|a, b| (*a - *b).abs() <= PLANE_EPSILON_M);
    crossings
        .windows(2)
        .filter_map(|pair| {
            let x = 0.5 * (pair[0] + pair[1]);
            let mut inside = false;
            for (a, b) in outline
                .iter()
                .zip(outline.iter().cycle().skip(1))
                .take(outline.len())
            {
                if (a[1] - y).abs() <= PLANE_EPSILON_M
                    && (b[1] - y).abs() <= PLANE_EPSILON_M
                    && x >= a[0].min(b[0])
                    && x <= a[0].max(b[0])
                {
                    return Some([pair[0], pair[1]]);
                }
                if (a[1] > y) != (b[1] > y) && x < a[0] + (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1])
                {
                    inside = !inside;
                }
            }
            inside.then_some([pair[0], pair[1]])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coplanar_front_peak_and_asymmetric_material_are_not_discarded() {
        let vertices = [
            [-0.2, 0., 0.1],
            [0.2, 0., 0.1],
            [0., 1., 0.1],
            [-0.1, 0.5, 0.4],
            [0.15, 0.5, 0.6],
            [0.1, 0.5, 0.9],
        ];
        let outline = [[-0.1, 0.], [0.15, 0.], [0.15, 1.], [-0.1, 1.]];
        let rows = sections(&vertices, &[[0, 1, 2], [3, 4, 5]], &outline, [0., 1.], 0.).unwrap();
        assert_eq!(rows[128].raw_body_z_m, 0.9);
        assert_eq!(material_intervals(&outline, 0.5), vec![[-0.1, 0.15]]);
        for padding in [-0.01, f64::NAN] {
            assert!(sections(&vertices, &[[0, 1, 2]], &outline, [0., 1.], padding).is_err());
        }
        assert!(sections(&vertices, &[[0, 1, 2]], &outline, [0., f64::NAN], 0.).is_err());
        assert!(sections(&vertices, &[[0, 1, 2]], &[[f64::NAN, 0.]; 3], [0., 1.], 0.).is_err());
    }
    #[test]
    fn section_clipping_excludes_external_limb_and_applies_padding_once() {
        let vertices = [
            [-0.2, 0., 0.1],
            [0.2, 0., 0.1],
            [0., 1., 0.2],
            [0.5, 0., 1.],
            [0.8, 0., 1.],
            [0.6, 1., 1.],
        ];
        let faces = [[0, 1, 2], [3, 4, 5]];
        let outline = [[-0.1, 0.], [0.1, 0.], [0.1, 1.], [-0.1, 1.]];
        let rows = sections(&vertices, &faces, &outline, [0., 0.9], 0.0165).unwrap();
        for row in rows {
            assert!((row.raw_body_z_m - (0.1 + 0.1 * row.height_m)).abs() < 1e-12);
            assert!((row.floor_m - row.raw_body_z_m - 0.0165).abs() < 1e-12);
            assert_eq!(row.body_face, 0);
            assert!(row.x_m.abs() <= 0.1);
        }
    }
}
