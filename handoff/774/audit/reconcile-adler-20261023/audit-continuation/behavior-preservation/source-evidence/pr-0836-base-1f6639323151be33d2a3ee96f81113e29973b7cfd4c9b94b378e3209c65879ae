//! Shared skin domain for torso fitting and body-relative equipment support.
//!
//! Eligibility identifies candidate torso faces. It does not identify a bearing
//! footprint, an anatomical attachment, or a permitted sliding region.
use anyhow::{Result, ensure};

const TORSO_JOINTS: [&str; 8] = [
    "root",
    "c_spine0",
    "c_spine1",
    "c_spine2",
    "c_spine3",
    "c_neck",
    "l_clavicle",
    "r_clavicle",
];
const LIMB_JOINTS: [&str; 6] = ["uparm", "loarm", "hand", "upleg", "loleg", "foot"];
const MINIMUM_MEAN_TORSO_WEIGHT: f32 = 0.08;
const MAXIMUM_MEAN_LIMB_WEIGHT: f32 = 0.35;

/// Canonical body topology and skin ownership, before any equipment is fitted.
pub struct TorsoSkinDomain<'a> {
    pub faces: &'a [[u32; 3]],
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
    pub joint_names: &'a [String],
}

impl TorsoSkinDomain<'_> {
    /// Original body triangle identities, in input order, including duplicate
    /// faces. Device fitting and posed support must retain the same identities.
    pub fn triangle_indices(&self) -> Result<Vec<u32>> {
        ensure!(
            self.joint_indices.len() == self.joint_weights.len()
                && !self.joint_names.is_empty()
                && self.faces.len() <= u32::MAX as usize
                && self
                    .faces
                    .iter()
                    .flatten()
                    .all(|&vertex| { (vertex as usize) < self.joint_indices.len() }),
            "torso skin domain has inconsistent topology"
        );
        let torso = self
            .joint_names
            .iter()
            .map(|name| TORSO_JOINTS.contains(&name.as_str()));
        let limbs = self
            .joint_names
            .iter()
            .map(|name| LIMB_JOINTS.iter().any(|limb| name.contains(limb)));
        let ownership: Vec<_> = torso.zip(limbs).collect();
        let weights = self
            .joint_indices
            .iter()
            .zip(self.joint_weights)
            .map(|(indices, weights)| {
                let mut totals = [0.0; 2];
                for (&joint, &weight) in indices.iter().zip(weights) {
                    ensure!(
                        weight.is_finite() && (0.0..=1.0).contains(&weight),
                        "invalid torso skin weight"
                    );
                    let Some(&(torso, limb)) = ownership.get(joint as usize) else {
                        anyhow::bail!("torso skin references a missing joint");
                    };
                    if torso {
                        totals[0] += weight;
                    }
                    if limb {
                        totals[1] += weight;
                    }
                }
                Ok(totals)
            })
            .collect::<Result<Vec<_>>>()?;
        let selected: Vec<_> = self
            .faces
            .iter()
            .enumerate()
            .filter_map(|(index, face)| {
                let mean = [0, 1].map(|side| {
                    face.iter()
                        .map(|&vertex| weights[vertex as usize][side])
                        .sum::<f32>()
                        / 3.0
                });
                (mean[0] >= MINIMUM_MEAN_TORSO_WEIGHT && mean[1] <= MAXIMUM_MEAN_LIMB_WEIGHT)
                    .then_some(index as u32)
            })
            .collect();
        ensure!(
            !selected.is_empty(),
            "torso section face selection is empty"
        );
        Ok(selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_original_face_identity_and_excludes_limb_dominated_skin() {
        let names = vec!["c_spine3".into(), "l_uparm".into()];
        let indices = [[0; 8], [0; 8], [0; 8], [1; 8], [1; 8], [1; 8]];
        let weights = [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 6];
        let domain = TorsoSkinDomain {
            faces: &[[3, 4, 5], [0, 1, 2], [0, 1, 2]],
            joint_indices: &indices,
            joint_weights: &weights,
            joint_names: &names,
        };
        assert_eq!(domain.triangle_indices().unwrap(), vec![1, 2]);
    }

    #[test]
    fn malformed_skin_and_missing_support_return_errors() {
        let names = vec!["l_hand".into()];
        let mut indices = [[0; 8]; 3];
        let mut weights = [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 3];
        let selected = |faces: &[[u32; 3]], indices: &[[u32; 8]], weights: &[[f32; 8]]| {
            TorsoSkinDomain {
                faces,
                joint_indices: indices,
                joint_weights: weights,
                joint_names: &names,
            }
            .triangle_indices()
        };
        assert!(selected(&[[0, 1, 2]], &indices, &weights).is_err());
        assert!(selected(&[[0, 1, 3]], &indices, &weights).is_err());
        indices[0][0] = 1;
        assert!(selected(&[[0, 1, 2]], &indices, &weights).is_err());
        indices[0][0] = 0;
        weights[0][0] = f32::NAN;
        assert!(selected(&[[0, 1, 2]], &indices, &weights).is_err());
    }

    #[test]
    fn mixed_shoulder_skin_keeps_the_fitting_domains_thresholds() {
        let names = vec!["c_spine3".into(), "l_uparm".into(), "c_head".into()];
        let indices = [[0, 1, 2, 0, 0, 0, 0, 0]; 3];
        let selected = |torso: f32, limb: f32| {
            let weights = [[torso, limb, 1.0 - torso - limb, 0.0, 0.0, 0.0, 0.0, 0.0]; 3];
            TorsoSkinDomain {
                faces: &[[0, 1, 2]],
                joint_indices: &indices,
                joint_weights: &weights,
                joint_names: &names,
            }
            .triangle_indices()
        };
        assert_eq!(selected(0.08, 0.35).unwrap(), vec![0]);
        assert!(selected(0.079, 0.35).is_err());
        assert!(selected(0.08, 0.351).is_err());
    }
}
