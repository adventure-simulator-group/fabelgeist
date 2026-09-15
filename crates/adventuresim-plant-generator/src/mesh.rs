use crate::Pigment;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use serde::{Deserialize, Serialize};

/// Bounded surface resolution shared by authoring and tactical presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tessellation {
    Close,
    Field,
}
impl Tessellation {
    pub(crate) fn segments(self) -> usize {
        match self {
            Self::Close => 16,
            Self::Field => 8,
        }
    }
}

/// Renderer-neutral indexed geometry; coordinates and normals are Y-up.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PlantMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}
impl PlantMesh {
    pub fn into_bevy(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_indices(Indices::U32(self.indices))
    }

    /// Add an instance to a cell mesh while preserving its palette and normals.
    pub fn append(&mut self, source: &Self, translation: Vec3, rotation: Quat, scale: f32) {
        let base = self.positions.len() as u32;
        self.positions.extend(
            source
                .positions
                .iter()
                .map(|p| (translation + rotation * Vec3::from_array(*p) * scale).to_array()),
        );
        self.normals.extend(
            source
                .normals
                .iter()
                .map(|n| (rotation * Vec3::from_array(*n)).to_array()),
        );
        self.colors.extend_from_slice(&source.colors);
        self.indices
            .extend(source.indices.iter().map(|index| base + index));
    }

    pub(crate) fn surface(
        &mut self,
        segments: [usize; 2],
        pigment: Pigment,
        point: impl Fn(f32, f32) -> Vec3,
    ) {
        const DERIVATIVE_STEP: f32 = 0.0001;
        const MIN_TRIANGLE_AREA_SQUARED: f32 = 1e-18;
        let base = self.positions.len();
        let [rows, columns] = segments;
        for row in 0..=rows {
            for col in 0..=columns {
                let u = row as f32 / rows as f32;
                let v = col as f32 / columns as f32;
                let du = point((u + DERIVATIVE_STEP).min(1.0), v)
                    - point((u - DERIVATIVE_STEP).max(0.0), v);
                let dv = point(u, (v + DERIVATIVE_STEP).min(1.0))
                    - point(u, (v - DERIVATIVE_STEP).max(0.0));
                self.positions.push(point(u, v).to_array());
                self.normals
                    .push(dv.cross(du).normalize_or(Vec3::Y).to_array());
                self.colors.push(pigment.linear());
            }
        }
        for row in 0..rows {
            for col in 0..columns {
                let a = base + row * (columns + 1) + col;
                let b = a + columns + 1;
                for triangle in [[a, a + 1, b], [a + 1, b + 1, b]] {
                    let [p, q, r] = triangle.map(|i| Vec3::from_array(self.positions[i]));
                    if (q - p).cross(r - p).length_squared() > MIN_TRIANGLE_AREA_SQUARED {
                        self.indices.extend(triangle.map(|i| i as u32));
                    }
                }
            }
        }
    }

    pub(crate) fn tube(
        &mut self,
        from: Vec3,
        to: Vec3,
        radii: [f32; 2],
        pigment: Pigment,
        sides: usize,
    ) {
        let axis = (to - from).normalize_or(Vec3::Y);
        let right = axis.any_orthonormal_vector();
        let forward = axis.cross(right);
        self.surface([1, sides], pigment, |t, v| {
            let angle = v * std::f32::consts::TAU;
            from.lerp(to, t)
                + (right * angle.cos() + forward * angle.sin())
                    * (radii[0] + (radii[1] - radii[0]) * t)
        });
    }

    pub(crate) fn ellipsoid(
        &mut self,
        center: Vec3,
        radii: Vec3,
        pigment: Pigment,
        segments: usize,
    ) {
        self.surface([segments / 2, segments], pigment, |u, v| {
            let latitude = u * std::f32::consts::PI;
            let longitude = v * std::f32::consts::TAU;
            center
                + radii
                    * Vec3::new(
                        latitude.sin() * longitude.cos(),
                        latitude.cos(),
                        latitude.sin() * longitude.sin(),
                    )
        });
    }
}
