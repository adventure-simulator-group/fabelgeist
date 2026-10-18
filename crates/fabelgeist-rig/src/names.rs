//! Authored label vocabulary; unfamiliar labels remain valid.

use super::{Cow, RigJointName};
use strum::VariantNames;

#[derive(VariantNames)]
enum AuthoredJoint {
    #[strum(serialize = "Skeleton")]
    Skeleton,
    #[strum(serialize = "body_world")]
    BodyWorld,
    #[strum(serialize = "c_camera")]
    CCamera,
    #[strum(serialize = "c_head")]
    CHead,
    #[strum(serialize = "c_jaw")]
    CJaw,
    #[strum(serialize = "c_jaw_null")]
    CJawNull,
    #[strum(serialize = "c_neck")]
    CNeck,
    #[strum(serialize = "c_neck_twist")]
    CNeckTwist,
    #[strum(serialize = "c_spine0")]
    CSpine0,
    #[strum(serialize = "c_spine1")]
    CSpine1,
    #[strum(serialize = "c_spine2")]
    CSpine2,
    #[strum(serialize = "c_spine3")]
    CSpine3,
    #[strum(serialize = "c_tongue")]
    CTongue,
    #[strum(serialize = "l_ball")]
    LBall,
    #[strum(serialize = "l_clavicle")]
    LClavicle,
    #[strum(serialize = "l_eye")]
    LEye,
    #[strum(serialize = "l_foot")]
    LFoot,
    #[strum(serialize = "l_index1")]
    LIndex1,
    #[strum(serialize = "l_index2")]
    LIndex2,
    #[strum(serialize = "l_lowarm")]
    LLowarm,
    #[strum(serialize = "l_lowarm_twist")]
    LLowarmTwist,
    #[strum(serialize = "l_lowleg")]
    LLowleg,
    #[strum(serialize = "l_lowleg_twist")]
    LLowlegTwist,
    #[strum(serialize = "l_middle1")]
    LMiddle1,
    #[strum(serialize = "l_middle_null")]
    LMiddleNull,
    #[strum(serialize = "l_pinky_null")]
    LPinkyNull,
    #[strum(serialize = "l_ring1")]
    LRing1,
    #[strum(serialize = "l_subtalar")]
    LSubtalar,
    #[strum(serialize = "l_talocrural")]
    LTalocrural,
    #[strum(serialize = "l_thumb")]
    LThumb,
    #[strum(serialize = "l_thumb0")]
    LThumb0,
    #[strum(serialize = "l_thumb1")]
    LThumb1,
    #[strum(serialize = "l_transversetarsal")]
    LTransversetarsal,
    #[strum(serialize = "l_uparm")]
    LUparm,
    #[strum(serialize = "l_uparm_twist")]
    LUparmTwist,
    #[strum(serialize = "l_uparm_twist0_proc")]
    LUparmTwist0Proc,
    #[strum(serialize = "l_uparm_twist3_proc")]
    LUparmTwist3Proc,
    #[strum(serialize = "l_upleg")]
    LUpleg,
    #[strum(serialize = "l_upleg_twist")]
    LUplegTwist,
    #[strum(serialize = "l_upleg_twist2_proc")]
    LUplegTwist2Proc,
    #[strum(serialize = "l_upperarm")]
    LUpperarm,
    #[strum(serialize = "l_weapon")]
    LWeapon,
    #[strum(serialize = "l_wrist")]
    LWrist,
    #[strum(serialize = "r_ball")]
    RBall,
    #[strum(serialize = "r_clavicle")]
    RClavicle,
    #[strum(serialize = "r_eye")]
    REye,
    #[strum(serialize = "r_foot")]
    RFoot,
    #[strum(serialize = "r_index1")]
    RIndex1,
    #[strum(serialize = "r_index2")]
    RIndex2,
    #[strum(serialize = "r_lowarm")]
    RLowarm,
    #[strum(serialize = "r_lowarm_twist")]
    RLowarmTwist,
    #[strum(serialize = "r_lowleg")]
    RLowleg,
    #[strum(serialize = "r_lowleg_twist")]
    RLowlegTwist,
    #[strum(serialize = "r_middle1")]
    RMiddle1,
    #[strum(serialize = "r_ring1")]
    RRing1,
    #[strum(serialize = "r_subtalar")]
    RSubtalar,
    #[strum(serialize = "r_thumb")]
    RThumb,
    #[strum(serialize = "r_thumb0")]
    RThumb0,
    #[strum(serialize = "r_thumb1")]
    RThumb1,
    #[strum(serialize = "r_uparm")]
    RUparm,
    #[strum(serialize = "r_uparm_twist")]
    RUparmTwist,
    #[strum(serialize = "r_uparm_twist3_proc")]
    RUparmTwist3Proc,
    #[strum(serialize = "r_upleg")]
    RUpleg,
    #[strum(serialize = "r_upleg_twist")]
    RUplegTwist,
    #[strum(serialize = "r_upleg_twist3_proc")]
    RUplegTwist3Proc,
    #[strum(serialize = "r_upperarm")]
    RUpperarm,
    #[strum(serialize = "r_weapon")]
    RWeapon,
    #[strum(serialize = "r_wrist")]
    RWrist,
    #[strum(serialize = "root")]
    Root,
}

impl AuthoredJoint {
    const fn name(self) -> RigJointName {
        RigJointName(Cow::Borrowed(Self::VARIANTS[self as usize]))
    }
}

impl RigJointName {
    /// Ordered upper leg, lower leg and foot targets for terrain IK.
    pub const LEFT_LEG_CHAIN: [Self; 3] = [Self::L_UPLEG, Self::L_LOWLEG, Self::L_FOOT];
    pub const RIGHT_LEG_CHAIN: [Self; 3] = [Self::R_UPLEG, Self::R_LOWLEG, Self::R_FOOT];
    pub const SKELETON: Self = AuthoredJoint::Skeleton.name();
    pub const BODY_WORLD: Self = AuthoredJoint::BodyWorld.name();
    pub const C_CAMERA: Self = AuthoredJoint::CCamera.name();
    pub const C_HEAD: Self = AuthoredJoint::CHead.name();
    pub const C_JAW: Self = AuthoredJoint::CJaw.name();
    pub const C_JAW_NULL: Self = AuthoredJoint::CJawNull.name();
    pub const C_NECK: Self = AuthoredJoint::CNeck.name();
    pub const C_NECK_TWIST: Self = AuthoredJoint::CNeckTwist.name();
    pub const C_SPINE0: Self = AuthoredJoint::CSpine0.name();
    pub const C_SPINE1: Self = AuthoredJoint::CSpine1.name();
    pub const C_SPINE2: Self = AuthoredJoint::CSpine2.name();
    pub const C_SPINE3: Self = AuthoredJoint::CSpine3.name();
    pub const C_TONGUE: Self = AuthoredJoint::CTongue.name();
    pub const L_BALL: Self = AuthoredJoint::LBall.name();
    pub const L_CLAVICLE: Self = AuthoredJoint::LClavicle.name();
    pub const L_EYE: Self = AuthoredJoint::LEye.name();
    pub const L_FOOT: Self = AuthoredJoint::LFoot.name();
    pub const L_INDEX1: Self = AuthoredJoint::LIndex1.name();
    pub const L_INDEX2: Self = AuthoredJoint::LIndex2.name();
    pub const L_LOWARM: Self = AuthoredJoint::LLowarm.name();
    pub const L_LOWARM_TWIST: Self = AuthoredJoint::LLowarmTwist.name();
    pub const L_LOWLEG: Self = AuthoredJoint::LLowleg.name();
    pub const L_LOWLEG_TWIST: Self = AuthoredJoint::LLowlegTwist.name();
    pub const L_MIDDLE1: Self = AuthoredJoint::LMiddle1.name();
    pub const L_MIDDLE_NULL: Self = AuthoredJoint::LMiddleNull.name();
    pub const L_PINKY_NULL: Self = AuthoredJoint::LPinkyNull.name();
    pub const L_RING1: Self = AuthoredJoint::LRing1.name();
    pub const L_SUBTALAR: Self = AuthoredJoint::LSubtalar.name();
    pub const L_TALOCRURAL: Self = AuthoredJoint::LTalocrural.name();
    pub const L_THUMB: Self = AuthoredJoint::LThumb.name();
    pub const L_THUMB0: Self = AuthoredJoint::LThumb0.name();
    pub const L_THUMB1: Self = AuthoredJoint::LThumb1.name();
    pub const L_TRANSVERSETARSAL: Self = AuthoredJoint::LTransversetarsal.name();
    pub const L_UPARM: Self = AuthoredJoint::LUparm.name();
    pub const L_UPARM_TWIST: Self = AuthoredJoint::LUparmTwist.name();
    pub const L_UPARM_TWIST0_PROC: Self = AuthoredJoint::LUparmTwist0Proc.name();
    pub const L_UPARM_TWIST3_PROC: Self = AuthoredJoint::LUparmTwist3Proc.name();
    pub const L_UPLEG: Self = AuthoredJoint::LUpleg.name();
    pub const L_UPLEG_TWIST: Self = AuthoredJoint::LUplegTwist.name();
    pub const L_UPLEG_TWIST2_PROC: Self = AuthoredJoint::LUplegTwist2Proc.name();
    pub const L_UPPERARM: Self = AuthoredJoint::LUpperarm.name();
    pub const L_WEAPON: Self = AuthoredJoint::LWeapon.name();
    pub const L_WRIST: Self = AuthoredJoint::LWrist.name();
    pub const R_BALL: Self = AuthoredJoint::RBall.name();
    pub const R_CLAVICLE: Self = AuthoredJoint::RClavicle.name();
    pub const R_EYE: Self = AuthoredJoint::REye.name();
    pub const R_FOOT: Self = AuthoredJoint::RFoot.name();
    pub const R_INDEX1: Self = AuthoredJoint::RIndex1.name();
    pub const R_INDEX2: Self = AuthoredJoint::RIndex2.name();
    pub const R_LOWARM: Self = AuthoredJoint::RLowarm.name();
    pub const R_LOWARM_TWIST: Self = AuthoredJoint::RLowarmTwist.name();
    pub const R_LOWLEG: Self = AuthoredJoint::RLowleg.name();
    pub const R_LOWLEG_TWIST: Self = AuthoredJoint::RLowlegTwist.name();
    pub const R_MIDDLE1: Self = AuthoredJoint::RMiddle1.name();
    pub const R_RING1: Self = AuthoredJoint::RRing1.name();
    pub const R_SUBTALAR: Self = AuthoredJoint::RSubtalar.name();
    pub const R_THUMB: Self = AuthoredJoint::RThumb.name();
    pub const R_THUMB0: Self = AuthoredJoint::RThumb0.name();
    pub const R_THUMB1: Self = AuthoredJoint::RThumb1.name();
    pub const R_UPARM: Self = AuthoredJoint::RUparm.name();
    pub const R_UPARM_TWIST: Self = AuthoredJoint::RUparmTwist.name();
    pub const R_UPARM_TWIST3_PROC: Self = AuthoredJoint::RUparmTwist3Proc.name();
    pub const R_UPLEG: Self = AuthoredJoint::RUpleg.name();
    pub const R_UPLEG_TWIST: Self = AuthoredJoint::RUplegTwist.name();
    pub const R_UPLEG_TWIST3_PROC: Self = AuthoredJoint::RUplegTwist3Proc.name();
    pub const R_UPPERARM: Self = AuthoredJoint::RUpperarm.name();
    pub const R_WEAPON: Self = AuthoredJoint::RWeapon.name();
    pub const R_WRIST: Self = AuthoredJoint::RWrist.name();
    pub const ROOT: Self = AuthoredJoint::Root.name();
}
