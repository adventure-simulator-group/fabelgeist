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

/// Native final-mesh triangle and barycentric weights frozen on the reference.
#[derive(Clone, Copy)]
pub(crate) struct ClothSurfaceSample {
    pub vertices: [usize; 3],
    pub weights: [f32; 3],
}

#[derive(Clone, Copy)]
pub(crate) enum ClosureSide {
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub(crate) struct ClosureAnchors {
    pub left: ClothSurfaceSample,
    pub right: ClothSurfaceSample,
}

impl ClothOpening {
    /// `first` converts this opening's carrier ordinals to final outer vertices.
    pub fn sample(&self, at: Permille, first: usize) -> Result<ClosureAnchors> {
        let sample = |stations: &[ClothEdgeStation]| -> Result<ClothSurfaceSample> {
            let pair = stations
                .windows(2)
                .find(|p| p[0].progress <= at.unit() && at.unit() <= p[1].progress)
                .context("closure lies outside its physical opening edges")?;
            let span = pair[1].progress - pair[0].progress;
            ensure!(span > 0., "closure edge stations must advance");
            Ok(ClothSurfaceSample {
                vertices: [
                    pair[0].vertex + first,
                    pair[1].vertex + first,
                    pair[1].vertex + first,
                ],
                weights: [
                    1. - (at.unit() - pair[0].progress) / span,
                    (at.unit() - pair[0].progress) / span,
                    0.,
                ],
            })
        };
        Ok(ClosureAnchors {
            left: sample(&self.left)?,
            right: sample(&self.right)?,
        })
    }
}
impl ClothSurfaceSample {
    pub fn position(&self, part: &BuiltPart) -> Result<Vec3> {
        self.interpolate(&part.positions)
    }
    pub fn normal(&self, part: &BuiltPart) -> Result<Vec3> {
        Ok(self.interpolate(&part.normals)?.normalize())
    }
    pub fn skin_vertex(&self) -> usize {
        let mut strongest = 0;
        for i in 1..3 {
            if self.weights[i] > self.weights[strongest] {
                strongest = i;
            }
        }
        self.vertices[strongest]
    }
    pub fn shifted(self, first: usize) -> Self {
        Self {
            vertices: self.vertices.map(|v| v + first),
            weights: self.weights,
        }
    }
    pub fn interpolate(&self, points: &[[f32; 3]]) -> Result<Vec3> {
        let point = |i| -> Result<Vec3> {
            let &[x, y, z] = points
                .get(i)
                .context("closure edge source is absent from fitted cloth")?;
            Ok(Vec3::new(x, y, z))
        };
        Ok(point(self.vertices[0])? * self.weights[0]
            + point(self.vertices[1])? * self.weights[1]
            + point(self.vertices[2])? * self.weights[2])
    }
}
impl ClosureAnchors {
    pub fn side(&self, side: ClosureSide) -> ClothSurfaceSample {
        match side {
            ClosureSide::Left => self.left,
            ClosureSide::Right => self.right,
        }
    }
}
