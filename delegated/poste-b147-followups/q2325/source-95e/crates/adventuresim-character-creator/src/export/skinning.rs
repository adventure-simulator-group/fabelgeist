//! Bevy consumes one VEC4 skin set; discarded influences must not lose mass.
use super::*;

pub(super) fn append(
    buffer: &mut BufferBuilder,
    joints: &fabelgeist_rig::PackedRigSkin<8>,
    weights: &[[f32; 8]],
) -> Result<(usize, usize)> {
    let skin = joints
        .iter()
        .zip(weights)
        .map(|(joints, weights)| {
            fabelgeist_rig::RigInfluences {
                joints,
                weights: *weights,
            }
            .strongest_four()
        })
        .collect::<Vec<_>>();
    let native_joints = skin
        .iter()
        .map(|skin| skin.native_u16())
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let joints = buffer.push(
        &u16_bytes(native_joints.into_iter().flatten()),
        Some(34_962),
    );
    let weights = buffer.push(
        &f32_bytes(skin.iter().flat_map(|skin| skin.weights)),
        Some(34_962),
    );
    Ok((joints, weights))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_minor_influences_preserves_world_translation() {
        let skin = fabelgeist_rig::RigInfluences {
            joints: [0usize, 1, 2, 3, 4, 5, 6, 7].map(RigJointOrdinal::from),
            weights: [0.05, 0.2, 0.1, 0.05, 0.3, 0.15, 0.1, 0.05],
        }
        .strongest_four();
        let joints = skin.native_u16().unwrap();
        let weights = skin.weights;
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
