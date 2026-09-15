//! Authored path stations and transported section frames.
use super::*;
pub(super) struct SampledPath {
    pub(super) points: Vec<Point>,
    pub(super) scales: Vec<f64>,
    pub(super) progress: Vec<f64>,
}
impl SampledPath {
    pub(super) fn new(
        input: &[Point],
        sweep: &Sweep,
        section_vertices: usize,
        detail: Detail,
        closed: bool,
    ) -> Result<Self, String> {
        let room = if sweep.fit_bends {
            bend_scales(input, sweep.width.hypot(sweep.depth) / 2.0, closed)
        } else {
            sweep
                .ring_scales
                .clone()
                .unwrap_or_else(|| vec![1.0; input.len()])
        };
        if room.len() != input.len() {
            return Err("section scales must match member stations".into());
        }
        let chord = detail.error(0.025).min(
            sweep.width.min(sweep.depth)
                * sweep.tip_scale.min(1.0)
                * (PI / section_vertices as f64).sin()
                * 32.0,
        );
        let stations = input
            .windows(2)
            .map(|p| (magnitude(sub(p[1], p[0])) / chord).ceil().max(1.0))
            .sum::<f64>()
            + 1.0;
        construction_budget(stations * section_vertices as f64 * 2.0)?;
        let mut points = Vec::new();
        let mut scales = Vec::new();
        let mut progress = Vec::new();
        for i in 0..input.len() - 1 {
            let count = (magnitude(sub(input[i + 1], input[i])) / chord)
                .ceil()
                .max(1.0) as usize;
            for step in 0..count {
                let t = step as f64 / count as f64;
                points.push(lerp(input[i], input[i + 1], t));
                scales.push(room[i] * (1.0 - t) + room[i + 1] * t);
                progress.push((i as f64 + t) / (input.len() - 1) as f64);
            }
        }
        points.push(*input.last().unwrap());
        scales.push(*room.last().unwrap());
        progress.push(1.0);

        Ok(Self {
            points,
            scales,
            progress,
        })
    }
}
pub(super) struct TransportFrames {
    pub(super) tangents: Vec<Point>,
    pub(super) normals: Vec<Point>,
}
impl TransportFrames {
    pub(super) fn along(points: &[Point], closed: bool) -> Result<Self, String> {
        let n = points.len();
        let tangents: Vec<_> = (0..n)
            .map(|row| {
                normalize(sub(
                    points[if row == n - 1 {
                        if closed { 1 } else { row }
                    } else {
                        row + 1
                    }],
                    points[if row == 0 {
                        if closed { n - 2 } else { 0 }
                    } else {
                        row - 1
                    }],
                ))
            })
            .collect();
        let mut frames = Vec::new();
        for row in 0..n {
            let tangent = tangents[row];
            if row == 0 {
                let reference = if tangent[2].abs() < 0.8 {
                    [0.0, 0.0, 1.0]
                } else {
                    [0.0, 1.0, 0.0]
                };
                frames.push(normalize(cross(reference, tangent)));
            } else {
                let axis = cross(tangents[row - 1], tangent);
                let sine = magnitude(axis);
                let cosine = dot(tangents[row - 1], tangent);
                if cosine < -0.99 {
                    return Err("member centerline reverses direction".into());
                }
                frames.push(if sine < 1e-8 {
                    frames[row - 1]
                } else {
                    normalize(rotate_around(
                        frames[row - 1],
                        mul(axis, 1.0 / sine),
                        sine.atan2(cosine),
                    ))
                });
            }
        }
        if closed {
            let angle = dot(cross(frames[n - 1], frames[0]), tangents[0])
                .atan2(dot(frames[n - 1], frames[0]));
            for row in 0..n {
                frames[row] = rotate_around(
                    frames[row],
                    tangents[row],
                    angle * row as f64 / (n - 1) as f64,
                );
            }
            frames[n - 1] = frames[0];
        }

        Ok(Self {
            tangents,
            normals: frames,
        })
    }
}
