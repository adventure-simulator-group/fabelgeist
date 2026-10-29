//! Builds a momentum character (skeleton, mesh, skin weights, blend shapes)
//! out of an MHR `lod*.fbx` rig.
//!
//! The traversal order mirrors `momentum::loadOpenFbxCharacter`, because the
//! joint indices it produces are what the `.model` parameter transform, the
//! skin clusters, and the pose-corrective network are all indexed by.

use std::collections::HashMap;

use anyhow::{Context, Result, bail};

use crate::math::{Mat4, Transform};
use fabelgeist_fbx::{FbxObjectName, Object, Scene};
use fabelgeist_rig::{RigJointOrdinal, RigJointTable};
pub use skeleton::Skeleton;
use skeleton::{ParsedSkeleton, parse_skeleton};
pub use skin::SkinWeights;
use skin::parse_skin;

mod skeleton;
mod skin;

/// Momentum allows at most eight joint influences per vertex.
pub const MAX_SKIN_JOINTS: usize = 8;

/// `tx, ty, tz, rx, ry, rz, sc`.
pub const PARAMETERS_PER_JOINT: usize = 7;

/// Triangulated mesh with the original polygon topology dropped.
#[derive(Debug, Default, Clone)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub texcoord_faces: Vec<[u32; 3]>,
}

/// Number of vertices in an MHR mesh, distinct from a generic tensor axis.
/// This quantity carries no per-mesh identity or immutability guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshVertexCount(usize);

/// Dense blend-shape basis, `shape-major`: `vectors[shape * n_verts * 3 + ..]`.
#[derive(Debug, Default, Clone)]
pub struct BlendShapes {
    pub names: Vec<FbxObjectName>,
    pub vectors: Vec<f32>,
}

/// Everything the MHR forward pass needs from the rig file.
pub struct Character {
    pub skeleton: Skeleton,
    pub mesh: Mesh,
    pub skin_weights: SkinWeights,
    /// Inverse bind pose per joint as a skeleton state `[t, q, s]`.
    pub inverse_bind_pose: RigJointTable<[f32; 8]>,
    pub blend_shapes: BlendShapes,
}

impl Mesh {
    pub fn vertex_count(&self) -> MeshVertexCount {
        MeshVertexCount::from_native_vertex_count(self.vertices.len())
    }
}

impl MeshVertexCount {
    /// Admits a count at a mesh-storage or literal boundary, including zero.
    pub const fn from_native_vertex_count(vertices: usize) -> Self {
        Self(vertices)
    }

    /// Projects the count at a native numerical, allocation or text boundary.
    pub const fn len(self) -> usize {
        self.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BlendShapes {
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

impl Character {
    /// Loads a character from the bytes of a binary FBX rig.
    pub fn from_fbx_bytes(data: &[u8], load_blend_shapes: bool) -> Result<Self> {
        let scene = Scene::parse(data)?;
        let ParsedSkeleton {
            skeleton,
            joint_ids,
        } = parse_skeleton(&scene);
        if skeleton.is_empty() {
            bail!("no joints found in FBX rig");
        }

        let joint_of_object: HashMap<i64, RigJointOrdinal> = joint_ids
            .indexed()
            .map(|(joint, id)| (*id, joint))
            .collect();

        // Default to each joint's bind-pose global transform; clusters override
        // the joints they actually bind, which for MHR is all of them.
        let bind_pose = skeleton.bind_pose();
        let mut inverse_bind_pose: RigJointTable<Mat4> = bind_pose
            .iter()
            .map(|transform| {
                let inverse = transform.inverse();
                let rotation = crate::math::quat_to_matrix(inverse.rotation);
                let mut m = [[0.0; 4]; 4];
                for row in 0..3 {
                    for col in 0..3 {
                        m[row][col] = rotation[row][col] * inverse.scale;
                    }
                    m[row][3] = inverse.translation[row];
                }
                m[3][3] = 1.0;
                m
            })
            .collect();

        let mesh_model = scene
            .objects_of_kind("Model", "Mesh")
            .next()
            .context("FBX rig has no mesh")?;
        let geometry = scene
            .child_of_kind(mesh_model.id, "Geometry", "Mesh")
            .context("mesh model has no geometry")?;

        let vertex_values = geometry
            .node
            .child("Vertices")
            .and_then(|n| n.f64_array())
            .context("mesh geometry has no vertices")?;
        let (vertex_triples, remainder) = vertex_values.as_chunks::<3>();
        if !remainder.is_empty() {
            bail!("FBX vertex array contains an incomplete three-axis position");
        }
        let vertices: Vec<[f32; 3]> = vertex_triples
            .iter()
            .map(|v| [v[0] as f32, v[1] as f32, v[2] as f32])
            .collect();
        let polygon_vertex_index = geometry
            .node
            .child("PolygonVertexIndex")
            .and_then(|n| n.i64_array())
            .context("mesh geometry has no polygons")?;

        let faces = triangulate(&polygon_vertex_index)?;
        let (texcoords, texcoord_indices) = parse_uvs(geometry, polygon_vertex_index.len())?;
        let texcoord_faces = triangulate_texcoords(&polygon_vertex_index, &texcoord_indices);

        let mesh = Mesh {
            vertices,
            faces,
            texcoords,
            texcoord_faces,
        };
        let num_vertices = mesh.vertex_count();
        let blend_shapes = if load_blend_shapes {
            parse_blend_shapes(&scene, geometry, num_vertices)
        } else {
            BlendShapes::default()
        };

        let skin_weights = parse_skin(
            &scene,
            geometry,
            &joint_of_object,
            num_vertices,
            &mut inverse_bind_pose,
        )?;

        Ok(Self {
            skeleton,
            mesh,
            skin_weights,
            inverse_bind_pose: inverse_bind_pose
                .iter()
                .map(|m| Transform::from_matrix(m).to_skel_state())
                .collect(),
            blend_shapes,
        })
    }
}

/// Splits an FBX `PolygonVertexIndex` stream (negative index ends a polygon)
/// into fan-triangulated triangles.
fn triangulate(polygon_vertex_index: &[i64]) -> Result<Vec<[u32; 3]>> {
    let mut faces = Vec::new();
    let mut polygon: Vec<u32> = Vec::with_capacity(4);
    for raw in polygon_vertex_index {
        let (index, last) = if *raw < 0 {
            ((-(*raw + 1)) as u32, true)
        } else {
            (*raw as u32, false)
        };
        polygon.push(index);
        if !last {
            continue;
        }
        if polygon.len() < 3 {
            bail!(
                "invalid face with {} indices; expected at least 3",
                polygon.len()
            );
        }
        for j in 1..polygon.len() - 1 {
            faces.push([polygon[0], polygon[j], polygon[j + 1]]);
        }
        polygon.clear();
    }
    if !polygon.is_empty() {
        bail!("trailing polygon indices without a terminator");
    }
    Ok(faces)
}

fn parse_uvs(geometry: &Object, polygon_count: usize) -> Result<(Vec<[f32; 2]>, Vec<i64>)> {
    let Some(layer) = geometry.node.child("LayerElementUV") else {
        return Ok((Vec::new(), Vec::new()));
    };
    let Some(values) = layer.child("UV").and_then(|n| n.f64_array()) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let (uv_pairs, remainder) = values.as_chunks::<2>();
    if !remainder.is_empty() {
        bail!("FBX UV array contains an incomplete coordinate pair");
    }
    // FBX texture coordinates are bottom-up.
    let texcoords: Vec<[f32; 2]> = uv_pairs
        .iter()
        .map(|uv| [uv[0] as f32, 1.0 - uv[1] as f32])
        .collect();

    let reference = layer
        .child("ReferenceInformationType")
        .and_then(|n| n.str_prop(0))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let indices = if reference == b"IndexToDirect" {
        layer
            .child("UVIndex")
            .and_then(|n| n.i64_array())
            .unwrap_or_default()
    } else {
        (0..polygon_count as i64).collect()
    };
    Ok((texcoords, indices))
}

/// Re-splits UV indices onto the polygon boundaries of the vertex index stream.
fn triangulate_texcoords(polygon_vertex_index: &[i64], texcoord_indices: &[i64]) -> Vec<[u32; 3]> {
    if texcoord_indices.len() != polygon_vertex_index.len() {
        return Vec::new();
    }
    let mut faces = Vec::new();
    let mut polygon: Vec<u32> = Vec::with_capacity(4);
    for (raw, tex) in polygon_vertex_index.iter().zip(texcoord_indices) {
        polygon.push(*tex as u32);
        if *raw >= 0 {
            continue;
        }
        for j in 1..polygon.len().saturating_sub(1) {
            faces.push([polygon[0], polygon[j], polygon[j + 1]]);
        }
        polygon.clear();
    }
    faces
}

fn parse_blend_shapes(
    scene: &Scene,
    geometry: &Object,
    num_vertices: MeshVertexCount,
) -> BlendShapes {
    let mut names = Vec::new();
    let mut vectors: Vec<f32> = Vec::new();

    let Some(blend_shape) = scene
        .children(geometry.id)
        .find(|o| o.kind == "Deformer" && o.class == "BlendShape")
    else {
        return BlendShapes::default();
    };

    for channel in scene
        .children(blend_shape.id)
        .filter(|o| o.kind == "Deformer" && o.class == "BlendShapeChannel")
        .collect::<Vec<_>>()
    {
        for shape in scene
            .children(channel.id)
            .filter(|o| o.kind == "Geometry" && o.class == "Shape")
            .collect::<Vec<_>>()
        {
            let offsets = shape.node.child("Vertices").and_then(|n| n.f64_array());
            let indices = shape.node.child("Indexes").and_then(|n| n.i64_array());
            let base = vectors.len();
            vectors.resize(base + num_vertices.len() * 3, 0.0);
            if let (Some(offsets), Some(indices)) = (offsets, indices) {
                for (slot, vertex) in indices.iter().enumerate() {
                    let vertex = *vertex as usize;
                    if vertex >= num_vertices.len() || slot * 3 + 2 >= offsets.len() {
                        continue;
                    }
                    for axis in 0..3 {
                        vectors[base + vertex * 3 + axis] = offsets[slot * 3 + axis] as f32;
                    }
                }
            }
            names.push(shape.name.clone());
        }
    }

    BlendShapes { names, vectors }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulates_polygon_fans() {
        // Two polygons: a quad and a triangle, FBX-style negative terminators.
        let stream = [0, 1, 2, -4, 4, 5, -7];
        let faces = triangulate(&stream).unwrap();
        assert_eq!(faces, vec![[0, 1, 2], [0, 2, 3], [4, 5, 6]]);
    }

    #[test]
    fn rejects_degenerate_polygons() {
        assert!(triangulate(&[0, -2]).is_err());
        assert!(triangulate(&[0, 1, 2]).is_err());
    }
}
