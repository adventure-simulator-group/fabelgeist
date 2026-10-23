//! Shared shell directions must point outside every incident source face.
//! Numerical kernel: unit face planes and native unposed metre positions.
use anyhow::{Context, Result, ensure};
use bevy::math::{DVec3, Vec3};

/// One part per million bounds roundoff in the small face-plane solves.
const PLANE_SOLVE_ROUNDING: f64 = 0.000001;

pub(super) fn outward(points: &[Vec3], faces: &[[u32; 3]]) -> Result<Vec<Vec3>> {
    Ok(clearance_vectors(points, faces)?
        .into_iter()
        .map(|direction| direction.normalize())
        .collect())
}

/// Minimum-length displacement with one metre of clearance from every
/// incident unit face plane; callers supply the actual dimensional clearance.
pub(super) fn clearance_vectors(points: &[Vec3], faces: &[[u32; 3]]) -> Result<Vec<Vec3>> {
    let mut incident = vec![Vec::new(); points.len()];
    for face in faces {
        let [a, b, c] = face.map(|v| points[v as usize].as_dvec3());
        let normal = (b - a)
            .cross(c - a)
            .try_normalize()
            .context("panel refit folded a triangle")?;
        for &vertex in face {
            incident[vertex as usize].push(normal);
        }
    }
    incident
        .iter()
        .enumerate()
        .map(|(vertex, faces)| {
            let mut best = None;
            let mut admit = |candidate: DVec3| {
                if candidate.is_finite()
                    && faces
                        .iter()
                        .all(|normal| normal.dot(candidate) >= 1. - PLANE_SOLVE_ROUNDING)
                    && best.is_none_or(|previous: DVec3| {
                        candidate.length_squared() < previous.length_squared()
                    })
                {
                    best = Some(candidate);
                }
            };
            for (i, &a) in faces.iter().enumerate() {
                admit(a);
                for (j, &b) in faces.iter().enumerate().skip(i + 1) {
                    let denominator = 1. + a.dot(b);
                    if denominator > 0. {
                        admit((a + b) / denominator);
                    }
                    for &c in faces.iter().skip(j + 1) {
                        let denominator = a.dot(b.cross(c));
                        if denominator != 0. {
                            admit((b.cross(c) + c.cross(a) + a.cross(b)) / denominator);
                        }
                    }
                }
            }
            let direction = best
                .with_context(|| format!("panel vertex {vertex} has no outward shell direction"))?
                .as_vec3();
            ensure!(direction.is_finite(), "invalid panel shell direction");
            Ok(direction)
        })
        .collect()
}
