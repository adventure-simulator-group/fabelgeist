//! Mixamo's finger spelling, with shared side and humanoid-chain authority.
use super::super::semantic::{HumanoidChain, HumanoidJoint};
use fabelgeist_rig::{RigJointName, RigSide};

#[derive(Clone, Copy)]
pub(super) enum MixamoFinger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}
#[derive(Clone, Copy)]
pub(super) enum MixamoFingerSegment {
    Proximal,
    Intermediate,
    Distal,
}
impl MixamoFinger {
    pub const ALL: [Self; 5] = [
        Self::Thumb,
        Self::Index,
        Self::Middle,
        Self::Ring,
        Self::Pinky,
    ];
    pub fn segments(self, side: RigSide) -> [(MixamoFingerSegment, HumanoidJoint); 3] {
        use HumanoidChain::*;
        let chain = match (side, self) {
            (RigSide::Left, Self::Thumb) => ThumbLeft,
            (RigSide::Left, Self::Index) => IndexLeft,
            (RigSide::Left, Self::Middle) => MiddleLeft,
            (RigSide::Left, Self::Ring) => RingLeft,
            (RigSide::Left, Self::Pinky) => LittleLeft,
            (RigSide::Right, Self::Thumb) => ThumbRight,
            (RigSide::Right, Self::Index) => IndexRight,
            (RigSide::Right, Self::Middle) => MiddleRight,
            (RigSide::Right, Self::Ring) => RingRight,
            (RigSide::Right, Self::Pinky) => LittleRight,
        };
        let roles = chain.joints();
        [
            (MixamoFingerSegment::Proximal, roles[0]),
            (MixamoFingerSegment::Intermediate, roles[1]),
            (MixamoFingerSegment::Distal, roles[2]),
        ]
    }
    pub fn name(self, side: RigSide, segment: MixamoFingerSegment) -> RigJointName {
        let side = match side {
            RigSide::Left => "Left",
            RigSide::Right => "Right",
        };
        let finger = match self {
            Self::Thumb => "Thumb",
            Self::Index => "Index",
            Self::Middle => "Middle",
            Self::Ring => "Ring",
            Self::Pinky => "Pinky",
        };
        let segment = match segment {
            MixamoFingerSegment::Proximal => "1",
            MixamoFingerSegment::Intermediate => "2",
            MixamoFingerSegment::Distal => "3",
        };
        RigJointName::from(format!("mixamorig:{side}Hand{finger}{segment}"))
    }
}
