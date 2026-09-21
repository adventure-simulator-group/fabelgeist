use crate::{FrontFace, PrimitiveTopology};
use anyhow::Result;
use fabelgeist_gpu::data::{
    PassParameter,
    gpu::{buffer::Buffer, parameters::PassParameters, sampler::Sampler, texture::Texture2d},
    matrix::Mat4,
    vector::{Vec2, Vec3},
};
use fabelgeist_gpu::globals::WgpuContext;
use indexmap::IndexMap;
#[derive(Debug, Clone, PartialEq)]
pub struct GpuMesh {
    pub positions: Buffer,
    pub normals: Buffer,
    pub tex_coords: Buffer,
    pub indices: Option<Buffer>,
    pub joints: Option<Buffer>,
    pub weights: Option<Buffer>,
    pub vertex_count: u32,
    pub topology: PrimitiveTopology,
    pub front_face: FrontFace,
}

impl GpuMesh {
    pub fn plane(
        context: &WgpuContext,
        resolution: Option<Vec2>,
        right: Option<Vec3>,
        up: Option<Vec3>,
        normal: Option<Vec3>,
    ) -> Result<GpuMesh> {
        let res = resolution.unwrap_or_else(|| Vec2::new(1.0, 1.0));
        let r_vec = right.unwrap_or_else(|| Vec3::new(1.0, 0.0, 0.0));
        let u_vec = up.unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0));
        let n_vec = normal.unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0));

        let res_x = res.x.max(1.0) as u32;
        let res_y = res.y.max(1.0) as u32;

        let mut positions = Vec::with_capacity(((res_x + 1) * (res_y + 1) * 3) as usize);
        let mut normals = Vec::with_capacity(((res_x + 1) * (res_y + 1) * 3) as usize);
        let mut tex_coords = Vec::with_capacity(((res_x + 1) * (res_y + 1) * 2) as usize);
        let mut indices = Vec::with_capacity((res_x * res_y * 6) as usize);

        for y in 0..=res_y {
            for x in 0..=res_x {
                let u = x as f32 / res_x as f32;
                let v = y as f32 / res_y as f32;

                let ux = (u - 0.5) * 2.0;
                let uy = (v - 0.5) * 2.0;

                let p = r_vec * ux + u_vec * uy;

                positions.push(p.x);
                positions.push(p.y);
                positions.push(p.z);

                normals.push(n_vec.x);
                normals.push(n_vec.y);
                normals.push(n_vec.z);

                tex_coords.push(u);
                tex_coords.push(v);
            }
        }

        for y in 0..res_y {
            for x in 0..res_x {
                let i0 = y * (res_x + 1) + x;
                let i1 = i0 + 1;
                let i2 = (y + 1) * (res_x + 1) + x;
                let i3 = i2 + 1;

                indices.push(i0);
                indices.push(i1);
                indices.push(i2);

                indices.push(i1);
                indices.push(i3);
                indices.push(i2);
            }
        }

        let pos_buf = Buffer::from_slice(
            context,
            &positions,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Plane Positions")
                .with_vertex(),
        )?;

        let norm_buf = Buffer::from_slice(
            context,
            &normals,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Plane Normals")
                .with_vertex(),
        )?;

        let uv_buf = Buffer::from_slice(
            context,
            &tex_coords,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Plane TexCoords")
                .with_vertex(),
        )?;

        let index_buf = Buffer::from_slice(
            context,
            &indices,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Plane Indices")
                .with_index(),
        )?;

        Ok(GpuMesh {
            positions: pos_buf,
            normals: norm_buf,
            tex_coords: uv_buf,
            indices: Some(index_buf),
            joints: None,
            weights: None,
            vertex_count: (res_x + 1) * (res_y + 1),
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Cw,
        })
    }

    pub fn displacement(
        context: &WgpuContext,
        source: GpuMesh,
        depth: Texture2d,
        projection: Mat4,
        strength: f64,
    ) -> Result<GpuMesh> {
        use fabelgeist_gpu::data::gpu::compute::{ComputePass, ComputePipeline, ComputeShader};
        use std::sync::OnceLock;
        static DISPLACEMENT_PIPELINE: OnceLock<ComputePipeline> = OnceLock::new();

        let pos_core = &source.positions;
        let norm_core = &source.normals;
        let uv_core = &source.tex_coords;

        let out_pos_buf = Buffer::new(
            context,
            pos_core.size,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Displaced Positions")
                .with_vertex(),
        )?;

        let pipeline = DISPLACEMENT_PIPELINE.get_or_init(|| {
            let shader_code = r#"
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
            "#;
            let shader = ComputeShader::new(context, shader_code.to_string()).unwrap();
            ComputePipeline::new(context, shader).unwrap()
        });

        let buf = |core: Buffer| PassParameter::from(core);

        let inv_projection = projection.inverse().unwrap_or_default();

        let mut parameters = IndexMap::new();
        parameters.insert("pos".to_string(), buf(pos_core.clone()));
        parameters.insert("normals".to_string(), buf(norm_core.clone()));
        parameters.insert("uvs".to_string(), buf(uv_core.clone()));
        parameters.insert("depth_tex".to_string(), PassParameter::from(depth));
        parameters.insert(
            "depth_samp".to_string(),
            PassParameter::from(Sampler::new(context, None, None, None, None, None)?),
        );

        parameters.insert("projection".to_string(), PassParameter::from(projection));
        parameters.insert(
            "inv_projection".to_string(),
            PassParameter::from(inv_projection),
        );
        parameters.insert("strength".to_string(), PassParameter::from(strength as f32));

        parameters.insert("out_pos".to_string(), buf(out_pos_buf.clone()));

        let workgroups = (pos_core.size as u32 / 12 + 63) / 64;
        ComputePass::new(
            context,
            pipeline.clone(),
            PassParameters::from(parameters),
            workgroups,
            1,
            1,
        )?;

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

    pub fn r#box(context: &WgpuContext, size: Option<Vec3>) -> Result<GpuMesh> {
        Self::box_shape(context, size)
    }

    pub fn box_shape(context: &WgpuContext, size: Option<Vec3>) -> Result<GpuMesh> {
        let s = size.unwrap_or_else(|| Vec3::new(1.0, 1.0, 1.0));
        let h = s * 0.5;

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut indices = Vec::new();

        let add_face = |pos: &mut Vec<f32>,
                        norm: &mut Vec<f32>,
                        uv: &mut Vec<f32>,
                        ind: &mut Vec<u32>,
                        n: Vec3,
                        r: Vec3,
                        u: Vec3| {
            let start_idx = (pos.len() / 3) as u32;
            let center = Vec3::new(n.x * h.x, n.y * h.y, n.z * h.z);
            let r_vec = Vec3::new(r.x * h.x, r.y * h.y, r.z * h.z);
            let u_vec = Vec3::new(u.x * h.x, u.y * h.y, u.z * h.z);

            let p0 = center - r_vec - u_vec;
            let p1 = center + r_vec - u_vec;
            let p2 = center + r_vec + u_vec;
            let p3 = center - r_vec + u_vec;

            for p in &[p0, p1, p2, p3] {
                pos.push(p.x);
                pos.push(p.y);
                pos.push(p.z);
                norm.push(n.x);
                norm.push(n.y);
                norm.push(n.z);
            }

            uv.push(0.0);
            uv.push(0.0);
            uv.push(1.0);
            uv.push(0.0);
            uv.push(1.0);
            uv.push(1.0);
            uv.push(0.0);
            uv.push(1.0);

            ind.push(start_idx);
            ind.push(start_idx + 1);
            ind.push(start_idx + 2);
            ind.push(start_idx);
            ind.push(start_idx + 2);
            ind.push(start_idx + 3);
        };

        // Front face (+Z)
        add_face(
            &mut positions,
            &mut normals,
            &mut tex_coords,
            &mut indices,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        // Back face (-Z)
        add_face(
            &mut positions,
            &mut normals,
            &mut tex_coords,
            &mut indices,
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        // Top face (+Y)
        add_face(
            &mut positions,
            &mut normals,
            &mut tex_coords,
            &mut indices,
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
        );
        // Bottom face (-Y)
        add_face(
            &mut positions,
            &mut normals,
            &mut tex_coords,
            &mut indices,
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        );
        // Right face (+X)
        add_face(
            &mut positions,
            &mut normals,
            &mut tex_coords,
            &mut indices,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        // Left face (-X)
        add_face(
            &mut positions,
            &mut normals,
            &mut tex_coords,
            &mut indices,
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
        );

        let pos_buf = Buffer::from_slice(
            context,
            &positions,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Box Positions")
                .with_vertex(),
        )?;
        let norm_buf = Buffer::from_slice(
            context,
            &normals,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Box Normals")
                .with_vertex(),
        )?;
        let uv_buf = Buffer::from_slice(
            context,
            &tex_coords,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Box TexCoords")
                .with_vertex(),
        )?;
        let index_buf = Buffer::from_slice(
            context,
            &indices,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Box Indices")
                .with_index(),
        )?;

        Ok(GpuMesh {
            positions: pos_buf,
            normals: norm_buf,
            tex_coords: uv_buf,
            indices: Some(index_buf),
            joints: None,
            weights: None,
            vertex_count: 24,
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Cw,
        })
    }

    pub fn sphere(
        context: &WgpuContext,
        radius: Option<f64>,
        rings: Option<f64>,
        sectors: Option<f64>,
    ) -> Result<GpuMesh> {
        let r = radius.unwrap_or(0.5) as f32;
        let num_rings = rings.unwrap_or(16.0).max(2.0) as u32;
        let num_sectors = sectors.unwrap_or(16.0).max(3.0) as u32;

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut indices = Vec::new();

        let r_step = 1.0 / (num_rings as f32);
        let s_step = 1.0 / (num_sectors as f32);

        for y in 0..=num_rings {
            let phi = std::f32::consts::PI * (y as f32) * r_step;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            for x in 0..=num_sectors {
                let theta = 2.0 * std::f32::consts::PI * (x as f32) * s_step;
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();

                let nx = cos_theta * sin_phi;
                let ny = cos_phi;
                let nz = sin_theta * sin_phi;

                positions.push(nx * r);
                positions.push(ny * r);
                positions.push(nz * r);

                normals.push(nx);
                normals.push(ny);
                normals.push(nz);

                tex_coords.push(x as f32 * s_step);
                tex_coords.push(y as f32 * r_step);
            }
        }

        for y in 0..num_rings {
            for x in 0..num_sectors {
                let r0 = y * (num_sectors + 1) + x;
                let r1 = r0 + 1;
                let r2 = (y + 1) * (num_sectors + 1) + x;
                let r3 = r2 + 1;

                indices.push(r0);
                indices.push(r1);
                indices.push(r2);

                indices.push(r1);
                indices.push(r3);
                indices.push(r2);
            }
        }

        let vertex_count = (num_rings + 1) * (num_sectors + 1);

        let pos_buf = Buffer::from_slice(
            context,
            &positions,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Sphere Positions")
                .with_vertex(),
        )?;
        let norm_buf = Buffer::from_slice(
            context,
            &normals,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Sphere Normals")
                .with_vertex(),
        )?;
        let uv_buf = Buffer::from_slice(
            context,
            &tex_coords,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Sphere TexCoords")
                .with_vertex(),
        )?;
        let index_buf = Buffer::from_slice(
            context,
            &indices,
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label("Sphere Indices")
                .with_index(),
        )?;

        Ok(GpuMesh {
            positions: pos_buf,
            normals: norm_buf,
            tex_coords: uv_buf,
            indices: Some(index_buf),
            joints: None,
            weights: None,
            vertex_count,
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Ccw,
        })
    }
}
