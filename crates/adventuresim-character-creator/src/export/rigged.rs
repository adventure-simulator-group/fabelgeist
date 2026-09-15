//! Geometry, skinning, morph and physical-boundary contracts for rigged export.
use super::SurfaceTextures;

pub struct RiggedMesh<'a> {
    pub joint_proportions: &'a [adventuresim_core::character_proportions::JointProportionBasis],
    pub morph_targets: &'a [RiggedMorphTarget<'a>],
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
    /// Whether the source body faces are emitted as a rendered primitive.
    /// Shell-only equipment still supplies them to locate authored sockets.
    pub export_body: bool,
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
    pub joint_names: &'a [String],
    pub joint_parents: &'a [i32],
    /// Identity-shaped global MHR transforms, in metres.
    pub global_joint_states: &'a [[f32; 8]],
}

pub struct RiggedShell<'a> {
    /// Physical plate boundaries, separate from UV seams.
    pub plate_edges: &'a [[u32; 2]],
    pub textures: Option<SurfaceTextures>,
    /// Exact per-vertex UVs, including seam splits and interpolated cut edges.
    pub texcoords: Option<&'a [[f32; 2]]>,
    pub hinge: Option<adventuresim_armor_model::ArmorHinge>,
    pub name: &'a str,
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
    /// Independent armor topology supplies its own skin. Body-topology
    /// clothing leaves these empty and reuses the body's skin arrays.
    pub joint_indices: Option<&'a [[u32; 8]]>,
    pub joint_weights: Option<&'a [[f32; 8]]>,
    pub morph_targets: &'a [RiggedMorphTarget<'a>],
    /// Artist-facing sRGB color. glTF factors are converted to linear RGB.
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}

pub struct RiggedMorphTarget<'a> {
    pub name: &'a str,
    pub position_deltas: &'a [[f32; 3]],
    pub normal_deltas: &'a [[f32; 3]],
}
