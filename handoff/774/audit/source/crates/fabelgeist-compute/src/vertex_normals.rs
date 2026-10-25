//! Vertex normals of an indexed triangle mesh.
//!
//! A scatter of face normals into their vertices with atomics would sum them
//! in whatever order the device happens to run, and floating-point addition
//! does not commute: two runs could disagree in the last bit. Instead every
//! triangle corner writes its contribution to its own slot, a stable
//! [`RadixSort`] groups the corners by vertex -- keeping triangle order within
//! each vertex -- and one invocation per vertex sums its corners in that
//! order. The result is the one a host loop over the triangles produces.
//!
//! Both weighting modes validate face directions with the same normalization
//! contract. A small area alone does not make a resolved face unusable. This
//! check does not certify topology, intersections, or fitting clearance.

use crate::prelude::*;
#[cfg(test)]
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::BufferUse;
use std::sync::Arc;

/// How a triangle's normal counts toward the normal of each of its vertices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormalWeighting {
    /// By triangle area: the unnormalised face normal is summed.
    Area,
    /// By the corner's angle: the unit face normal times the corner angle.
    /// Independent of how finely the surface around a crease is split.
    Angle,
}

/// Set in [`VertexNormals::status`] when a triangle has no usable normal.
pub const DEGENERATE_TRIANGLE: u32 = 1;
/// Set when a vertex's summed normal vanishes, or it belongs to no triangle.
pub const VANISHED_NORMAL: u32 = 2;

const WORKGROUP: u32 = 256;

fn corners_code(weighting: NormalWeighting) -> String {
    let contribution = match weighting {
        NormalWeighting::Area => {
            r#"
    // Weighting must not change whether a face has a usable direction.
    // Validate with the same normalization check as angle weighting, while
    // retaining the unnormalized face for its area contribution.
    _ = unit(face);
    for (var corner = 0u; corner < 3u; corner = corner + 1u) {
        write_corner(triangle, corner, face);
    }
"#
        }
        NormalWeighting::Angle => {
            r#"
    let unit_face = unit(face);
    for (var corner = 0u; corner < 3u; corner = corner + 1u) {
        let here = points[corner];
        let a = unit(points[(corner + 1u) % 3u] - here);
        let b = unit(points[(corner + 2u) % 3u] - here);
        let angle = acos(clamp(dot(a, b), -1.0, 1.0));
        write_corner(triangle, corner, unit_face * angle);
    }
"#
        }
    };
    format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> triangles: array<u32>;
@group(0) @binding(2) var<storage, read_write> contributions: array<f32>;
@group(0) @binding(3) var<storage, read_write> keys: array<u32>;
@group(0) @binding(4) var<storage, read_write> corners: array<u32>;
@group(0) @binding(5) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(6) var<uniform> params: Params;

fn load_point(index: u32) -> vec3<f32> {{
    return vec3<f32>(positions[index * 3u], positions[index * 3u + 1u], positions[index * 3u + 2u]);
}}

fn unit(v: vec3<f32>) -> vec3<f32> {{
    let length = sqrt((v.x * v.x + v.y * v.y) + v.z * v.z);
    if (!(length > 1e-12) || length > 3.4e38) {{
        atomicOr(&status[0], 1u);
        return vec3<f32>(0.0);
    }}
    return v / length;
}}

fn write_corner(triangle: u32, corner: u32, contribution: vec3<f32>) {{
    let slot = triangle * 3u + corner;
    contributions[slot * 3u] = contribution.x;
    contributions[slot * 3u + 1u] = contribution.y;
    contributions[slot * 3u + 2u] = contribution.z;
    keys[slot] = triangles[slot];
    corners[slot] = slot;
}}

@compute @workgroup_size({WORKGROUP}u)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let triangle = global_id.x;
    if (triangle >= params.count) {{
        return;
    }}
    let points = array<vec3<f32>, 3>(
        load_point(triangles[triangle * 3u]),
        load_point(triangles[triangle * 3u + 1u]),
        load_point(triangles[triangle * 3u + 2u]),
    );
    let face = cross(points[1] - points[0], points[2] - points[0]);
    {contribution}
}}
"#
    )
}

fn ranges_code() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> keys: array<u32>;
@group(0) @binding(1) var<storage, read_write> starts: array<u32>;
@group(0) @binding(2) var<storage, read_write> ends: array<u32>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;

// Each run of equal keys marks its own first and one-past-last slot.
@compute @workgroup_size({WORKGROUP}u)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let slot = global_id.x;
    if (slot >= params.count) {{
        return;
    }}
    let key = keys[slot];
    if (slot == 0u || keys[slot - 1u] != key) {{
        starts[key] = slot;
    }}
    if (slot + 1u == params.count || keys[slot + 1u] != key) {{
        ends[key] = slot + 1u;
    }}
}}
"#
    )
}

fn gather_code() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> contributions: array<f32>;
@group(0) @binding(1) var<storage, read> corners: array<u32>;
@group(0) @binding(2) var<storage, read> starts: array<u32>;
@group(0) @binding(3) var<storage, read> ends: array<u32>;
@group(0) @binding(4) var<storage, read_write> normals: array<f32>;
@group(0) @binding(5) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(6) var<uniform> params: Params;

@compute @workgroup_size({WORKGROUP}u)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let vertex = global_id.x;
    if (vertex >= params.count) {{
        return;
    }}
    var sum = vec3<f32>(0.0);
    for (var slot = starts[vertex]; slot < ends[vertex]; slot = slot + 1u) {{
        let corner = corners[slot];
        sum = sum + vec3<f32>(
            contributions[corner * 3u],
            contributions[corner * 3u + 1u],
            contributions[corner * 3u + 2u],
        );
    }}
    let magnitude = sqrt((sum.x * sum.x + sum.y * sum.y) + sum.z * sum.z);
    var normal = vec3<f32>(0.0);
    if (magnitude > 1e-12 && magnitude <= 3.4e38) {{
        normal = sum / magnitude;
    }} else {{
        atomicOr(&status[0], 2u);
    }}
    normals[vertex * 3u] = normal.x;
    normals[vertex * 3u + 1u] = normal.y;
    normals[vertex * 3u + 2u] = normal.z;
}}
"#
    )
}

/// Parameters for a pass over `count` items; the pads fill out the uniform.
fn counted(count: u32) -> PassParameters {
    let mut parameters = PassParameters::new();
    parameters.insert("count", count);
    for pad in ["pad0", "pad1", "pad2"] {
        parameters.insert(pad, 0u32);
    }
    parameters
}

/// Buffers for the normals of meshes up to a vertex and triangle capacity.
#[derive(Clone, Debug)]
pub struct VertexNormals {
    /// Unit normals as `f32` triples. A vanished normal is written as zero.
    pub normals: Buffer,
    /// [`DEGENERATE_TRIANGLE`] and [`VANISHED_NORMAL`] bits, accumulated
    /// until cleared.
    pub status: Buffer,
    contributions: Buffer,
    keys: Buffer,
    corners: Buffer,
    starts: Buffer,
    ends: Buffer,
    sort: SortScratch,
    vertex_capacity: u32,
    triangle_capacity: u32,
}

impl VertexNormals {
    pub fn new(
        context: &WgpuContext,
        vertex_capacity: u32,
        triangle_capacity: u32,
    ) -> Result<Self> {
        let vertex_capacity = vertex_capacity.max(1);
        let triangle_capacity = triangle_capacity.max(1);
        let corner_capacity = triangle_capacity * 3;
        let storage = BufferDefinition::storage().with_usage(BufferUse::CopySource);
        let buffer = |bytes: u64, label: &str| {
            Buffer::new(
                context,
                (bytes).into(),
                storage.clone().with_label((label).into()),
            )
        };
        Ok(Self {
            normals: buffer(vertex_capacity as u64 * 12, "vertex normals")?,
            status: buffer(4, "vertex normal status")?,
            contributions: buffer(corner_capacity as u64 * 12, "corner normals")?,
            keys: buffer(corner_capacity as u64 * 4, "corner vertices")?,
            corners: buffer(corner_capacity as u64 * 4, "corners by vertex")?,
            starts: buffer(vertex_capacity as u64 * 4, "vertex corner starts")?,
            ends: buffer(vertex_capacity as u64 * 4, "vertex corner ends")?,
            sort: SortScratch::new(context, corner_capacity)?,
            vertex_capacity,
            triangle_capacity,
        })
    }

    /// Read the normals of the first `count` vertices, and the status bits.
    /// Stalls on the device.
    pub async fn read(&self, context: &WgpuContext, count: u32) -> Result<(Vec<[f32; 3]>, u32)> {
        let normals: Vec<[f32; 3]> = self.normals.read(context).await?;
        let status: Vec<u32> = self.status.read(context).await?;
        Ok((
            normals[..count.min(self.vertex_capacity) as usize].to_vec(),
            status[0],
        ))
    }
}

/// The compiled passes, for one weighting.
#[derive(Clone, Debug)]
pub struct VertexNormalKernels {
    corners: Arc<Kernel>,
    ranges: Arc<Kernel>,
    gather: Arc<Kernel>,
    sort: RadixSort,
}

impl VertexNormalKernels {
    pub fn new(context: &WgpuContext, weighting: NormalWeighting) -> Result<Self> {
        Self::with_cache(context, &KernelCache::new(), weighting)
    }

    pub fn with_cache(
        context: &WgpuContext,
        cache: &KernelCache,
        weighting: NormalWeighting,
    ) -> Result<Self> {
        Ok(Self {
            corners: cache.get(context, &corners_code(weighting))?,
            ranges: cache.get(context, &ranges_code())?,
            gather: cache.get(context, &gather_code())?,
            sort: RadixSort::with_cache(context, cache)?,
        })
    }

    /// Record the normals of `vertex_count` vertices of the triangles, `u32`
    /// triples indexing `positions`. Clears the status first.
    pub fn record(
        &self,
        batch: &mut KernelBatch,
        positions: &Buffer,
        triangles: &Buffer,
        vertex_count: u32,
        triangle_count: u32,
        output: &mut VertexNormals,
    ) -> Result<()> {
        if vertex_count > output.vertex_capacity || triangle_count > output.triangle_capacity {
            return Err(anyhow!(
                "VertexNormals: {vertex_count} vertices and {triangle_count} triangles exceed {} and {}",
                output.vertex_capacity,
                output.triangle_capacity
            ));
        }
        let corner_count = triangle_count * 3;
        batch.clear_buffer(&output.status);
        batch.clear_buffer(&output.starts);
        batch.clear_buffer(&output.ends);

        let mut corners = counted(triangle_count);
        corners.insert("positions", positions.clone());
        corners.insert("triangles", triangles.clone());
        corners.insert("contributions", output.contributions.clone());
        corners.insert("keys", output.keys.clone());
        corners.insert("corners", output.corners.clone());
        corners.insert("status", output.status.clone());
        batch.dispatch_items(&self.corners, &corners, triangle_count)?;

        let bits = u32::BITS - vertex_count.saturating_sub(1).leading_zeros();
        self.sort.record(
            batch,
            &output.keys,
            &output.corners,
            &mut output.sort,
            corner_count,
            bits.max(1),
        )?;

        let mut ranges = counted(corner_count);
        ranges.insert("keys", output.keys.clone());
        ranges.insert("starts", output.starts.clone());
        ranges.insert("ends", output.ends.clone());
        batch.dispatch_items(&self.ranges, &ranges, corner_count)?;

        let mut gather = counted(vertex_count);
        gather.insert("contributions", output.contributions.clone());
        gather.insert("corners", output.corners.clone());
        gather.insert("starts", output.starts.clone());
        gather.insert("ends", output.ends.clone());
        gather.insert("normals", output.normals.clone());
        gather.insert("status", output.status.clone());
        batch.dispatch_items(&self.gather, &gather, vertex_count)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A host loop over the triangles: the order the GPU sum reproduces.
    fn host_area_normals(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> Vec<[f32; 3]> {
        let mut sums = vec![[0.0f32; 3]; positions.len()];
        for t in triangles {
            let [a, b, c] = t.map(|i| positions[i as usize]);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            for i in t {
                for axis in 0..3 {
                    sums[*i as usize][axis] += n[axis];
                }
            }
        }
        sums.into_iter()
            .map(|n| {
                let m = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                n.map(|v| v / m)
            })
            .collect()
    }

    /// A wavy height field, so neighbouring normals all differ.
    fn surface(size: u32) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let mut positions = Vec::new();
        for y in 0..size {
            for x in 0..size {
                let (u, v) = (x as f32 / size as f32, y as f32 / size as f32);
                positions.push([u, v, 0.1 * (7.0 * u).sin() * (5.0 * v).cos()]);
            }
        }
        let mut triangles = Vec::new();
        for y in 0..size - 1 {
            for x in 0..size - 1 {
                let a = y * size + x;
                triangles.push([a, a + 1, a + size]);
                triangles.push([a + 1, a + size + 1, a + size]);
            }
        }
        (positions, triangles)
    }

    async fn normals(
        positions: &[[f32; 3]],
        triangles: &[[u32; 3]],
        weighting: NormalWeighting,
    ) -> Result<(Vec<[f32; 3]>, u32)> {
        let context = WgpuContext::new().await?;
        let kernels = VertexNormalKernels::new(&context, weighting)?;
        let definition = BufferDefinition::storage();
        let position_buffer = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(positions),
            definition.clone(),
        )?;
        let triangle_buffer =
            Buffer::from_upload(&context, BufferUpload::from_elements(triangles), definition)?;
        let mut output =
            VertexNormals::new(&context, positions.len() as u32, triangles.len() as u32)?;
        let mut batch = KernelBatch::new(&context);
        kernels.record(
            &mut batch,
            &position_buffer,
            &triangle_buffer,
            positions.len() as u32,
            triangles.len() as u32,
            &mut output,
        )?;
        batch.submit();
        output.read(&context, positions.len() as u32).await
    }

    #[tokio::test]
    async fn area_weighted_normals_match_a_host_sum() -> Result<()> {
        let (positions, triangles) = surface(70);
        let (found, status) = normals(&positions, &triangles, NormalWeighting::Area).await?;
        assert_eq!(status, 0);
        for (gpu, host) in found.iter().zip(host_area_normals(&positions, &triangles)) {
            for axis in 0..3 {
                assert!((gpu[axis] - host[axis]).abs() < 1e-5, "{gpu:?} vs {host:?}");
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn angle_weighting_ignores_how_a_crease_is_split() -> Result<()> {
        // Two faces meet along a crease at vertex 0; refining one of them must
        // not turn the crease normal toward it.
        let positions = [
            [0.0f32; 3],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.2, 0.0, 0.0],
        ];
        let coarse = [[0u32, 1, 2], [0, 3, 1]];
        let refined = [[0u32, 4, 2], [4, 1, 2], [0, 3, 1]];
        let (a, _) = normals(&positions[..4], &coarse, NormalWeighting::Angle).await?;
        let (b, _) = normals(&positions, &refined, NormalWeighting::Angle).await?;
        for axis in 0..3 {
            assert!((a[0][axis] - b[0][axis]).abs() < 1e-6);
        }
        assert!((a[0][1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert!((a[0][2] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        Ok(())
    }

    #[tokio::test]
    async fn small_resolved_faces_are_valid_for_both_weightings() -> Result<()> {
        // A clipped armor facet: its 3.8 micrometre edge remains resolved in
        // f32 at body-scale coordinates. Area weighting used to reject it
        // even though angle weighting could normalize the same face.
        let facet = [
            [-0.02812963, 1.2093203, 0.19875167],
            [-0.028126605, 1.2095866, 0.19873089],
            [-0.028129822, 1.2095873, 0.19872896],
        ];
        for scale in [1.0, 0.1] {
            let positions = facet.map(|p| p.map(|v| v * scale));
            let expected = host_area_normals(&positions, &[[0, 1, 2]]);
            for weighting in [NormalWeighting::Area, NormalWeighting::Angle] {
                let (found, status) = normals(&positions, &[[0, 1, 2]], weighting).await?;
                assert_eq!(status, 0, "{weighting:?} scale={scale}");
                for (found, expected) in found.iter().zip(&expected) {
                    for axis in 0..3 {
                        assert!((found[axis] - expected[axis]).abs() < 1e-5);
                    }
                }
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn degenerate_input_is_reported() -> Result<()> {
        // A zero-area triangle, and a vertex no triangle uses.
        let positions = [
            [0.0f32; 3],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [5.0, 5.0, 5.0],
        ];
        for weighting in [NormalWeighting::Area, NormalWeighting::Angle] {
            let (_, status) = normals(&positions, &[[0, 1, 2]], weighting).await?;
            assert_eq!(status, DEGENERATE_TRIANGLE | VANISHED_NORMAL);
        }
        Ok(())
    }
}
