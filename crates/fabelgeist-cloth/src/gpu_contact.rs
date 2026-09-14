//! Swept cloth surface contacts on the GPU: the device-side counterpart of
//! [`crate::surface_contact::SurfaceContacts`], run between solver substeps
//! without reading a particle back.
//!
//! The host solve projects one pair at a time, each seeing the corrections
//! before it. A dispatch guarantees no such order, so here every candidate
//! pair is resolved from the same positions, and each particle moves by the
//! largest push it received along each axis in each direction. Iterations
//! close what that leaves.
//!
//! Hierarchies over the swept cloth faces and edges are rebuilt before every
//! pass, as the host rebuilds its trees; the obstacle's are built once.

use std::collections::BTreeSet;
use std::sync::Arc;

use anyhow::ensure;
use fabelgeist_bvh::gpu::{BvhKernels, GpuBvh, TraversalConfig};
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
use fabelgeist_xpbd::Particles;

mod wgsl;

/// Items per dispatch. Every item walks a hierarchy, and a dispatch that runs
/// for seconds is what the display driver's watchdog resets the device over.
const CHUNK: u32 = 16_384;
/// Conservative advancement steps per candidate pair, as on the host. Running
/// out takes the last safe time as the contact, and that stale contact is still
/// projected: a primitive that was only passing close by gets pushed along an
/// old normal. With 32 steps, the fast approach of closing seams exhausted it
/// often enough to stretch a sewn garment to twice its edge lengths.
const SWEEP_BUDGET: u32 = 256;
/// Accumulator words per particle: extreme position and velocity corrections.
const WORDS_PER_PARTICLE: u64 = 12;

struct Kernels {
    swept_bounds: Arc<Kernel>,
    static_bounds: Arc<Kernel>,
    vertex_bounds: Arc<Kernel>,
    vertex_face: Arc<Kernel>,
    vertex_static_face: Arc<Kernel>,
    static_vertex_face: Arc<Kernel>,
    edge_edge: Arc<Kernel>,
    edge_static_edge: Arc<Kernel>,
    apply: Arc<Kernel>,
    bvh: BvhKernels,
}

impl Kernels {
    fn new(context: &WgpuContext, cache: &KernelCache) -> Result<Self> {
        Ok(Self {
            swept_bounds: cache.get(context, wgsl::SWEPT_BOUNDS)?,
            static_bounds: cache.get(context, wgsl::STATIC_BOUNDS)?,
            vertex_bounds: cache.get(context, wgsl::VERTEX_BOUNDS)?,
            vertex_face: cache.get(context, &wgsl::vertex_face())?,
            vertex_static_face: cache.get(context, &wgsl::vertex_static_face())?,
            static_vertex_face: cache.get(context, &wgsl::static_vertex_face())?,
            edge_edge: cache.get(context, &wgsl::edge_edge())?,
            edge_static_edge: cache.get(context, &wgsl::edge_static_edge())?,
            apply: cache.get(context, wgsl::APPLY)?,
            bvh: BvhKernels::with_cache(context, cache)?,
        })
    }
}

/// Where each list starts in the shared index buffer: cloth faces, cloth
/// edges, seam groups, obstacle faces, obstacle edges. One buffer, because a
/// contact kernel already binds the device's eight storage buffers.
#[derive(Clone, Copy, Default)]
struct Layout {
    edges: u32,
    groups: u32,
    static_faces: u32,
    static_edges: u32,
}

/// A fixed surface in the same swept solve, such as the wearer.
struct Obstacle {
    positions: Buffer,
    faces: GpuBvh,
    edges: GpuBvh,
    vertices: GpuBvh,
    clearance: f32,
}

pub struct GpuSurfaceContacts {
    kernels: Kernels,
    particle_count: u32,
    faces: Vec<[u32; 3]>,
    edges: Vec<[u32; 2]>,
    groups: Vec<u32>,
    topology: Buffer,
    layout: Layout,
    /// Interval start positions, then velocities.
    motion: Buffer,
    interval_start: Buffer,
    accumulator: Buffer,
    face_bounds: Buffer,
    edge_bounds: Buffer,
    face_bvh: GpuBvh,
    edge_bvh: GpuBvh,
    obstacle: Option<Obstacle>,
}

impl GpuSurfaceContacts {
    /// Seams name the copies of one physical vertex: faces and edges between
    /// copies never repel, even before the seam has closed.
    pub fn new(
        context: &WgpuContext,
        cache: &KernelCache,
        particle_count: u32,
        faces: &[[u32; 3]],
        seams: &[[u32; 2]],
    ) -> Result<Self> {
        ensure!(particle_count > 0, "GpuSurfaceContacts: no particles");
        ensure!(
            faces.iter().flatten().chain(seams.iter().flatten()).all(|&i| i < particle_count),
            "GpuSurfaceContacts: an index is past the {particle_count} particles"
        );
        let edges = unique_edges(faces);
        let groups = seam_groups(particle_count, seams);
        let kernels = Kernels::new(context, cache)?;
        let (topology, layout) = upload_topology(context, faces, &edges, &groups, &[], &[])?;
        let storage = BufferDefinition::storage();
        let particle_bytes = particle_count as u64 * 16;
        Ok(Self {
            face_bvh: GpuBvh::new(context, kernels.bvh.clone(), faces.len() as u32)?,
            edge_bvh: GpuBvh::new(context, kernels.bvh.clone(), edges.len() as u32)?,
            face_bounds: bounds_buffer(context, faces.len(), "contact face bounds")?,
            edge_bounds: bounds_buffer(context, edges.len(), "contact edge bounds")?,
            motion: Buffer::new(
                context,
                particle_bytes * 2,
                storage.clone().with_label("contact motion"),
            )?,
            interval_start: Buffer::new(
                context,
                particle_bytes,
                storage.clone().with_label("contact interval start"),
            )?,
            accumulator: Buffer::new(
                context,
                (particle_count as u64 * WORDS_PER_PARTICLE + 1) * 4,
                storage.with_label("contact corrections"),
            )?,
            kernels,
            particle_count,
            faces: faces.to_vec(),
            edges,
            groups,
            topology,
            layout,
            obstacle: None,
        })
    }

    /// Include fixed obstacle triangles in the swept solve. Their interiors and
    /// edges constrain the cloth even where no cloth vertex touches them.
    /// Empty input removes the obstacle.
    pub fn set_static_surface(
        &mut self,
        context: &WgpuContext,
        positions: &[Vec3],
        faces: &[[u32; 3]],
        clearance: f32,
    ) -> Result<()> {
        ensure!(
            clearance.is_finite() && clearance >= 0.0,
            "obstacle clearance must be finite and non-negative"
        );
        ensure!(
            faces.iter().flatten().all(|&i| (i as usize) < positions.len()),
            "an obstacle face indexes past its {} vertices",
            positions.len()
        );
        if positions.is_empty() || faces.is_empty() {
            self.obstacle = None;
            (self.topology, self.layout) =
                upload_topology(context, &self.faces, &self.edges, &self.groups, &[], &[])?;
            return Ok(());
        }
        let edges = unique_edges(faces);
        (self.topology, self.layout) =
            upload_topology(context, &self.faces, &self.edges, &self.groups, faces, &edges)?;
        let packed: Vec<f32> = positions.iter().flat_map(|p| [p.x, p.y, p.z, 0.0]).collect();
        let mut obstacle = Obstacle {
            positions: Buffer::from_slice(
                context,
                &packed,
                BufferDefinition::storage().with_label("contact obstacle positions"),
            )?,
            faces: GpuBvh::new(context, self.kernels.bvh.clone(), faces.len() as u32)?,
            edges: GpuBvh::new(context, self.kernels.bvh.clone(), edges.len() as u32)?,
            vertices: GpuBvh::new(context, self.kernels.bvh.clone(), positions.len() as u32)?,
            clearance,
        };
        let face_bounds = bounds_buffer(context, faces.len(), "obstacle face bounds")?;
        let edge_bounds = bounds_buffer(context, edges.len(), "obstacle edge bounds")?;
        let vertex_bounds = bounds_buffer(context, positions.len(), "obstacle vertex bounds")?;

        let mut batch = KernelBatch::labelled(context, "contact obstacle hierarchy");
        for (bounds, offset, arity, count) in [
            (&face_bounds, self.layout.static_faces, 3u32, faces.len() as u32),
            (&edge_bounds, self.layout.static_edges, 2, edges.len() as u32),
        ] {
            let mut parameters = PassParameters::new();
            parameters.insert("static_positions", obstacle.positions.clone());
            parameters.insert("topology", self.topology.clone());
            parameters.insert("primitive_bounds", bounds.clone());
            parameters.insert("count", count);
            parameters.insert("offset", offset);
            parameters.insert("arity", arity);
            parameters.insert("margin", 0.0f32);
            batch.dispatch_items(&self.kernels.static_bounds, &parameters, count)?;
        }
        let mut parameters = PassParameters::new();
        parameters.insert("static_positions", obstacle.positions.clone());
        parameters.insert("primitive_bounds", vertex_bounds.clone());
        parameters.insert("count", positions.len() as u32);
        parameters.insert("margin", 0.0f32);
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        batch.dispatch_items(&self.kernels.vertex_bounds, &parameters, positions.len() as u32)?;
        obstacle
            .faces
            .record_build(&mut batch, &face_bounds, faces.len() as u32)?;
        obstacle
            .edges
            .record_build(&mut batch, &edge_bounds, edges.len() as u32)?;
        obstacle
            .vertices
            .record_build(&mut batch, &vertex_bounds, positions.len() as u32)?;
        batch.submit();
        self.obstacle = Some(obstacle);
        Ok(())
    }

    /// Remember where the particles are now, so the next projection sweeps
    /// from here rather than from the start of its last substep.
    pub fn record_interval_start(&self, batch: &mut KernelBatch, particles: &Particles) -> Result<()> {
        batch.copy_buffer(
            &particles.positions,
            &self.interval_start,
            self.particle_count as u64 * 16,
        )
    }

    /// Project swept contacts after a completed substep. Contacts change
    /// velocities by impulses along the contact normal, so sliding survives.
    /// Nothing is read back; the work is submitted and left queued.
    pub fn project(
        &mut self,
        context: &WgpuContext,
        particles: &Particles,
        thickness: f32,
        iterations: u32,
        from_interval_start: bool,
    ) -> Result<()> {
        let count = particles.count();
        ensure!(
            count == self.particle_count,
            "GpuSurfaceContacts: built for {} particles, given {count}",
            self.particle_count
        );
        if !thickness.is_finite() || thickness <= 0.0 || iterations == 0 {
            return Ok(());
        }
        let clearance = self.obstacle.as_ref().map_or(0.0, |o| o.clearance);
        let search_radius = thickness.max(clearance);
        let particle_bytes = count as u64 * 16;

        let mut batch = KernelBatch::labelled(context, "contact setup");
        batch.clear_buffer(&self.accumulator);
        let start = if from_interval_start {
            &self.interval_start
        } else {
            &particles.previous
        };
        batch.copy_buffer(start, &self.motion, particle_bytes)?;
        batch.submit();

        let mut contact = PassParameters::new();
        contact.insert("positions", particles.positions.clone());
        contact.insert("motion", self.motion.clone());
        contact.insert("accumulator", self.accumulator.clone());
        contact.insert("topology", self.topology.clone());
        if let Some(obstacle) = &self.obstacle {
            contact.insert("static_positions", obstacle.positions.clone());
        }
        contact.insert("particle_count", count);
        contact.insert("budget", SWEEP_BUDGET);
        contact.insert("thickness", thickness);
        contact.insert("static_clearance", clearance);
        contact.insert("search_radius", search_radius);
        contact.insert("edges_offset", self.layout.edges);
        contact.insert("groups_offset", self.layout.groups);
        contact.insert("static_faces_offset", self.layout.static_faces);
        contact.insert("static_edges_offset", self.layout.static_edges);

        let face_count = self.faces.len() as u32;
        let edge_count = self.edges.len() as u32;
        for _ in 0..iterations {
            // Vertices against faces, from the positions the last pass left.
            let mut batch = KernelBatch::labelled(context, "contact face hierarchy");
            self.record_velocities(&mut batch, particles);
            self.record_swept_bounds(&mut batch, particles, &self.face_bounds, 0, 3, face_count, search_radius)?;
            self.face_bvh
                .record_build(&mut batch, &self.face_bounds, face_count)?;
            batch.submit();
            dispatch_chunked(context, &self.kernels.vertex_face, &contact, &self.face_bvh, count)?;
            if let Some(obstacle) = &self.obstacle {
                dispatch_chunked(context, &self.kernels.vertex_static_face, &contact, &obstacle.faces, count)?;
                dispatch_chunked(context, &self.kernels.static_vertex_face, &contact, &obstacle.vertices, face_count)?;
            }
            self.apply(context, particles)?;

            // Edges against edges, from the vertex-corrected positions.
            let mut batch = KernelBatch::labelled(context, "contact edge hierarchy");
            self.record_velocities(&mut batch, particles);
            self.record_swept_bounds(
                &mut batch,
                particles,
                &self.edge_bounds,
                self.layout.edges,
                2,
                edge_count,
                search_radius,
            )?;
            self.edge_bvh
                .record_build(&mut batch, &self.edge_bounds, edge_count)?;
            batch.submit();
            dispatch_chunked(context, &self.kernels.edge_edge, &contact, &self.edge_bvh, edge_count)?;
            if let Some(obstacle) = &self.obstacle {
                dispatch_chunked(context, &self.kernels.edge_static_edge, &contact, &obstacle.edges, edge_count)?;
            }
            self.apply(context, particles)?;
        }
        Ok(())
    }

    /// Contacts projected since the last [`GpuSurfaceContacts::project`]
    /// began, summed over its iterations. Waits on the device.
    pub async fn contact_count(&self, context: &WgpuContext) -> Result<u32> {
        let words: Vec<i32> = self.accumulator.read(context).await?;
        Ok(words[(self.particle_count as u64 * WORDS_PER_PARTICLE) as usize] as u32)
    }

    fn record_velocities(&self, batch: &mut KernelBatch, particles: &Particles) {
        let bytes = self.particle_count as u64 * 16;
        batch.encoder().copy_buffer_to_buffer(
            &particles.velocities.buffer,
            0,
            &self.motion.buffer,
            bytes,
            bytes,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn record_swept_bounds(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        bounds: &Buffer,
        offset: u32,
        arity: u32,
        count: u32,
        margin: f32,
    ) -> Result<()> {
        let mut parameters = PassParameters::new();
        parameters.insert("positions", particles.positions.clone());
        parameters.insert("motion", self.motion.clone());
        parameters.insert("topology", self.topology.clone());
        parameters.insert("primitive_bounds", bounds.clone());
        parameters.insert("count", count);
        parameters.insert("offset", offset);
        parameters.insert("arity", arity);
        parameters.insert("margin", margin);
        batch.dispatch_items(&self.kernels.swept_bounds, &parameters, count)?;
        Ok(())
    }

    fn apply(&self, context: &WgpuContext, particles: &Particles) -> Result<()> {
        let mut parameters = PassParameters::new();
        parameters.insert("positions", particles.positions.clone());
        parameters.insert("velocities", particles.velocities.clone());
        parameters.insert("accumulator", self.accumulator.clone());
        parameters.insert("count", self.particle_count);
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        parameters.insert("pad2", 0u32);
        let mut batch = KernelBatch::labelled(context, "contact apply");
        batch.dispatch_items(&self.kernels.apply, &parameters, self.particle_count)?;
        batch.submit();
        Ok(())
    }
}

fn dispatch_chunked(
    context: &WgpuContext,
    kernel: &Kernel,
    parameters: &PassParameters,
    bvh: &GpuBvh,
    items: u32,
) -> Result<()> {
    if items == 0 || bvh.count() == 0 {
        return Ok(());
    }
    let mut parameters = parameters.clone();
    bvh.bind(&mut parameters, &TraversalConfig::default());
    parameters.insert("primitive_count", bvh.count());
    let mut first = 0;
    while first < items {
        let count = (items - first).min(CHUNK);
        parameters.insert("first", first);
        parameters.insert("count", count);
        let mut batch = KernelBatch::labelled(context, "surface contacts");
        batch.dispatch_items(kernel, &parameters, count)?;
        batch.submit();
        first += count;
    }
    Ok(())
}

fn bounds_buffer(context: &WgpuContext, count: usize, label: &str) -> Result<Buffer> {
    Buffer::new(
        context,
        count.max(1) as u64 * 32,
        BufferDefinition::storage().with_label(label),
    )
}

fn unique_edges(faces: &[[u32; 3]]) -> Vec<[u32; 2]> {
    let mut edges = BTreeSet::new();
    for face in faces {
        for k in 0..3 {
            let (a, b) = (face[k], face[(k + 1) % 3]);
            edges.insert([a.min(b), a.max(b)]);
        }
    }
    edges.into_iter().collect()
}

/// A representative per particle, shared by every copy a seam joins.
fn seam_groups(count: u32, seams: &[[u32; 2]]) -> Vec<u32> {
    fn find(parent: &mut [u32], mut i: u32) -> u32 {
        while parent[i as usize] != i {
            parent[i as usize] = parent[parent[i as usize] as usize];
            i = parent[i as usize];
        }
        i
    }
    let mut parent: Vec<u32> = (0..count).collect();
    for &[a, b] in seams {
        let (root_a, root_b) = (find(&mut parent, a), find(&mut parent, b));
        if root_a != root_b {
            parent[root_b as usize] = root_a;
        }
    }
    (0..count).map(|i| find(&mut parent, i)).collect()
}

fn upload_topology(
    context: &WgpuContext,
    faces: &[[u32; 3]],
    edges: &[[u32; 2]],
    groups: &[u32],
    static_faces: &[[u32; 3]],
    static_edges: &[[u32; 2]],
) -> Result<(Buffer, Layout)> {
    let mut data: Vec<u32> = faces.iter().flatten().copied().collect();
    let edges_offset = data.len() as u32;
    data.extend(edges.iter().flatten());
    let groups_offset = data.len() as u32;
    data.extend_from_slice(groups);
    let static_faces_offset = data.len() as u32;
    data.extend(static_faces.iter().flatten());
    let static_edges_offset = data.len() as u32;
    data.extend(static_edges.iter().flatten());
    let buffer = Buffer::from_slice(
        context,
        &data,
        BufferDefinition::storage().with_label("contact topology"),
    )?;
    Ok((
        buffer,
        Layout {
            edges: edges_offset,
            groups: groups_offset,
            static_faces: static_faces_offset,
            static_edges: static_edges_offset,
        },
    ))
}

#[cfg(test)]
mod tests;
