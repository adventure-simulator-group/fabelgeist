//! Optional cloth ownership and native Bevy skin layouts.

use super::*;

/// Unweighted cloth owns geometry and dynamics without any skin payload.
/// A retained weighted binding may be ignored while physics is unweighted.
pub enum ClothBinding {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "unweighted simulation is covered by tests; the current preview supplies weighted cloth"
        )
    )]
    Unweighted,
    Weighted {
        indices: fabelgeist_rig::PackedRigSkin<8>,
        weights: Vec<[f32; 8]>,
    },
}

pub fn skin_mesh(
    mesh: &mut Mesh,
    indices: &fabelgeist_rig::PackedRigSkin<8>,
    weights: &[[f32; 8]],
) -> Result<()> {
    let mut joint_indices = Vec::with_capacity(indices.len());
    let mut joint_weights = Vec::with_capacity(weights.len());
    for (indices, weights) in indices.iter().zip(weights) {
        let mut influences: Vec<_> = indices
            .iter()
            .copied()
            .zip(weights.iter().copied())
            .collect();
        influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        let mut ids = [0u16; 4];
        for (word, influence) in ids.iter_mut().zip(&influences) {
            *word = u16::try_from(usize::from(influence.0))?;
        }
        let mut values = std::array::from_fn::<_, 4, _>(|i| influences[i].1);
        let total: f32 = values.iter().sum();
        if total > 0.0 {
            for weight in &mut values {
                *weight /= total;
            }
        } else {
            values[0] = 1.0;
        }
        joint_indices.push(ids);
        joint_weights.push(values);
    }
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(joint_indices),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights);
    Ok(())
}

pub fn skin(preview: &WalkPreview) -> Option<SkinnedMesh> {
    (!preview.joints.is_empty()).then(|| SkinnedMesh {
        inverse_bindposes: preview.inverse_bindposes.clone(),
        joints: preview.joints.as_native_slice().to_vec(),
    })
}
