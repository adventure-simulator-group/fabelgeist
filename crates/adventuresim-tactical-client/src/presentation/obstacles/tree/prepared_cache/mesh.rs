//! Lossless vertex-channel boundary for production tree meshes.
use super::super::impostor_assets::bounded;
use anyhow::{Context, Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
};
use serde::{Deserialize, Serialize};

pub(super) const MAX_PART_VERTICES: usize = 100_000;
pub(super) const MAX_PART_INDICES: usize = 600_000;
const MIN_NORMAL_LENGTH_SQUARED: f32 = 0.25;
const MAX_NORMAL_LENGTH_SQUARED: f32 = 2.25;

#[derive(Serialize, Deserialize)]
pub(super) struct PreparedMesh {
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_PART_VERTICES>")]
    positions: Vec<[f32; 3]>,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_PART_VERTICES>")]
    normals: Vec<[f32; 3]>,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_PART_VERTICES>")]
    uvs: Vec<[f32; 2]>,
    colors: ColorChannel,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_PART_INDICES>")]
    indices: Vec<u32>,
    bounds: Option<[[f32; 3]; 2]>,
}

#[derive(Serialize, Deserialize)]
enum ColorChannel {
    Absent,
    Present(
        #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_PART_VERTICES>")] Vec<[f32; 4]>,
    ),
}

#[derive(Clone, Copy)]
pub(super) enum Part {
    DetailedTrunk,
    MidTrunk,
    Wood,
    Buds,
    Leaves,
}

impl ColorChannel {
    fn values(&self) -> &[[f32; 4]] {
        match self {
            Self::Absent => &[],
            Self::Present(values) => values,
        }
    }
    fn is_present(&self) -> bool {
        matches!(self, Self::Present(_))
    }
}

impl PreparedMesh {
    pub(super) fn from_mesh(mesh: &Mesh) -> Result<Self> {
        ensure!(
            mesh.primitive_topology() == PrimitiveTopology::TriangleList,
            "tree mesh topology"
        );
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            anyhow::bail!("tree positions channel");
        };
        let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            anyhow::bail!("tree normals channel");
        };
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            anyhow::bail!("tree UV channel");
        };
        let colors = match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            None => ColorChannel::Absent,
            Some(VertexAttributeValues::Float32x4(colors)) => ColorChannel::Present(colors.clone()),
            Some(_) => anyhow::bail!("tree color channel"),
        };
        ensure!(
            mesh.attributes().count() == 3 + usize::from(colors.is_present()),
            "unexpected tree channel"
        );
        let source_indices = mesh.indices().context("indexed tree mesh")?;
        let indices = match source_indices {
            Indices::U32(indices) => indices.clone(),
            Indices::U16(_) => anyhow::bail!("tree mesh requires original u32 indices"),
        };
        let result = Self {
            positions: positions.clone(),
            normals: normals.clone(),
            uvs: uvs.clone(),
            colors,
            indices,
            bounds: Self::bounds(positions),
        };
        result.validate()?;
        Ok(result)
    }

    pub(super) fn validate(&self) -> Result<()> {
        let count = self.positions.len();
        ensure!(
            count <= MAX_PART_VERTICES && self.indices.len() <= MAX_PART_INDICES,
            "tree mesh size limit"
        );
        ensure!(
            self.normals.len() == count
                && self.uvs.len() == count
                && (!self.colors.is_present() || self.colors.values().len() == count),
            "tree channel cardinality"
        );
        ensure!(
            self.indices.len().is_multiple_of(3)
                && self.indices.iter().all(|index| (*index as usize) < count),
            "tree index range/topology"
        );
        ensure!(
            self.positions
                .iter()
                .flatten()
                .chain(self.normals.iter().flatten())
                .chain(self.uvs.iter().flatten())
                .chain(self.colors.values().iter().flatten())
                .all(|value| value.is_finite()),
            "nonfinite tree channel"
        );
        ensure!(
            self.normals.iter().all(|normal| (MIN_NORMAL_LENGTH_SQUARED
                ..=MAX_NORMAL_LENGTH_SQUARED)
                .contains(&Vec3::from_array(*normal).length_squared())),
            "invalid tree normal"
        );
        ensure!(
            self.bounds == Self::bounds(&self.positions),
            "tree mesh bounds mismatch"
        );
        Ok(())
    }

    pub(super) fn validate_part(&self, part: Part) -> Result<()> {
        self.validate()?;
        let colored = matches!(part, Part::DetailedTrunk | Part::MidTrunk | Part::Leaves);
        ensure!(
            self.colors.is_present() == colored,
            "tree part COLOR schema"
        );
        ensure!(
            self.positions.is_empty() == self.indices.is_empty(),
            "tree part triangle completeness"
        );
        if !matches!(part, Part::Buds) {
            ensure!(!self.indices.is_empty(), "required tree triangles");
        }
        Ok(())
    }

    pub(super) fn runtime(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        if let ColorChannel::Present(colors) = self.colors {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }

    pub(super) fn vertices(&self) -> usize {
        self.positions.len()
    }

    fn bounds(positions: &[[f32; 3]]) -> Option<[[f32; 3]; 2]> {
        let first = Vec3::from_array(*positions.first()?);
        let (mut low, mut high) = (first, first);
        for position in positions {
            let position = Vec3::from_array(*position);
            low = low.min(position);
            high = high.max(position);
        }
        Some([low.to_array(), high.to_array()])
    }
}
