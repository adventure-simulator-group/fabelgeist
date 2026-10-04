use crate::skeleton::SkinJointOrdinal;
use fabelgeist_rig::{RigJointMembership, RigJointName, RigJointOrdinal, RigJointPrefix};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MixamoRig;

pub trait JointBuilder {
    fn build() -> Self;
}

pub struct Joint {
    name: RigJointName,
    joints: Vec<Joint>,
}

impl Joint {
    pub fn new(name: RigJointName) -> Self {
        let joints = Default::default();
        Self { name, joints }
    }

    pub fn with_joint(mut self, joint: Self) -> Self {
        self.joints.push(joint);
        self
    }

    pub fn child(&mut self, name: RigJointName) -> &mut Self {
        let joint = Joint::new(name);
        self.joints.push(joint);
        self.joints.last_mut().unwrap()
    }

    pub fn flatten(&self) -> Vec<crate::skeleton::Joint> {
        let mut joints = Vec::new();
        self.flatten_recursive(None, &mut joints);
        joints
    }

    fn flatten_recursive(
        &self,
        parent_index: Option<RigJointOrdinal>,
        result: &mut Vec<crate::skeleton::Joint>,
    ) {
        let index = RigJointOrdinal::from(result.len());
        result.push(crate::skeleton::Joint::new(
            self.name.clone(),
            index,
            parent_index,
            fabelgeist_math::matrix::Mat4::identity(),
            Default::default(),
            Some(SkinJointOrdinal::from(index)),
        ));
        for child in &self.joints {
            child.flatten_recursive(Some(index), result);
        }
    }
}

macro_rules! joint {
    ($name:expr, ()) => {
        Joint::new(RigJointName::from($name))
    };
    ($name:expr, ($($child:tt),* $(,)?)) => {
        Joint::new(RigJointName::from($name))
            $(.with_joint(joint! $child))*
    };
    (($name:expr, $children:tt)) => {
        joint!($name, $children)
    };
}

impl JointBuilder for Joint {
    fn build() -> Self {
        joint!(
            "mixamorig:Hips",
            (
                (
                    "mixamorig:LeftUpLeg",
                    ((
                        "mixamorig:LeftLeg",
                        ((
                            "mixamorig:LeftFoot",
                            (("mixamorig:LeftToeBase", (("mixamorig:LeftToe_End", ()))))
                        ))
                    ))
                ),
                (
                    "mixamorig:RightUpLeg",
                    ((
                        "mixamorig:RightLeg",
                        ((
                            "mixamorig:RightFoot",
                            (("mixamorig:RightToeBase", (("mixamorig:RightToe_End", ()))))
                        ))
                    ))
                ),
                (
                    "mixamorig:Spine",
                    ((
                        "mixamorig:Spine1",
                        ((
                            "mixamorig:Spine2",
                            (
                                (
                                    "mixamorig:Neck",
                                    (("mixamorig:Head", (("mixamorig:HeadTop_End", ()))))
                                ),
                                (
                                    "mixamorig:LeftShoulder",
                                    ((
                                        "mixamorig:LeftArm",
                                        ((
                                            "mixamorig:LeftForeArm",
                                            ((
                                                "mixamorig:LeftHand",
                                                (
                                                    (
                                                        "mixamorig:LeftHandThumb1",
                                                        ((
                                                            "mixamorig:LeftHandThumb2",
                                                            ((
                                                                "mixamorig:LeftHandThumb3",
                                                                (("mixamorig:LeftHandThumb4", ()))
                                                            ))
                                                        ))
                                                    ),
                                                    (
                                                        "mixamorig:LeftHandIndex1",
                                                        ((
                                                            "mixamorig:LeftHandIndex2",
                                                            ((
                                                                "mixamorig:LeftHandIndex3",
                                                                (("mixamorig:LeftHandIndex4", ()))
                                                            ))
                                                        ))
                                                    ),
                                                    (
                                                        "mixamorig:LeftHandMiddle1",
                                                        ((
                                                            "mixamorig:LeftHandMiddle2",
                                                            ((
                                                                "mixamorig:LeftHandMiddle3",
                                                                (("mixamorig:LeftHandMiddle4", ()))
                                                            ))
                                                        ))
                                                    ),
                                                    (
                                                        "mixamorig:LeftHandRing1",
                                                        ((
                                                            "mixamorig:LeftHandRing2",
                                                            ((
                                                                "mixamorig:LeftHandRing3",
                                                                (("mixamorig:LeftHandRing4", ()))
                                                            ))
                                                        ))
                                                    ),
                                                    (
                                                        "mixamorig:LeftHandPinky1",
                                                        ((
                                                            "mixamorig:LeftHandPinky2",
                                                            ((
                                                                "mixamorig:LeftHandPinky3",
                                                                (("mixamorig:LeftHandPinky4", ()))
                                                            ))
                                                        ))
                                                    )
                                                )
                                            ))
                                        ))
                                    ))
                                ),
                                (
                                    "mixamorig:RightShoulder",
                                    ((
                                        "mixamorig:RightArm",
                                        ((
                                            "mixamorig:RightForeArm",
                                            ((
                                                "mixamorig:RightHand",
                                                (
                                                    (
                                                        "mixamorig:RightHandThumb1",
                                                        ((
                                                            "mixamorig:RightHandThumb2",
                                                            ((
                                                                "mixamorig:RightHandThumb3",
                                                                (("mixamorig:RightHandThumb4", ()))
                                                            ))
                                                        ))
                                                    ),
                                                    (
                                                        "mixamorig:RightHandIndex1",
                                                        ((
                                                            "mixamorig:RightHandIndex2",
                                                            ((
                                                                "mixamorig:RightHandIndex3",
                                                                (("mixamorig:RightHandIndex4", ()))
                                                            ))
                                                        ))
                                                    ),
                                                    (
                                                        "mixamorig:RightHandMiddle1",
                                                        ((
                                                            "mixamorig:RightHandMiddle2",
                                                            ((
                                                                "mixamorig:RightHandMiddle3",
                                                                ((
                                                                    "mixamorig:RightHandMiddle4",
                                                                    ()
                                                                ))
                                                            ))
                                                        ))
                                                    ),
                                                    (
                                                        "mixamorig:RightHandRing1",
                                                        ((
                                                            "mixamorig:RightHandRing2",
                                                            ((
                                                                "mixamorig:RightHandRing3",
                                                                (("mixamorig:RightHandRing4", ()))
                                                            ))
                                                        ))
                                                    ),
                                                    (
                                                        "mixamorig:RightHandPinky1",
                                                        ((
                                                            "mixamorig:RightHandPinky2",
                                                            ((
                                                                "mixamorig:RightHandPinky3",
                                                                (("mixamorig:RightHandPinky4", ()))
                                                            ))
                                                        ))
                                                    )
                                                )
                                            ))
                                        ))
                                    ))
                                )
                            )
                        ))
                    ))
                )
            )
        )
    }
}

impl MixamoRig {
    pub const JOINT_NAMES: &[RigJointName] = &[
        RigJointName::from_static("mixamorig:Hips"),
        RigJointName::from_static("mixamorig:Spine"),
        RigJointName::from_static("mixamorig:Spine1"),
        RigJointName::from_static("mixamorig:Spine2"),
        RigJointName::from_static("mixamorig:Neck"),
        RigJointName::from_static("mixamorig:Head"),
        RigJointName::from_static("mixamorig:LeftShoulder"),
        RigJointName::from_static("mixamorig:LeftArm"),
        RigJointName::from_static("mixamorig:LeftForeArm"),
        RigJointName::from_static("mixamorig:LeftHand"),
        RigJointName::from_static("mixamorig:RightShoulder"),
        RigJointName::from_static("mixamorig:RightArm"),
        RigJointName::from_static("mixamorig:RightForeArm"),
        RigJointName::from_static("mixamorig:RightHand"),
        RigJointName::from_static("mixamorig:LeftUpLeg"),
        RigJointName::from_static("mixamorig:LeftLeg"),
        RigJointName::from_static("mixamorig:LeftFoot"),
        RigJointName::from_static("mixamorig:LeftToeBase"),
        RigJointName::from_static("mixamorig:RightUpLeg"),
        RigJointName::from_static("mixamorig:RightLeg"),
        RigJointName::from_static("mixamorig:RightFoot"),
        RigJointName::from_static("mixamorig:RightToeBase"),
        // Fingers - Left
        RigJointName::from_static("mixamorig:LeftHandThumb1"),
        RigJointName::from_static("mixamorig:LeftHandThumb2"),
        RigJointName::from_static("mixamorig:LeftHandThumb3"),
        RigJointName::from_static("mixamorig:LeftHandIndex1"),
        RigJointName::from_static("mixamorig:LeftHandIndex2"),
        RigJointName::from_static("mixamorig:LeftHandIndex3"),
        RigJointName::from_static("mixamorig:LeftHandMiddle1"),
        RigJointName::from_static("mixamorig:LeftHandMiddle2"),
        RigJointName::from_static("mixamorig:LeftHandMiddle3"),
        RigJointName::from_static("mixamorig:LeftHandRing1"),
        RigJointName::from_static("mixamorig:LeftHandRing2"),
        RigJointName::from_static("mixamorig:LeftHandRing3"),
        RigJointName::from_static("mixamorig:LeftHandPinky1"),
        RigJointName::from_static("mixamorig:LeftHandPinky2"),
        RigJointName::from_static("mixamorig:LeftHandPinky3"),
        // Fingers - Right
        RigJointName::from_static("mixamorig:RightHandThumb1"),
        RigJointName::from_static("mixamorig:RightHandThumb2"),
        RigJointName::from_static("mixamorig:RightHandThumb3"),
        RigJointName::from_static("mixamorig:RightHandIndex1"),
        RigJointName::from_static("mixamorig:RightHandIndex2"),
        RigJointName::from_static("mixamorig:RightHandIndex3"),
        RigJointName::from_static("mixamorig:RightHandMiddle1"),
        RigJointName::from_static("mixamorig:RightHandMiddle2"),
        RigJointName::from_static("mixamorig:RightHandMiddle3"),
        RigJointName::from_static("mixamorig:RightHandRing1"),
        RigJointName::from_static("mixamorig:RightHandRing2"),
        RigJointName::from_static("mixamorig:RightHandRing3"),
        RigJointName::from_static("mixamorig:RightHandPinky1"),
        RigJointName::from_static("mixamorig:RightHandPinky2"),
        RigJointName::from_static("mixamorig:RightHandPinky3"),
    ];

    pub fn is_mixamo_joint(name: &RigJointName) -> RigJointMembership {
        RigJointPrefix::MIXAMO.membership(name)
    }

    pub fn strip_prefix(name: &RigJointName) -> RigJointName {
        RigJointPrefix::MIXAMO.remove_from(name)
    }

    pub fn skeleton() -> crate::skeleton::Skeleton {
        let root = Joint::build();
        crate::skeleton::Skeleton::new(root.flatten())
    }
}
