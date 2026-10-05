//! Which parts of the wearer a garment dresses: the parts its cloth may be
//! drawn onto. A tunic's hem hangs past the thighs and a sleeve may reach
//! over the hand, but neither should be wrapped around them.
use super::*;
use fabelgeist_rig::{RigJointMembership, RigJointName, RigJointPart};

/// A part of the body, by the joint that moves its skin most.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BodyPart {
    /// Pelvis, spine, neck, shoulders and arms down to the wrist.
    Trunk,
    Head,
    Hand,
    Leg,
    Foot,
}

impl BodyPart {
    /// MHR joints are named by side and part, such as `l_upleg` or `r_index1`.
    fn of_joint(name: &RigJointName) -> Self {
        const HEAD: [RigJointPart; 4] = [
            RigJointPart::Head,
            RigJointPart::Jaw,
            RigJointPart::Eye,
            RigJointPart::Tongue,
        ];
        const HAND: [RigJointPart; 6] = [
            RigJointPart::Wrist,
            RigJointPart::Thumb,
            RigJointPart::Index,
            RigJointPart::Middle,
            RigJointPart::Ring,
            RigJointPart::Pinky,
        ];
        const LEG: [RigJointPart; 2] = [RigJointPart::Upleg, RigJointPart::Lowleg];
        const FOOT: [RigJointPart; 5] = [
            RigJointPart::Foot,
            RigJointPart::Ball,
            RigJointPart::Toe,
            RigJointPart::Talocrural,
            RigJointPart::Subtalar,
        ];
        let contains = |parts: &[RigJointPart]| -> RigJointMembership {
            RigJointMembership::from(parts.iter().any(|part: &RigJointPart| -> bool {
                name.contains_part(*part) == RigJointMembership::Included
            }))
        };
        if contains(&HEAD) == RigJointMembership::Included {
            Self::Head
        } else if contains(&HAND) == RigJointMembership::Included {
            Self::Hand
        } else if contains(&FOOT) == RigJointMembership::Included {
            Self::Foot
        } else if contains(&LEG) == RigJointMembership::Included {
            Self::Leg
        } else {
            Self::Trunk
        }
    }

    fn dressed_by(self, form: GarmentForm) -> bool {
        match self {
            Self::Trunk => true,
            Self::Leg => form == GarmentForm::Legged,
            Self::Head => form == GarmentForm::Fitted,
            Self::Hand | Self::Foot => false,
        }
    }
}

/// The wearer's surface, and for each of its vertices whether the garment
/// dresses the body there.
pub(super) struct Dressing {
    body: fabelgeist_bvh::TriangleBvh,
    faces: Vec<[u32; 3]>,
    dressed: Vec<bool>,
}

impl Dressing {
    pub(super) fn new(input: &DrapeInput) -> Self {
        let form = input.selection.form();
        let parts: Vec<bool> = input
            .names
            .iter()
            .map(|name| BodyPart::of_joint(name).dressed_by(form))
            .collect();
        let dressed = input
            .indices
            .iter()
            .zip(&input.weights)
            .map(|(joints, weights)| {
                let (joint, _) = joints
                    .iter()
                    .zip(weights)
                    .max_by(|a, b| a.1.total_cmp(b.1))
                    .expect("skin influences are never empty");
                parts.get(*joint as usize).copied().unwrap_or(true)
            })
            .collect();
        Self {
            body: fabelgeist_bvh::TriangleBvh::new(
                input.positions.iter().copied().map(vector).collect(),
                input.faces.clone(),
            ),
            faces: input.faces.clone(),
            dressed,
        }
    }

    /// Whether the body nearest to `point` is dressed by the garment.
    pub(super) fn dresses(&self, point: Vec3) -> bool {
        let Some((triangle, closest, _)) = self.body.closest_point(point, f32::MAX) else {
            return false;
        };
        let corners = self.faces[triangle as usize];
        let nearest = corners
            .iter()
            .min_by(|a, b| {
                let distance = |i: &u32| (self.body.positions[*i as usize] - closest).length();
                distance(a).total_cmp(&distance(b))
            })
            .expect("a triangle has corners");
        self.dressed[*nearest as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_trunk_and_the_parts_a_cut_covers_are_dressed() {
        let part = BodyPart::of_joint;
        assert_eq!(part(&RigJointName::C_SPINE2), BodyPart::Trunk);
        assert_eq!(part(&RigJointName::L_LOWARM_TWIST), BodyPart::Trunk);
        assert_eq!(part(&RigJointName::R_THUMB1), BodyPart::Hand);
        assert_eq!(part(&RigJointName::L_UPLEG_TWIST), BodyPart::Leg);
        assert_eq!(part(&RigJointName::R_FOOT), BodyPart::Foot);
        assert_eq!(part(&RigJointName::C_HEAD), BodyPart::Head);
        assert!(!BodyPart::Leg.dressed_by(GarmentForm::Upper));
        assert!(BodyPart::Leg.dressed_by(GarmentForm::Legged));
        assert!(!BodyPart::Hand.dressed_by(GarmentForm::Upper));
        assert!(BodyPart::Head.dressed_by(GarmentForm::Fitted));
    }
}
