//! Anatomical surfaces selected from the wearer's body on the device.
//!
//! The bracer and the breastplate each fit to a patch of skin: the body
//! faces most of whose corners a family-specific rule supports. The patch's
//! vertices split at atlas seams, one surface vertex per (body vertex, atlas
//! coordinate) pair, numbered in the order the selected faces first use
//! them. Which faces are selected depends on where the body
//! is, so the device selects; which pairs exist depends on the body's
//! topology alone, so the host lists them once as a [`SeamTopology`].
//!
//! Selection keeps that numbering exactly: one invocation per face
//! marks it and claims each of its pairs' first corner, then one workgroup
//! walks the faces in order, 256 at a time, and compacts them with a
//! prefix sum. Everything that follows reads counts from the device.

use fabelgeist_gpu::prelude::BufferUpload;
use std::collections::HashMap;
use std::sync::Arc;

use fabelgeist_compute::{Kernel, KernelBatch};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::{ArmorGpu, device_error, wgsl};
use crate::GenerateError;

/// Words before the vertex list of a [`DeviceSurface`]: the vertex count,
/// the face count, and padding.
pub const SURFACE_HEADER: u32 = 4;

/// The bits `wgsl::STATUS` raises, as the host reads them.
pub const STATUS_INVALID_SURFACE: u32 = 1;
pub const STATUS_DEGENERATE: u32 = 2;

/// Set when a selection keeps no face at all.
pub const STATUS_EMPTY_SELECTION: u32 = 8;

/// Which (body vertex, atlas coordinate) pair every face corner uses.
#[derive(Clone, Debug)]
pub struct SeamTopology {
    corner_pairs: Vec<u32>,
    /// Body vertex and atlas coordinate of each pair.
    pairs: Vec<[u32; 2]>,
    face_count: u32,
}

impl SeamTopology {
    /// Pair every face corner with its atlas corner.
    pub fn new(faces: &[[u32; 3]], atlas_faces: &[[u32; 3]]) -> Result<Self, GenerateError> {
        if faces.len() != atlas_faces.len() || faces.is_empty() {
            return Err(GenerateError::InvalidSurface);
        }
        let mut ids = HashMap::<(u32, u32), u32>::new();
        let mut pairs = Vec::new();
        let mut corner_pairs = Vec::with_capacity(faces.len() * 3);
        for (face, atlas) in faces.iter().zip(atlas_faces) {
            for corner in 0..3 {
                let key = (face[corner], atlas[corner]);
                let id = *ids.entry(key).or_insert_with(|| {
                    pairs.push([key.0, key.1]);
                    (pairs.len() - 1) as u32
                });
                corner_pairs.push(id);
            }
        }
        Ok(Self {
            corner_pairs,
            pairs,
            face_count: faces.len() as u32,
        })
    }

    pub fn face_count(&self) -> u32 {
        self.face_count
    }

    pub fn pair_count(&self) -> u32 {
        self.pairs.len() as u32
    }
}

/// A [`SeamTopology`] uploaded for selection.
#[derive(Clone, Debug)]
pub struct DeviceSeams {
    corner_pairs: Buffer,
    pairs: Buffer,
    face_count: u32,
    pair_count: u32,
}

impl DeviceSeams {
    pub fn new(gpu: &ArmorGpu, topology: &SeamTopology) -> Result<Self, GenerateError> {
        Ok(Self {
            corner_pairs: gpu.upload(BufferUpload::from_elements(&topology.corner_pairs))?,
            pairs: gpu.upload(BufferUpload::from_elements(&topology.pairs))?,
            face_count: topology.face_count,
            pair_count: topology.pair_count(),
        })
    }
}

/// A selected surface on the device, laid out in one `u32` buffer: the
/// [`SURFACE_HEADER`], then each surface vertex's body vertex and atlas
/// coordinate, then each selected face as three surface vertices.
#[derive(Clone, Debug)]
pub struct DeviceSurface {
    pub words: Buffer,
    pub vertex_capacity: u32,
    pub face_capacity: u32,
}

impl DeviceSurface {
    /// Word offset of the first face.
    pub fn faces_at(&self) -> u32 {
        SURFACE_HEADER + 2 * self.vertex_capacity
    }

    /// Record the selection of the faces of `body_faces` with at least two
    /// corners whose `support` word is nonzero. An empty selection raises
    /// [`STATUS_EMPTY_SELECTION`] in `status`.
    pub fn record(
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        seams: &DeviceSeams,
        body_faces: &Buffer,
        support: &Buffer,
        status: &Buffer,
    ) -> Result<Self, GenerateError> {
        let surface = Self {
            words: gpu.scratch(
                (SURFACE_HEADER + 2 * seams.pair_count + 3 * seams.face_count) as u64 * 4,
                "anatomical surface",
            )?,
            vertex_capacity: seams.pair_count,
            face_capacity: seams.face_count,
        };
        // Each face's selection, then each pair's surface vertex.
        let selection = gpu.scratch(
            (seams.face_count + seams.pair_count) as u64 * 4,
            "selected faces and their vertices",
        )?;
        let first = gpu.scratch(seams.pair_count as u64 * 4, "first pair corners")?;
        let mut parameters = PassParameters::new();
        parameters.insert("count", seams.face_count);
        parameters.insert("faces_at", surface.faces_at());
        parameters.insert("pad1", 0u32);
        parameters.insert("pad2", 0u32);
        parameters.insert("body_faces", body_faces.clone());
        parameters.insert("support", support.clone());
        parameters.insert("corner_pairs", seams.corner_pairs.clone());
        parameters.insert("pairs", seams.pairs.clone());
        parameters.insert("selected", selection);
        parameters.insert("first", first);
        parameters.insert("surface", surface.words.clone());
        parameters.insert("status", status.clone());
        let kernel = |entry: &str| -> Result<Arc<Kernel>, GenerateError> {
            gpu.cache()
                .get(gpu.context(), &source(entry).into())
                .map_err(device_error)
        };
        batch
            .dispatch_items(&*kernel(MARK)?, &parameters, seams.face_count)
            .map_err(device_error)?;
        batch
            .dispatch(&*kernel(COMPACT)?, &parameters, [1, 1, 1])
            .map_err(device_error)?;
        batch
            .dispatch_items(&*kernel(FACES)?, &parameters, seams.face_count)
            .map_err(device_error)?;
        Ok(surface)
    }
}

fn source(entry: &str) -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> body_faces: array<u32>;
@group(0) @binding(1) var<storage, read> support: array<u32>;
@group(0) @binding(2) var<storage, read> corner_pairs: array<u32>;
@group(0) @binding(3) var<storage, read> pairs: array<u32>;
@group(0) @binding(4) var<storage, read_write> selected: array<u32>;
@group(0) @binding(5) var<storage, read_write> first: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> surface: array<u32>;
@group(0) @binding(7) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    faces_at: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(8) var<uniform> params: Params;
{status_code}
const HEADER: u32 = {header}u;
const EMPTY_SELECTION: u32 = {empty}u;
// A corner claims its pair by the complement of its index, so that the
// largest claim, from a zeroed buffer, is the first corner.
const UNCLAIMED: u32 = 0xffffffffu;

{entry}
"#,
        status_code = wgsl::STATUS,
        header = SURFACE_HEADER,
        empty = STATUS_EMPTY_SELECTION,
    )
}

/// Select each face, and claim its pairs for its corners.
const MARK: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let f = id.x;
    if (f >= params.count) {
        return;
    }
    var supported = 0u;
    for (var c = 0u; c < 3u; c = c + 1u) {
        if (support[body_faces[f * 3u + c]] != 0u) {
            supported = supported + 1u;
        }
    }
    let keep = supported >= 2u;
    selected[f] = select(0u, 1u, keep);
    if (keep) {
        for (var c = 0u; c < 3u; c = c + 1u) {
            atomicMax(&first[corner_pairs[f * 3u + c]], UNCLAIMED - (f * 3u + c));
        }
    }
}
"#;

/// One workgroup numbers the selected faces and their new pairs in face
/// order, 256 faces at a time.
const COMPACT: &str = r#"
var<workgroup> sums: array<vec2<u32>, 256>;
var<workgroup> base: vec2<u32>;

@compute @workgroup_size(256)
fn main(@builtin(local_invocation_index) t: u32) {
    if (t == 0u) {
        base = vec2<u32>(0u);
    }
    workgroupBarrier();
    let chunks = (params.count + 255u) / 256u;
    let vertices_at = HEADER;
    for (var chunk = 0u; chunk < chunks; chunk = chunk + 1u) {
        let f = chunk * 256u + t;
        var mine = vec2<u32>(0u);
        var fresh = 0u;
        if (f < params.count && selected[f] != 0u) {
            mine.x = 1u;
            for (var c = 0u; c < 3u; c = c + 1u) {
                let corner = f * 3u + c;
                if (atomicLoad(&first[corner_pairs[corner]]) == UNCLAIMED - corner) {
                    fresh = fresh | (1u << c);
                    mine.y = mine.y + 1u;
                }
            }
        }
        sums[t] = mine;
        workgroupBarrier();
        for (var offset = 1u; offset < 256u; offset = offset * 2u) {
            var before = vec2<u32>(0u);
            if (t >= offset) {
                before = sums[t - offset];
            }
            workgroupBarrier();
            sums[t] = sums[t] + before;
            workgroupBarrier();
        }
        let at = base + sums[t] - mine;
        if (mine.x != 0u) {
            // The selected face list is kept in `selected`, behind the flags
            // that are no longer read: slot k <= f.
            selected[at.x] = f | 0x80000000u;
        }
        var k = at.y;
        for (var c = 0u; c < 3u; c = c + 1u) {
            if ((fresh & (1u << c)) != 0u) {
                let pair = corner_pairs[f * 3u + c];
                surface[vertices_at + k * 2u] = pairs[pair * 2u];
                surface[vertices_at + k * 2u + 1u] = pairs[pair * 2u + 1u];
                selected[params.count + pair] = k;
                k = k + 1u;
            }
        }
        workgroupBarrier();
        if (t == 255u) {
            base = base + sums[255];
        }
        workgroupBarrier();
    }
    if (t == 0u) {
        surface[0] = base.y;
        surface[1] = base.x;
        if (base.x == 0u) {
            fail(EMPTY_SELECTION);
        }
    }
}
"#;

/// Each selected face, as surface vertices.
const FACES: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let k = id.x;
    if (k >= surface[1]) {
        return;
    }
    let f = selected[k] & 0x7fffffffu;
    for (var c = 0u; c < 3u; c = c + 1u) {
        surface[params.faces_at + k * 3u + c] = selected[params.count + corner_pairs[f * 3u + c]];
    }
}
"#;
