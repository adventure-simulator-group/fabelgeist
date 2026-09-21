//! Running a plate armor through the device: emit, weld, read back.

use fabelgeist_compute::{SortScratch, host_float};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::geometry::{self, VERTEX_FLOATS};
use super::plan::{Instance, Plan, PlannedPart};
use super::weld::KEY_WORDS;
use super::{PlateGpu, counted, device_error};
use crate::{Armor, ArmorMesh, ArmorPart};

/// Output triangles welded in one pass. A pass's key buffer is the largest,
/// at thirty-six bytes a vertex; this keeps it well inside a device's
/// smallest guaranteed storage binding.
const PASS_TRIANGLES: u32 = 1 << 19;

/// Bits of the hashes the weld sorts.
const HASH_BITS: u32 = 32;

/// Parts welded together in one device pass, with their instances rebased
/// to the pass.
struct Pass<'a> {
    parts: &'a [PlannedPart],
    instances: Vec<Instance>,
    first_triangle: u32,
    triangles: u32,
}

impl Plan {
    /// Consecutive runs of whole parts of at most [`PASS_TRIANGLES`] output
    /// triangles each, unless one part alone is larger.
    fn passes(&self) -> Vec<Pass<'_>> {
        let mut passes = Vec::new();
        let mut start = 0;
        while start < self.parts.len() {
            let first_triangle = self.parts[start].first_triangle;
            let end_of = |part: &PlannedPart| part.first_triangle + part.triangle_count;
            let mut end = start + 1;
            while end < self.parts.len()
                && end_of(&self.parts[end]) - first_triangle <= PASS_TRIANGLES
            {
                end += 1;
            }
            let instances = self
                .instances
                .iter()
                .filter(|instance| (start..end).contains(&(instance.part as usize)))
                .map(|instance| Instance {
                    first_output: instance.first_output - first_triangle,
                    part: instance.part - start as u32,
                    ..*instance
                })
                .collect();
            passes.push(Pass {
                parts: &self.parts[start..end],
                instances,
                first_triangle,
                triangles: end_of(&self.parts[end - 1]) - first_triangle,
            });
            start = end;
        }
        passes
    }
}

pub(super) fn build(gpu: &PlateGpu, a: &Armor) -> Result<Vec<ArmorPart>, String> {
    let plan = Plan::new(a, &gpu.tiles)?;
    let design = gpu.upload(&geometry::design(a))?;
    let sources = gpu.upload(&plan.sources)?;
    let mut parts = Vec::with_capacity(plan.parts.len());
    for pass in plan.passes() {
        match emit(gpu, &design, &sources, &pass)? {
            Some(emitted) => parts.extend(weld(gpu, &pass, emitted)?),
            None => parts.extend(pass.parts.iter().map(|part| empty(&part.name))),
        }
    }
    Ok(parts)
}

fn empty(name: &str) -> ArmorPart {
    ArmorPart {
        name: name.to_string(),
        mesh: ArmorMesh::default(),
    }
}

fn words(count: u32, per: usize) -> u64 {
    count as u64 * per as u64
}

/// A pass's triangles as emitted: three unwelded vertices each, and whether
/// each survived the mapping.
struct Emitted {
    vertices: Buffer,
    triangle_info: Buffer,
}

/// Emit every triangle of a pass; `None` when it has none.
fn emit(
    gpu: &PlateGpu,
    design: &Buffer,
    sources: &Buffer,
    pass: &Pass,
) -> Result<Option<Emitted>, String> {
    let triangles = pass.triangles;
    if triangles == 0 {
        return Ok(None);
    }
    let emitted = Emitted {
        vertices: gpu.scratch(words(triangles * 3, VERTEX_FLOATS), "plate vertices")?,
        triangle_info: gpu.scratch(triangles.into(), "plate triangle info")?,
    };
    let mut batch = gpu.batch("plate armor emit");
    let mut emit = PassParameters::new();
    emit.insert("count", triangles);
    emit.insert("instance_count", pass.instances.len() as u32);
    emit.insert(host_float::ZERO_FIELD, 0u32);
    emit.insert("pad0", 0u32);
    emit.insert("design", design.clone());
    emit.insert("sources", sources.clone());
    emit.insert("instances", gpu.upload(&pass.instances)?);
    emit.insert("vertices", emitted.vertices.clone());
    emit.insert("triangle_info", emitted.triangle_info.clone());
    gpu.dispatch(&mut batch, &gpu.emit, &emit, triangles)?;
    batch.submit();
    Ok(Some(emitted))
}

/// Each vertex's representative, and whether it is one.
struct Representatives {
    representative: Buffer,
    first: Buffer,
}

/// Find each vertex's representative: key, sort by hash, compare keys.
fn represent(gpu: &PlateGpu, emitted: &Emitted, slots: u32) -> Result<Representatives, String> {
    let keys = gpu.scratch(words(slots, KEY_WORDS), "plate weld keys")?;
    let hashes = gpu.scratch(slots.into(), "plate weld hashes")?;
    let order = gpu.scratch(slots.into(), "plate weld order")?;
    let found = Representatives {
        representative: gpu.scratch(slots.into(), "plate representatives")?,
        first: gpu.scratch(slots.into(), "plate first vertices")?,
    };
    let mut batch = gpu.batch("plate armor weld keys");
    let mut key = counted(slots);
    key.insert("vertices", emitted.vertices.clone());
    key.insert("triangle_info", emitted.triangle_info.clone());
    key.insert("keys", keys.clone());
    key.insert("hashes", hashes.clone());
    key.insert("slots", order.clone());
    gpu.dispatch(&mut batch, &gpu.keys, &key, slots)?;
    let mut sort_scratch = SortScratch::new(&gpu.context, slots).map_err(device_error)?;
    gpu.sort
        .record(
            &mut batch,
            &hashes,
            &order,
            &mut sort_scratch,
            slots,
            HASH_BITS,
        )
        .map_err(device_error)?;
    let mut represent = counted(slots);
    represent.insert("hashes", hashes);
    represent.insert("slots", order);
    represent.insert("keys", keys);
    represent.insert("triangle_info", emitted.triangle_info.clone());
    represent.insert("representative", found.representative.clone());
    represent.insert("first", found.first.clone());
    gpu.dispatch(&mut batch, &gpu.representatives, &represent, slots)?;
    batch.submit();
    Ok(found)
}

/// A pass's welded vertices and faces, and each part's counts of them.
struct Welded {
    vertices: Buffer,
    faces: Buffer,
    counts: Buffer,
}

/// Number the representatives and compact the surviving triangles.
fn compact(
    gpu: &PlateGpu,
    pass: &Pass,
    emitted: Emitted,
    found: Representatives,
) -> Result<Welded, String> {
    let triangles = pass.triangles;
    let slots = triangles * 3;
    let part_count = pass.parts.len() as u32;
    let local = |part: &PlannedPart| part.first_triangle - pass.first_triangle;
    let part_slots = pass.parts.iter().map(|p| local(p) * 3).collect::<Vec<_>>();
    let part_triangles = pass
        .parts
        .iter()
        .flat_map(|part| [local(part), part.triangle_count])
        .collect::<Vec<_>>();
    let rank = gpu.inclusive_scan(&found.first)?;
    let welded = Welded {
        vertices: gpu.scratch(words(slots, VERTEX_FLOATS), "plate welded vertices")?,
        faces: gpu.scratch(words(triangles, 3), "plate faces")?,
        counts: gpu.scratch(words(part_count, 2), "plate part counts")?,
    };
    let corners = gpu.scratch(words(triangles, 3), "plate welded corners")?;
    let kept = gpu.scratch(triangles.into(), "plate kept triangles")?;
    let mut batch = gpu.batch("plate armor weld");
    let mut copy = counted(slots);
    copy.insert("vertices", emitted.vertices);
    copy.insert("first", found.first);
    copy.insert("rank", rank.clone());
    copy.insert("welded", welded.vertices.clone());
    gpu.dispatch(&mut batch, &gpu.weld_vertices, &copy, slots)?;
    let mut faces = counted(triangles);
    faces.insert("triangle_info", emitted.triangle_info);
    faces.insert("part_slots", gpu.upload(&part_slots)?);
    faces.insert("representative", found.representative);
    faces.insert("rank", rank.clone());
    faces.insert("corners", corners.clone());
    faces.insert("kept", kept.clone());
    gpu.dispatch(&mut batch, &gpu.weld_faces, &faces, triangles)?;
    batch.submit();

    let kept_rank = gpu.inclusive_scan(&kept)?;
    let mut batch = gpu.batch("plate armor compact");
    let mut compact = counted(triangles);
    compact.insert("corners", corners);
    compact.insert("kept", kept);
    compact.insert("kept_rank", kept_rank.clone());
    compact.insert("faces", welded.faces.clone());
    gpu.dispatch(&mut batch, &gpu.compact, &compact, triangles)?;
    let mut ranges = counted(part_count);
    ranges.insert("part_triangles", gpu.upload(&part_triangles)?);
    ranges.insert("rank", rank);
    ranges.insert("kept_rank", kept_rank);
    ranges.insert("counts", welded.counts.clone());
    gpu.dispatch(&mut batch, &gpu.ranges, &ranges, part_count)?;
    batch.submit();
    Ok(welded)
}

/// Weld a pass's emitted triangles and read its parts back.
fn weld(gpu: &PlateGpu, pass: &Pass, emitted: Emitted) -> Result<Vec<ArmorPart>, String> {
    let found = represent(gpu, &emitted, pass.triangles * 3)?;
    let welded = compact(gpu, pass, emitted, found)?;
    let counts: Vec<[u32; 2]> = gpu.read(&welded.counts)?;
    let vertices: Vec<[f32; VERTEX_FLOATS]> = gpu.read(&welded.vertices)?;
    let faces: Vec<[u32; 3]> = gpu.read(&welded.faces)?;
    let mut vertex_start = 0;
    let mut face_start = 0;
    Ok(pass
        .parts
        .iter()
        .zip(counts)
        .map(|(part, [vertex_count, face_count])| {
            let vertex_end = vertex_start + vertex_count as usize;
            let face_end = face_start + face_count as usize;
            let part = ArmorPart {
                name: part.name.clone(),
                mesh: mesh(
                    &vertices[vertex_start..vertex_end],
                    faces[face_start..face_end].to_vec(),
                ),
            };
            vertex_start = vertex_end;
            face_start = face_end;
            part
        })
        .collect())
}

fn mesh(vertices: &[[f32; VERTEX_FLOATS]], faces: Vec<[u32; 3]>) -> ArmorMesh {
    ArmorMesh {
        positions: vertices.iter().map(|v| [v[0], v[1], v[2]]).collect(),
        normals: vertices.iter().map(|v| [v[3], v[4], v[5]]).collect(),
        uvs: vertices.iter().map(|v| [v[6], v[7]]).collect(),
        faces,
    }
}
