//! Frozen physical edge stations; material seams are not dressing openings.
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::{BuiltPart, Permille};
use fabelgeist_math::Vec3;

/// Native outer-carrier vertex ordinal and normalized construction distance.
#[derive(Clone, Copy)]
pub(crate) struct ClothEdgeStation {
    pub vertex: usize,
    pub progress: f32,
}

pub(crate) struct ClothOpening {
    pub left: Vec<ClothEdgeStation>,
    pub right: Vec<ClothEdgeStation>,
}

/// Native final-mesh edge and interpolation weight, frozen on the reference.
#[derive(Clone, Copy)]
pub(crate) struct ClothEdgeSample {
    pub vertices: [usize; 2],
    pub weight: f32,
}

#[derive(Clone, Copy)]
pub(crate) enum ClosureSide {
    Left,
    Right,
}

pub(crate) struct ClosureAnchors {
    pub left: ClothEdgeSample,
    pub right: ClothEdgeSample,
}

impl ClothOpening {
    /// `first` converts this opening's carrier ordinals to final outer vertices.
    pub fn sample(&self, at: Permille, first: usize) -> Result<ClosureAnchors> {
        let sample = |stations: &[ClothEdgeStation]| -> Result<ClothEdgeSample> {
            let pair = stations
                .windows(2)
                .find(|p| p[0].progress <= at.unit() && at.unit() <= p[1].progress)
                .context("closure lies outside its physical opening edges")?;
            let span = pair[1].progress - pair[0].progress;
            ensure!(span > 0., "closure edge stations must advance");
            Ok(ClothEdgeSample {
                vertices: [pair[0].vertex + first, pair[1].vertex + first],
                weight: (at.unit() - pair[0].progress) / span,
            })
        };
        Ok(ClosureAnchors {
            left: sample(&self.left)?,
            right: sample(&self.right)?,
        })
    }
}
impl ClothEdgeSample {
    pub fn position(&self, part: &BuiltPart) -> Result<Vec3> {
        self.interpolate(&part.positions)
    }
    pub fn normal(&self, part: &BuiltPart) -> Result<Vec3> {
        Ok(self.interpolate(&part.normals)?.normalize())
    }
    pub fn skin_vertex(&self) -> usize {
        self.vertices[usize::from(self.weight >= 0.5)]
    }
    fn interpolate(&self, points: &[[f32; 3]]) -> Result<Vec3> {
        let point = |i| -> Result<Vec3> {
            let &[x, y, z] = points
                .get(i)
                .context("closure edge source is absent from fitted cloth")?;
            Ok(Vec3::new(x, y, z))
        };
        Ok(point(self.vertices[0])?.lerp(point(self.vertices[1])?, self.weight))
    }
}
impl ClosureAnchors {
    pub fn side(&self, side: ClosureSide) -> ClothEdgeSample {
        match side {
            ClosureSide::Left => self.left,
            ClosureSide::Right => self.right,
        }
    }
}
