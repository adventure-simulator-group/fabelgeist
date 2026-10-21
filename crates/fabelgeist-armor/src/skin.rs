//! Skin weights of a vertex blended from others.

use fabelgeist_rig::{RigInfluences, RigJointOrdinal};

pub(crate) struct SkinShare {
    pub skin: RigInfluences<8>,
    pub share: f32,
}

struct Influence {
    joint: RigJointOrdinal,
    weight: f32,
}

/// The heaviest eight joints of `skins`, each weighted by its share,
/// renormalised. With no weight at all, the first skin.
pub(crate) fn blend(first: SkinShare, remaining: &[SkinShare]) -> RigInfluences<8> {
    let mut merged = Vec::<Influence>::with_capacity((1 + remaining.len()) * 8);
    for SkinShare { skin, share } in std::iter::once(&first).chain(remaining) {
        for (joint, weight) in skin.joints.iter().zip(&skin.weights) {
            let weight = weight * share;
            if weight <= 0.0 {
                continue;
            }
            match merged
                .iter_mut()
                .find(|influence| influence.joint == *joint)
            {
                Some(influence) => influence.weight += weight,
                None => merged.push(Influence {
                    joint: *joint,
                    weight,
                }),
            }
        }
    }
    merged.sort_by(|x, y| y.weight.total_cmp(&x.weight).then(x.joint.cmp(&y.joint)));
    merged.truncate(8);
    let total = merged.iter().map(|influence| influence.weight).sum::<f32>();
    if total <= 0.0 {
        return first.skin;
    }
    let mut joints = [RigJointOrdinal::from(0usize); 8];
    let mut weights = [0.0f32; 8];
    for (slot, influence) in merged.into_iter().enumerate() {
        joints[slot] = influence.joint;
        weights[slot] = influence.weight / total;
    }
    RigInfluences { joints, weights }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single(joint: u32) -> RigInfluences<8> {
        let mut joints = [RigJointOrdinal::from(0usize); 8];
        let mut weights = [0.0; 8];
        joints[0] = joint.into();
        weights[0] = 1.0;
        RigInfluences { joints, weights }
    }

    #[test]
    fn shares_split_the_weight_heaviest_first() {
        let skin = blend(
            SkinShare {
                skin: single(3),
                share: 0.25,
            },
            &[SkinShare {
                skin: single(7),
                share: 0.75,
            }],
        );
        assert_eq!(
            &skin.joints[..2],
            &[RigJointOrdinal::from(7u32), RigJointOrdinal::from(3u32)]
        );
        assert_eq!(&skin.weights[..2], &[0.75, 0.25]);
    }

    #[test]
    fn no_weight_keeps_the_first_skin() {
        assert_eq!(
            blend(
                SkinShare {
                    skin: single(3),
                    share: 0.0
                },
                &[SkinShare {
                    skin: single(7),
                    share: 0.0
                }]
            ),
            single(3)
        );
    }
}
