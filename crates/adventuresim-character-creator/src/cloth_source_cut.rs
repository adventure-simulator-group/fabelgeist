//! Validate the Boolean cut evaluated through frozen packed point ancestry.
//! Boundary indices name packed rows; positions are canonical unposed metres.
use crate::surface_cut::POINT_WORDS;
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;

pub(super) fn validate(
    table: &[u32],
    faces: &[[u32; 3]],
    before: &[[f32; 3]],
    after: &[[f32; 3]],
) -> Result<()> {
    ensure!(
        table.len().is_multiple_of(POINT_WORDS),
        "partial cloth source record"
    );
    let rows = table.as_chunks::<POINT_WORDS>().0;
    for &face in faces {
        let prior = evaluate_face(rows, face, before)?;
        let fitted = evaluate_face(rows, face, after)?;
        let [a, b, c] = prior;
        let [next_a, next_b, next_c] = fitted;
        ensure!(
            (b - a)
                .cross(c - a)
                .dot((next_b - next_a).cross(next_c - next_a))
                > 0.,
            "inner layer fitting folds an evaluated cloth cut: points={face:?} before={prior:?} after={fitted:?}"
        );
    }
    Ok(())
}

fn evaluate_face(
    rows: &[[u32; POINT_WORDS]],
    face: [u32; 3],
    positions: &[[f32; 3]],
) -> Result<[Vec3; 3]> {
    let mut result = [Vec3::ZERO; 3];
    for (point, row) in result.iter_mut().zip(face) {
        let row = rows
            .get(row as usize)
            .context("cloth cut has no packed source row")?;
        for corner in 0..3 {
            let position = positions
                .get(row[corner] as usize)
                .context("cloth cut source vertex is out of range")?;
            // Packed native rows contain position indices, UV indices, then
            // three barycentric f32 weights. Skin still uses original indices.
            *point += Vec3::from_array(*position) * f32::from_bits(row[6 + corner]);
        }
        ensure!(
            point.is_finite(),
            "cloth cut has a nonfinite evaluated point"
        );
    }
    Ok(result)
}
