//! Bevy consumes one VEC4 skin set; discarded influences must not lose mass.
use super::*;

pub fn strongest_four(joints: [u32; 8], weights: [f32; 8]) -> ([u16; 4], [f32; 4]) {
    let mut combined = std::collections::BTreeMap::<u32, f32>::new();
    for (joint, weight) in joints.into_iter().zip(weights) {
        *combined.entry(joint).or_default() += weight;
    }
    let mut influences = combined.into_iter().collect::<Vec<_>>();
    influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    influences.truncate(4);
    let sum: f32 = influences.iter().map(|(_, weight)| weight).sum();
    let mut joints = [0; 4];
    let mut weights = [0.; 4];
    for (i, (joint, weight)) in influences.into_iter().enumerate() {
        joints[i] = joint as u16;
        weights[i] = weight / sum;
    }
    (joints, weights)
}

pub(super) fn append(
    buffer: &mut BufferBuilder,
    joints: &[[u32; 8]],
    weights: &[[f32; 8]],
) -> (usize, usize) {
    let skin = joints
        .iter()
        .zip(weights)
        .map(|(j, w)| strongest_four(*j, *w))
        .collect::<Vec<_>>();
    let joints = buffer.push(&u16_bytes(skin.iter().flat_map(|(j, _)| *j)), Some(34_962));
    let weights = buffer.push(&f32_bytes(skin.iter().flat_map(|(_, w)| *w)), Some(34_962));
    (joints, weights)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_minor_influences_preserves_world_translation() {
        let (joints, weights) = strongest_four(
            [0, 1, 2, 3, 4, 5, 6, 7],
            [0.05, 0.2, 0.1, 0.05, 0.3, 0.15, 0.1, 0.05],
        );
        assert_eq!(joints, [4, 1, 5, 2]);
        // The runtime uses the weighted xyz directly. A common translation
        // must move the vertex by that whole distance, even far from origin.
        let position = [0.2, 1.3, -0.1];
        let translation = [20., -8., 50.];
        let actual: [f32; 3] = std::array::from_fn(|axis| {
            weights
                .iter()
                .map(|w| w * (position[axis] + translation[axis]))
                .sum()
        });
        for axis in 0..3 {
            assert!((actual[axis] - position[axis] - translation[axis]).abs() < 1e-5);
        }
    }
}
