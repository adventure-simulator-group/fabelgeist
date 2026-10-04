//! Depth displacement, with reusable pipeline authority tied to one context.
use super::GpuMesh;
use fabelgeist_gpu::prelude::{
    Buffer, BufferByteLength, BufferDefinition, BufferUse, ComputePass, ComputePipeline,
    InvocationCount, Mat4, PassParameter, PassParameterName, PassParameters, Sampler, ShaderSource,
    Texture2d, UniformNumber, WgpuContext, WorkgroupGrid, WorkgroupShape,
};

/// Signed depth displacement scale, rounded at the uniform ABI boundary.
///
/// ```compile_fail
/// use fabelgeist_mesh::{DisplacementScale, SphereGeometry};
/// SphereGeometry::default().with_radius(DisplacementScale::from(0.5));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplacementScale(f64);
impl From<f64> for DisplacementScale {
    fn from(scale: f64) -> Self {
        Self(scale)
    }
}
impl DisplacementScale {
    fn bind(self, parameters: &mut PassParameters) {
        parameters.insert(
            PassParameterName::from("strength"),
            PassParameter::Number(UniformNumber::from(self.0 as f32)),
        );
    }
}

/// Projection and its inverse travel together; singular input retains the
/// established default-matrix fallback rather than a new admission failure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplacementProjection {
    projection: Mat4,
    inverse: Mat4,
}
impl From<Mat4> for DisplacementProjection {
    fn from(projection: Mat4) -> Self {
        Self {
            projection,
            inverse: projection.inverse().unwrap_or_default(),
        }
    }
}
impl DisplacementProjection {
    fn bind(self, parameters: &mut PassParameters) {
        parameters.insert(
            PassParameterName::from("projection"),
            self.projection.into(),
        );
        parameters.insert(
            PassParameterName::from("inv_projection"),
            self.inverse.into(),
        );
    }
}

/// Position-buffer ABI extent; narrowing precedes division just as in the
/// original dispatch. It does not derive coverage from the mesh draw count.
struct DisplacementPositionSpan(BufferByteLength);
impl From<BufferByteLength> for DisplacementPositionSpan {
    fn from(length: BufferByteLength) -> Self {
        Self(length)
    }
}
impl DisplacementPositionSpan {
    fn grid(self) -> WorkgroupGrid {
        let position_bytes = std::mem::size_of::<[f32; 3]>() as u32;
        let invocations = InvocationCount::from(u64::from(self.0) as u32 / position_bytes);
        // This is the fixed displacement shader's declared invocation shape.
        WorkgroupShape::try_from([64, 1, 1])
            .expect("fixed displacement workgroup shape")
            .covering_x(invocations)
    }
}

/// Reusable displacement pipeline compiled for the borrowed GPU context.
/// Retain this operation to reuse the pipeline; no process-wide device cache
/// participates in construction or dispatch.
pub struct MeshDisplacement<'context> {
    context: &'context WgpuContext,
    pipeline: ComputePipeline,
}
impl<'context> MeshDisplacement<'context> {
    pub fn new(context: &'context WgpuContext) -> Result<Self, crate::MeshDisplacementError> {
        let pipeline = ComputePipeline::from_source(context, source())?;
        Ok(Self { context, pipeline })
    }
    pub fn apply(
        &self,
        source: GpuMesh,
        depth: Texture2d,
        projection: DisplacementProjection,
        scale: DisplacementScale,
    ) -> Result<GpuMesh, crate::MeshDisplacementError> {
        let context = self.context;
        let pos_core = &source.positions;
        let norm_core = &source.normals;
        let uv_core = &source.tex_coords;

        let out_pos_buf = Buffer::new(
            context,
            pos_core.length(),
            BufferDefinition::storage()
                .with_label(("Displaced Positions").into())
                .with_usage(BufferUse::Vertex),
        )?;

        let mut parameters = PassParameters::new();
        parameters.insert(
            PassParameterName::from("pos"),
            PassParameter::from(pos_core.clone()),
        );
        parameters.insert(
            PassParameterName::from("normals"),
            PassParameter::from(norm_core.clone()),
        );
        parameters.insert(
            PassParameterName::from("uvs"),
            PassParameter::from(uv_core.clone()),
        );
        parameters.insert(
            PassParameterName::from("depth_tex"),
            PassParameter::from(depth),
        );
        parameters.insert(
            PassParameterName::from("depth_samp"),
            PassParameter::from(Sampler::new(context, None, None, None, None, None)),
        );

        projection.bind(&mut parameters);
        scale.bind(&mut parameters);

        parameters.insert(
            PassParameterName::from("out_pos"),
            PassParameter::from(out_pos_buf.clone()),
        );

        let grid = DisplacementPositionSpan::from(pos_core.length()).grid();
        ComputePass::dispatch(context, self.pipeline.clone(), parameters, grid)?;

        Ok(GpuMesh {
            positions: out_pos_buf,
            normals: source.normals,
            tex_coords: source.tex_coords,
            indices: source.indices,
            joints: source.joints,
            weights: source.weights,
            vertex_count: source.vertex_count,
            topology: source.topology,
            front_face: source.front_face,
        })
    }
}

fn source() -> ShaderSource {
    ShaderSource::from(
        r#"
                struct Params {
                    projection:     mat4x4<f32>,
                    inv_projection: mat4x4<f32>,
                    strength:       f32,
                    padding:        vec3<f32>,
                }

                @group(0) @binding(0) var<storage, read>       pos:       array<f32>;
                @group(0) @binding(1) var<storage, read>       normals:   array<f32>;
                @group(0) @binding(2) var<storage, read>       uvs:       array<f32>;
                @group(0) @binding(3) var depth_tex:           texture_2d<f32>;
                @group(0) @binding(4) var depth_samp:          sampler;
                @group(0) @binding(5) var<uniform>             params:    Params;
                @group(0) @binding(6) var<storage, read_write> out_pos:   array<f32>;

                @compute @workgroup_size(64)
                fn main(@builtin(global_invocation_id) id: vec3<u32>) {
                    let idx     = id.x;
                    let f32_idx = idx * 3u;
                    let uv_idx  = idx * 2u;

                    let vertex_count = arrayLength(&pos) / 3u;
                    if (idx >= vertex_count) { return; }

                    let p  = vec3<f32>(pos[f32_idx], pos[f32_idx+1u], pos[f32_idx+2u]);
                    let n  = vec3<f32>(normals[f32_idx], normals[f32_idx+1u], normals[f32_idx+2u]);
                    let uv = vec2<f32>(uvs[uv_idx], uvs[uv_idx+1u]);

                    let depth = textureSampleLevel(depth_tex, depth_samp, uv, 0.0).r;

                    let ndc_pos = vec4<f32>(uv.x * 2.0 - 1.0, (1.0 - uv.y) * 2.0 - 1.0, depth, 1.0);
                    let far_ndc = vec4<f32>(uv.x * 2.0 - 1.0, (1.0 - uv.y) * 2.0 - 1.0, 1.0,   1.0);

                    let view_pos_h = params.inv_projection * ndc_pos;
                    let far_pos_h  = params.inv_projection * far_ndc;

                    var view_pos = vec3<f32>(0.0);
                    if (abs(view_pos_h.w) > 1e-6) {
                        view_pos = view_pos_h.xyz / view_pos_h.w;
                    }

                    var far_pos = vec3<f32>(0.0);
                    if (abs(far_pos_h.w) > 1e-6) {
                        far_pos = far_pos_h.xyz / far_pos_h.w;
                    }

                    let displacement_vec = (view_pos - far_pos) * params.strength;

                    let displaced_p = p + displacement_vec;

                    out_pos[f32_idx]     = displaced_p.x;
                    out_pos[f32_idx+1u]  = displaced_p.y;
                    out_pos[f32_idx+2u]  = displaced_p.z;
                }
            "#,
    )
}

#[cfg(test)]
mod tests;
