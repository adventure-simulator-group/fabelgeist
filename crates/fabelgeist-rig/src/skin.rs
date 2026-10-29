//! Fixed-width influence ordinals retained in their native 32-bit layout.

use crate::{RigJointOrdinal, RigJointTable};
use derive_more::{Display, Error};

/// The ordinal bound of an admitted rig column. Equal bounds do not establish
/// that two independently produced columns belong to the same rig instance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RigJointCount(usize);

/// Packed rig influences. The width, slot order, and zero-weight slots are
/// retained exactly; admission checks every word against the actual rig bound.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PackedRigSkin<const WIDTH: usize> {
    words: Vec<[u32; WIDTH]>,
    joints: RigJointCount,
}

/// One vertex's ordered influences during a numerical skin calculation.
/// Ordinals retain their rig meaning; membership comes from the source buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigInfluences<const WIDTH: usize> {
    pub joints: [RigJointOrdinal; WIDTH],
    pub weights: [f32; WIDTH],
}

/// Rejected skin admission or encoding.
/// `OutsideRig` retains the supplied rig bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, Error)]
pub enum RigSkinError {
    #[display("skin joint {joint} is outside a rig of {} joints", joints.len())]
    OutsideRig { joint: u32, joints: RigJointCount },
    #[display("skin joint cannot be represented by a native 32-bit word")]
    NativeWidth,
    #[display("appended skin has a different rig bound")]
    DifferentRigBound,
}

impl RigJointCount {
    /// A count supplied by a native rig decoder or an authored numerical
    /// fixture. Production callers retain their source rig's actual extent;
    /// deriving a larger extent from skin words would bypass membership checks.
    pub const fn from_native_joint_count(joints: usize) -> Self {
        Self(joints)
    }

    pub(crate) fn from_table<T>(table: &RigJointTable<T>) -> Self {
        Self(table.len())
    }

    pub fn len(self) -> usize {
        self.0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl RigInfluences<8> {
    pub fn whole_joint(joint: RigJointOrdinal) -> Self {
        let mut result = Self {
            joints: [RigJointOrdinal::from(0usize); 8],
            weights: [0.0; 8],
        };
        result.joints[0] = joint;
        result.weights[0] = 1.0;
        result
    }
}

impl<const WIDTH: usize> RigInfluences<WIDTH> {
    /// The VEC4 reduction shared by Bevy and glTF. Duplicate joints combine
    /// before sorting; equal weights retain ascending ordinal order.
    pub fn strongest_four(self) -> RigInfluences<4> {
        let mut combined = std::collections::BTreeMap::<RigJointOrdinal, f32>::new();
        for (joint, weight) in self.joints.into_iter().zip(self.weights) {
            *combined.entry(joint).or_default() += weight;
        }
        let mut influences = combined.into_iter().collect::<Vec<_>>();
        influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        influences.truncate(4);
        let sum: f32 = influences.iter().map(|(_, weight)| weight).sum();
        let mut result = RigInfluences {
            joints: [RigJointOrdinal::from(0usize); 4],
            weights: [0.0; 4],
        };
        for (slot, (joint, weight)) in influences.into_iter().enumerate() {
            result.joints[slot] = joint;
            result.weights[slot] = weight / sum;
        }
        result
    }

    /// Native Bevy/glTF VEC4 word encoding, with no ordinal truncation.
    pub fn native_u16(self) -> Result<[u16; WIDTH], std::num::TryFromIntError> {
        let mut words = [0; WIDTH];
        for (word, joint) in words.iter_mut().zip(self.joints) {
            *word = u16::try_from(usize::from(joint))?;
        }
        Ok(words)
    }
}

impl<const WIDTH: usize> PackedRigSkin<WIDTH> {
    /// Decode glTF, FBX, GPU readback, or fixture words once at their source.
    pub fn from_native(
        words: Vec<[u32; WIDTH]>,
        joints: RigJointCount,
    ) -> Result<Self, RigSkinError> {
        for &joint in words.iter().flatten() {
            if usize::try_from(joint).map_or(true, |joint| joint >= joints.0) {
                return Err(RigSkinError::OutsideRig { joint, joints });
            }
        }
        Ok(Self { words, joints })
    }

    pub fn empty(joints: RigJointCount) -> Self {
        Self {
            words: Vec::new(),
            joints,
        }
    }

    pub fn repeated(
        influences: [RigJointOrdinal; WIDTH],
        vertices: usize,
        joints: RigJointCount,
    ) -> Result<Self, RigSkinError> {
        let mut skin = Self::empty(joints);
        let words = skin.encode(influences)?;
        skin.words = vec![words; vertices];
        Ok(skin)
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn clear(&mut self) {
        self.words.clear();
    }

    pub fn joint_count(&self) -> RigJointCount {
        self.joints
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = [RigJointOrdinal; WIDTH]> + '_ {
        self.words.iter().map(|row| row.map(RigJointOrdinal::from))
    }

    /// The vertex is established by the associated surface traversal.
    pub fn vertex(&self, vertex: usize) -> [RigJointOrdinal; WIDTH] {
        self.words[vertex].map(RigJointOrdinal::from)
    }

    pub fn weighted_vertex(&self, vertex: usize, weights: [f32; WIDTH]) -> RigInfluences<WIDTH> {
        RigInfluences {
            joints: self.vertex(vertex),
            weights,
        }
    }

    /// Native upload/export port. Application operations use typed influences.
    pub fn native_words(&self) -> &[[u32; WIDTH]] {
        &self.words
    }

    pub fn push(&mut self, joints: [RigJointOrdinal; WIDTH]) -> Result<(), RigSkinError> {
        let words = self.encode(joints)?;
        self.words.push(words);
        Ok(())
    }

    pub fn set_vertex(
        &mut self,
        vertex: usize,
        joints: [RigJointOrdinal; WIDTH],
    ) -> Result<(), RigSkinError> {
        let words = self.encode(joints)?;
        self.words[vertex] = words;
        Ok(())
    }

    pub fn select(&self, vertices: impl IntoIterator<Item = usize>) -> Self {
        Self {
            words: vertices
                .into_iter()
                .map(|vertex| self.words[vertex])
                .collect(),
            joints: self.joints,
        }
    }

    pub fn append(&mut self, other: Self) -> Result<(), RigSkinError> {
        if self.is_empty() {
            *self = other;
            return Ok(());
        }
        if !other.is_empty() && self.joints != other.joints {
            return Err(RigSkinError::DifferentRigBound);
        }
        self.words.extend(other.words);
        Ok(())
    }

    fn encode(&self, joints: [RigJointOrdinal; WIDTH]) -> Result<[u32; WIDTH], RigSkinError> {
        let mut words = [0; WIDTH];
        for (word, joint) in words.iter_mut().zip(joints) {
            *word = u32::try_from(joint).map_err(|_| RigSkinError::NativeWidth)?;
            if usize::from(joint) >= self.joints.0 {
                return Err(RigSkinError::OutsideRig {
                    joint: *word,
                    joints: self.joints,
                });
            }
        }
        Ok(words)
    }
}
