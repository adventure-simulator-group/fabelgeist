//! Vertex skin bindings retain skin slots and dimensionless blending quantities.
use super::SkinJointOrdinal;

/// A computed blending quantity. Admission preserves IEEE values; this owner
/// does not claim that imported geometry or computed weights are finite.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct SkinBlendWeight(f32);
impl SkinBlendWeight {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);
    pub fn normalized_by(self, total: Self) -> Self {
        Self(self.0 / total.0)
    }
}
impl From<f32> for SkinBlendWeight {
    fn from(weight: f32) -> Self {
        Self(weight)
    }
}
impl From<SkinBlendWeight> for f32 {
    fn from(weight: SkinBlendWeight) -> Self {
        weight.0
    }
}
impl std::ops::AddAssign for SkinBlendWeight {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}
impl std::iter::Sum<SkinBlendWeight> for SkinBlendWeight {
    fn sum<I: Iterator<Item = Self>>(weights: I) -> Self {
        Self(weights.map(f32::from).sum())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VertexSkinBinding {
    pub joints: [SkinJointOrdinal; 4],
    pub weights: [SkinBlendWeight; 4],
}
impl Default for VertexSkinBinding {
    fn default() -> Self {
        Self {
            joints: [SkinJointOrdinal::from(0_usize); 4],
            weights: [SkinBlendWeight::ZERO; 4],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkinnedVertexCount(usize);
impl From<usize> for SkinnedVertexCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkinnedVertexOrdinal(usize);
impl From<usize> for SkinnedVertexOrdinal {
    fn from(index: usize) -> Self {
        Self(index)
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VertexSkinWeights(Vec<VertexSkinBinding>);
impl From<Vec<VertexSkinBinding>> for VertexSkinWeights {
    fn from(vertices: Vec<VertexSkinBinding>) -> Self {
        Self(vertices)
    }
}
impl VertexSkinWeights {
    pub fn count(&self) -> SkinnedVertexCount {
        SkinnedVertexCount::from(self.0.len())
    }
    pub fn iter(&self) -> std::slice::Iter<'_, VertexSkinBinding> {
        self.0.iter()
    }
}
impl std::ops::Index<SkinnedVertexOrdinal> for VertexSkinWeights {
    type Output = VertexSkinBinding;
    fn index(&self, index: SkinnedVertexOrdinal) -> &VertexSkinBinding {
        &self.0[index.0]
    }
}

impl std::fmt::Display for SkinBlendWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
