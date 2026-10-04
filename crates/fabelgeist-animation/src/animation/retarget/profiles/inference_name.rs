//! Name admission and the closed keyword vocabulary used by profile inference.
use fabelgeist_rig::{RigJointMembership, RigJointName};
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Side {
    Left,
    Right,
    Center,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct InferredName {
    pub side: Side,
    pub stem: InferenceStem,
}
impl From<&RigJointName> for InferredName {
    fn from(name: &RigJointName) -> Self {
        let spelling: Cow<'static, str> = name.clone().into();
        let bare = spelling.rsplit([':', '|']).next().unwrap_or(&spelling);
        let lower = bare.to_ascii_lowercase();

        let strip = |side: Side, rest: String| -> Self {
            Self {
                side,
                stem: InferenceStem::from(rest),
            }
        };

        for (word, side) in [("left", Side::Left), ("right", Side::Right)] {
            if let Some(position) = lower.find(word) {
                let mut rest = lower.clone();
                rest.replace_range(position..position + word.len(), "");
                return strip(side, rest);
            }
        }

        let separators = ['_', '-', '.', ' '];
        for (prefix, side) in [("l", Side::Left), ("r", Side::Right)] {
            for separator in separators {
                let marker = format!("{prefix}{separator}");
                if lower.starts_with(&marker) {
                    return strip(side, lower[marker.len()..].to_string());
                }
                let marker = format!("{separator}{prefix}");
                if lower.ends_with(&marker) {
                    return strip(side, lower[..lower.len() - marker.len()].to_string());
                }
            }
        }

        // `LHipJoint`, `lFemur`: a lone side letter before a capitalized word.
        let mut characters = bare.chars();
        if let (Some(first), Some(second)) = (characters.next(), characters.next())
            && second.is_ascii_uppercase()
        {
            match first {
                'l' | 'L' => return strip(Side::Left, bare[1..].to_string()),
                'r' | 'R' => return strip(Side::Right, bare[1..].to_string()),
                _ => {}
            }
        }

        Self {
            side: Side::Center,
            stem: InferenceStem::from(bare.to_owned()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct InferenceStem(String);
impl From<String> for InferenceStem {
    fn from(spelling: String) -> Self {
        Self(
            spelling
                .chars()
                .filter(|character: &char| -> bool { character.is_ascii_alphanumeric() })
                .map(|character: char| -> char { character.to_ascii_lowercase() })
                .collect(),
        )
    }
}
impl InferenceStem {
    pub fn contains(&self, keyword: InferenceKeyword) -> RigJointMembership {
        RigJointMembership::from(self.0.contains(<&'static str>::from(keyword)))
    }
    pub fn starts_with(&self, keyword: InferenceKeyword) -> RigJointMembership {
        RigJointMembership::from(self.0.starts_with(<&'static str>::from(keyword)))
    }
    pub fn root_membership(&self) -> RigJointMembership {
        RigJointMembership::from(matches!(self.0.as_str(), "root" | "reference" | "armature"))
    }
}

/// A recognized fragment, independent of an exact rig-joint label.
#[derive(Clone, Copy)]
pub(super) enum InferenceKeyword {
    Hips,
    Pelvis,
    Hip,
    Neck,
    Head,
    Clavicle,
    Shoulder,
    Collar,
    Upperarm,
    Uparm,
    Humerus,
    Shldr,
    Arm,
    Forearm,
    Lowerarm,
    Lowarm,
    Elbow,
    Ulna,
    Hand,
    Wrist,
    Upperleg,
    Upleg,
    Thigh,
    Femur,
    Lowerleg,
    Lowleg,
    Shin,
    Calf,
    Knee,
    Tibia,
    Leg,
    Foot,
    Ankle,
    Toebase,
    Toe,
    Ball,
    Spine,
    Chest,
    Torso,
    Abdomen,
    Waist,
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
    Little,
}
impl From<InferenceKeyword> for &'static str {
    fn from(keyword: InferenceKeyword) -> Self {
        match keyword {
            InferenceKeyword::Hips => "hips",
            InferenceKeyword::Pelvis => "pelvis",
            InferenceKeyword::Hip => "hip",
            InferenceKeyword::Neck => "neck",
            InferenceKeyword::Head => "head",
            InferenceKeyword::Clavicle => "clavicle",
            InferenceKeyword::Shoulder => "shoulder",
            InferenceKeyword::Collar => "collar",
            InferenceKeyword::Upperarm => "upperarm",
            InferenceKeyword::Uparm => "uparm",
            InferenceKeyword::Humerus => "humerus",
            InferenceKeyword::Shldr => "shldr",
            InferenceKeyword::Arm => "arm",
            InferenceKeyword::Forearm => "forearm",
            InferenceKeyword::Lowerarm => "lowerarm",
            InferenceKeyword::Lowarm => "lowarm",
            InferenceKeyword::Elbow => "elbow",
            InferenceKeyword::Ulna => "ulna",
            InferenceKeyword::Hand => "hand",
            InferenceKeyword::Wrist => "wrist",
            InferenceKeyword::Upperleg => "upperleg",
            InferenceKeyword::Upleg => "upleg",
            InferenceKeyword::Thigh => "thigh",
            InferenceKeyword::Femur => "femur",
            InferenceKeyword::Lowerleg => "lowerleg",
            InferenceKeyword::Lowleg => "lowleg",
            InferenceKeyword::Shin => "shin",
            InferenceKeyword::Calf => "calf",
            InferenceKeyword::Knee => "knee",
            InferenceKeyword::Tibia => "tibia",
            InferenceKeyword::Leg => "leg",
            InferenceKeyword::Foot => "foot",
            InferenceKeyword::Ankle => "ankle",
            InferenceKeyword::Toebase => "toebase",
            InferenceKeyword::Toe => "toe",
            InferenceKeyword::Ball => "ball",
            InferenceKeyword::Spine => "spine",
            InferenceKeyword::Chest => "chest",
            InferenceKeyword::Torso => "torso",
            InferenceKeyword::Abdomen => "abdomen",
            InferenceKeyword::Waist => "waist",
            InferenceKeyword::Thumb => "thumb",
            InferenceKeyword::Index => "index",
            InferenceKeyword::Middle => "middle",
            InferenceKeyword::Ring => "ring",
            InferenceKeyword::Pinky => "pinky",
            InferenceKeyword::Little => "little",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct KeywordSpecificity(usize);
impl InferenceKeyword {
    pub fn specificity(self) -> KeywordSpecificity {
        KeywordSpecificity(<&'static str>::from(self).len())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ClaimState {
    Available,
    Claimed,
}
