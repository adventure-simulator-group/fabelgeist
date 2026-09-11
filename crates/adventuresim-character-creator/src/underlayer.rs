//! Fitted textile garments and detachable mail patches on the source body.
mod direction;
mod gap;
mod pattern;
mod standoff;
pub use pattern::regions;

use crate::{
    armor_frames::Wearer,
    surface_cut::{SurfaceCut, interpolate},
};
use adventuresim_armor_model::{Millimeters, PartMesh, Permille};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnderlayerKind {
    ArmingDoublet,
    PaddedHose,
    MailVoiders,
    MailBrayette,
    MailKneeVoider,
    MailStandard,
}

impl UnderlayerKind {
    pub fn is_mail(self) -> bool {
        matches!(
            self,
            Self::MailVoiders | Self::MailBrayette | Self::MailKneeVoider | Self::MailStandard
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnderlayerDesign {
    pub kind: UnderlayerKind,
    pub clearance: Millimeters,
    pub thickness: Millimeters,
    pub length: Permille,
    pub sleeve_length: Permille,
    pub patch_width: Millimeters,
    /// Additional Boolean subtraction boxes, in the reference body's metre space.
    pub cuts: Vec<SurfaceBox>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceBox {
    pub minimum: ReferencePoint,
    pub maximum: ReferencePoint,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ReferencePoint(pub [f32; 3]);

pub const CLEARANCE_MM: std::ops::RangeInclusive<u16> = 1..=10;
pub const THICKNESS_MM: std::ops::RangeInclusive<u16> = 1..=8;
pub const DOUBLET_LENGTH: std::ops::RangeInclusive<u16> = 700..=1200;
pub const HOSE_LENGTH: std::ops::RangeInclusive<u16> = 700..=1000;
pub const BRAYETTE_LENGTH: std::ops::RangeInclusive<u16> = 900..=1200;
pub const SLEEVE_LENGTH: std::ops::RangeInclusive<u16> = 500..=1000;
pub const PATCH_WIDTH_MM: std::ops::RangeInclusive<u16> = 35..=100;

impl UnderlayerDesign {
    pub fn length_range(&self) -> std::ops::RangeInclusive<u16> {
        match self.kind {
            UnderlayerKind::PaddedHose => HOSE_LENGTH,
            UnderlayerKind::MailBrayette => BRAYETTE_LENGTH,
            _ => DOUBLET_LENGTH,
        }
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            CLEARANCE_MM.contains(&self.clearance.0),
            "underlayer clearance must be 1..10 mm"
        );
        ensure!(
            THICKNESS_MM.contains(&self.thickness.0),
            "underlayer thickness must be 1..8 mm"
        );
        ensure!(
            self.length_range().contains(&self.length.0),
            "underlayer length is outside its construction range"
        );
        ensure!(
            SLEEVE_LENGTH.contains(&self.sleeve_length.0),
            "underlayer sleeve length must be 500..1000 permille"
        );
        ensure!(
            PATCH_WIDTH_MM.contains(&self.patch_width.0),
            "mail patch width must be 35..100 mm"
        );
        for cut in &self.cuts {
            ensure!(
                (0..3).all(|i| cut.minimum.0[i].is_finite()
                    && cut.maximum.0[i].is_finite()
                    && cut.minimum.0[i] < cut.maximum.0[i]),
                "invalid underlayer subtraction box"
            );
        }
        Ok(())
    }
}

/// Immutable connectivity and source points shared by neutral and morphed bodies.
pub struct UnderlayerPattern {
    pub cut: SurfaceCut,
    borders: Vec<[u32; 2]>,
    compression: Vec<f32>,
    direction_constraints: Vec<direction::Constraint>,
}

impl UnderlayerPattern {
    pub fn new(
        design: &UnderlayerDesign,
        placement: &str,
        body: &Wearer<'_>,
        uv_faces: &[[u32; 3]],
    ) -> Result<Self> {
        design.validate()?;
        let (include, subtract) = regions(design, placement, body)?;
        let cut = SurfaceCut::new(body.positions, body.faces, uv_faces, &include, &subtract);
        ensure!(
            !cut.faces.is_empty(),
            "underlayer cuts removed the whole garment"
        );
        let positions = cut
            .points
            .iter()
            .map(|s| {
                interpolate(
                    body.faces[s.triangle].map(|v| body.positions[v as usize]),
                    s.weights,
                )
            })
            .collect::<Vec<_>>();
        let borders = cut.borders(&positions);
        Ok(Self {
            cut,
            borders,
            compression: standoff::compression(body),
            direction_constraints: Vec::new(),
        })
    }

    /// Freeze the tightest sampled layer envelope before creating any targets.
    /// Interpolating independent compression minima could invert the offset
    /// when many identity targets are blended with negative weights.
    pub fn constrain_for(&mut self, body: &Wearer<'_>) {
        let directions = direction::constrained(body, &self.direction_constraints);
        for (limit, sample) in self
            .compression
            .iter_mut()
            .zip(standoff::along(body, &directions))
        {
            *limit = limit.min(sample);
        }
    }

    /// Unposed body proportions can turn a neutral offset through its surface.
    /// Establish their common outward cone before computing the layer envelope.
    pub fn constrain_directions_for(&mut self, body: &Wearer<'_>) {
        self.direction_constraints
            .extend(direction::constraints(body));
    }

    /// Bone proportion translations retain the reference offset vectors.
    /// Check that exact extrusion, rather than solving a new shaped direction.
    pub fn constrain_translation_for(&mut self, reference: &Wearer<'_>, shaped: &Wearer<'_>) {
        let directions = direction::constrained(reference, &self.direction_constraints);
        for (limit, sample) in self
            .compression
            .iter_mut()
            .zip(standoff::along(shaped, &directions))
        {
            *limit = limit.min(sample);
        }
    }

    pub fn evaluate(&self, design: &UnderlayerDesign, body: &Wearer<'_>) -> PartMesh {
        let count = self.cut.points.len() as u32;
        let compression = &self.compression;
        let directions = direction::constrained(body, &self.direction_constraints);
        let mut positions = Vec::new();
        for offset in [
            design.clearance.metres() + design.thickness.metres(),
            design.clearance.metres(),
        ] {
            for point in &self.cut.points {
                let face = body.faces[point.triangle];
                // Cut vertices lie on the already offset source triangle.
                // Renormalizing an interpolated normal here would bow its
                // interior and make changing a cut change the bulk surface.
                positions.push(interpolate(
                    face.map(|v| {
                        let v = v as usize;
                        std::array::from_fn(|i| {
                            body.positions[v][i] + offset * compression[v] * directions[v][i]
                        })
                    }),
                    point.weights,
                ));
            }
        }
        let mut indices = Vec::new();
        for &[a, b, c] in &self.cut.faces {
            indices.extend([a, b, c, c + count, b + count, a + count]);
        }
        for &[a, b] in &self.borders {
            // A textile cut edge has its own shading normal. Sharing these
            // vertices with the inner face would darken the hem and collar.
            let first = positions.len() as u32;
            for index in [a, b, a + count, b + count] {
                positions.push(positions[index as usize]);
            }
            indices.extend([first + 1, first, first + 2, first + 1, first + 2, first + 3]);
        }
        let mut mesh = PartMesh::new();
        mesh.positions = positions;
        mesh.indices = indices;
        mesh
    }

    /// Transfer source attributes to the inner face and independent cut-edge vertices.
    pub fn shell_attributes<T: Copy>(&self, outer: &[T]) -> Vec<T> {
        assert_eq!(outer.len(), self.cut.points.len());
        let mut result = outer.to_vec();
        result.extend_from_slice(outer);
        for &[a, b] in &self.borders {
            result.extend([
                outer[a as usize],
                outer[b as usize],
                outer[a as usize],
                outer[b as usize],
            ]);
        }
        result
    }
}

#[cfg(test)]
mod body_fixture;
#[cfg(test)]
mod tests;
