//! A scalp on the GPU: what roots and guides are placed on -- cut there from
//! the head's field ([`GpuGroomer::cut_scalp`]), or uploaded -- kept for as
//! long as the hairline stays.

use crate::gpu::error::{HairGpuError, Result};
use fabelgeist_compute::KernelBatch;
use fabelgeist_compute::ScanElementCount;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_math::Vec3;

use super::grow::params;
use super::{GpuGroomer, head_shape, storage, upload};
use crate::head::Head;
use crate::surface::Surface;

mod layout;

/// A scalp mesh on the GPU: its triangles with their running area (to
/// place points by area), its vertices, and its open edge -- the hairline
/// -- as points two millimetres apart (the guides' row along it) and one
/// apart (how far in from it each vertex is).
pub struct GpuScalp {
    pub(super) triangles: u32,
    pub(super) vertices: u32,
    pub(super) area: f32,
    pub(super) tri_cdf: Buffer,
    pub(super) tri_index: Buffer,
    pub(super) points: Buffer,
    pub(super) normals: Buffer,
    pub(super) uvs: Buffer,
    pub(super) boundary: Buffer,
    pub(super) boundary_count: u32,
    pub(super) boundary_fine: Buffer,
    pub(super) boundary_fine_count: u32,
    /// A box the scalp is inside.
    pub bounds: [Vec3; 2],
}

impl GpuScalp {
    /// Triangles, and vertices.
    pub fn counts(&self) -> (usize, usize) {
        (self.triangles as usize, self.vertices as usize)
    }

    /// The whole area, square metres.
    pub fn area(&self) -> f32 {
        self.area
    }

    /// Points along the open edge (the hairline), two millimetres apart,
    /// and one.
    pub fn boundary_counts(&self) -> (usize, usize) {
        (
            self.boundary_count as usize,
            self.boundary_fine_count as usize,
        )
    }

    /// The scalp back on the host: its vertices (position, normal, UV) and
    /// triangles.
    pub fn read(&self, groomer: &GpuGroomer) -> Result<Surface> {
        let context = groomer.context();
        let n = self.vertices as usize;
        let four = |buffer: &Buffer| -> Result<Vec<[f32; 4]>> {
            let mut all: Vec<[f32; 4]> = pollster::block_on(buffer.read(context))?;
            all.truncate(n);
            Ok(all)
        };
        let positions = four(&self.points)?;
        let normals = four(&self.normals)?;
        let uvs = four(&self.uvs)?;
        let mut triangles: Vec<[u32; 4]> = pollster::block_on(self.tri_index.read(context))?;
        triangles.truncate(self.triangles as usize);
        let v = |p: &[f32; 4]| Vec3::new(p[0], p[1], p[2]);
        Ok(Surface::new(
            positions.iter().map(v).collect(),
            Some(normals.iter().map(v).collect()),
            Some(uvs.iter().map(|uv| [uv[0], uv[1]]).collect()),
            triangles.iter().map(|t| [t[0], t[1], t[2]]).collect(),
        ))
    }
}

impl GpuGroomer {
    /// `scalp` onto the GPU, for [`GpuGroomer::grow`] to root hair on.
    pub fn upload_scalp(&self, scalp: &Surface) -> Result<GpuScalp> {
        let context = &self.context;
        let mut tri_cdf = Vec::with_capacity(scalp.triangles.len());
        let mut tri_index = Vec::with_capacity(scalp.triangles.len());
        let mut total = 0.0;
        for t in &scalp.triangles {
            let [a, b, c] = t.map(|i| scalp.positions[i as usize]);
            total += (b - a).cross(c - a).length() * 0.5;
            tri_cdf.push(total);
            tri_index.push([t[0], t[1], t[2], 0]);
        }
        let four = |v: &[Vec3]| v.iter().map(|p| [p.x, p.y, p.z, 0.0]).collect::<Vec<_>>();
        let uvs: Vec<[f32; 4]> = scalp
            .uvs
            .iter()
            .map(|uv| [uv[0], uv[1], 0.0, 0.0])
            .collect();
        let boundary = scalp.boundary(0.002);
        let boundary_fine = scalp.boundary(0.001);
        let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
        for p in &scalp.positions {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
        Ok(GpuScalp {
            triangles: scalp.triangles.len() as u32,
            vertices: scalp.positions.len() as u32,
            area: scalp.area(),
            tri_cdf: upload(context, &tri_cdf, "hair scalp areas".into())?,
            tri_index: upload(context, &tri_index, "hair scalp triangles".into())?,
            points: upload(context, &four(&scalp.positions), "hair scalp points".into())?,
            normals: upload(context, &four(&scalp.normals), "hair scalp normals".into())?,
            uvs: upload(context, &uvs, "hair scalp uvs".into())?,
            boundary: upload(context, &four(&boundary), "hair scalp edge".into())?,
            boundary_count: boundary.len() as u32,
            boundary_fine: upload(
                context,
                &four(&boundary_fine),
                "hair scalp edge, fine".into(),
            )?,
            boundary_fine_count: boundary_fine.len() as u32,
            bounds: [lo, hi],
        })
    }

    /// `head`'s scalp, cut on the GPU as [`Head::scalp`] cuts it on the
    /// host -- surface nets over the head's field on a grid of `cell`
    /// metres, the part above the hairline -- to the same triangles and
    /// vertices in the same order. Two waits, for how many triangles and
    /// then how many vertices and hairline points.
    pub fn cut_scalp(&mut self, head: &Head, cell: f32) -> Result<GpuScalp> {
        let context = self.context.clone();
        let shape = upload(&context, &head_shape(head), "hair head shape".into())?;
        let layout = layout::ScalpGrid::new(head, cell);
        let (corners, cells, edges) = (layout.corners, layout.cells, layout.edges);
        let grid = |count| layout.parameters(head, count);
        let one = |count: u32| {
            params! {
                "count" => count,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            }
        };

        // The field, the cells' vertices, and how many triangles each grid
        // edge's quad keeps.
        let mut batch = KernelBatch::labelled(&context, "hair scalp cells");
        let values = self.scratch.nets_values.fit(
            &context,
            (corners as u64 * 4).into(),
            "hair scalp field".into(),
        )?;
        let mut p = grid(corners);
        p.insert("head_shape", shape.clone());
        p.insert("values", values.clone());
        batch.dispatch_folded(&self.kernels.nets_values, &p, corners)?;
        let cell_vertices = self.scratch.nets_cells.fit(
            &context,
            (cells as u64 * 16).into(),
            "hair scalp cells".into(),
        )?;
        let mut p = grid(cells);
        p.insert("head_shape", shape.clone());
        p.insert("values", values.clone());
        p.insert("cells", cell_vertices.clone());
        batch.dispatch_folded(&self.kernels.nets_cells, &p, cells)?;
        let counts = self.scratch.nets_counts.fit(
            &context,
            ((edges as u64 + 1) * 4).into(),
            "hair scalp quad counts".into(),
        )?;
        batch.clear_buffer(&counts);
        let mut p = grid(edges);
        p.insert("values", values.clone());
        p.insert("cells", cell_vertices.clone());
        p.insert("counts", counts.clone());
        batch.dispatch_folded(&self.kernels.nets_quads_count, &p, edges)?;
        self.ensure_scan(edges + 1)?;
        self.scan.record(
            &mut batch,
            &counts,
            &self.scan_scratch,
            ScanElementCount::from_elements(edges + 1),
        )?;
        let triangles = self.read_words(batch, &[(counts.clone(), edges)])?[0];
        if triangles == 0 {
            return Err(HairGpuError::EmptyScalp);
        }

        // The triangles in order, their vertices numbered as the host
        // numbers them, the running area, and the open edges.
        let mut batch = KernelBatch::labelled(&context, "hair scalp mesh");
        let tri_cells = self.scratch.nets_triangles.fit(
            &context,
            (triangles as u64 * 16).into(),
            "hair scalp triangle cells".into(),
        )?;
        let mut p = grid(edges);
        p.insert("values", values.clone());
        p.insert("cells", cell_vertices.clone());
        p.insert("offsets", counts.clone());
        p.insert("triangles", tri_cells.clone());
        batch.dispatch_folded(&self.kernels.nets_quads_emit, &p, edges)?;
        let first = self.scratch.nets_first.fit(
            &context,
            (cells as u64 * 4).into(),
            "hair scalp first uses".into(),
        )?;
        batch.clear_buffer(&first);
        let mut p = one(triangles);
        p.insert("triangles", tri_cells.clone());
        p.insert("first", first.clone());
        batch.dispatch_folded(&self.kernels.nets_first, &p, triangles)?;
        let uses = triangles * 3;
        let flags = self.scratch.nets_flags.fit(
            &context,
            ((uses as u64 + 1) * 4).into(),
            "hair scalp vertex flags".into(),
        )?;
        batch.clear_buffer(&flags);
        let cell_at = self.scratch.nets_cell_at.fit(
            &context,
            (uses as u64 * 4).into(),
            "hair scalp vertex cells".into(),
        )?;
        let mut p = one(cells);
        p.insert("first", first);
        p.insert("flags", flags.clone());
        p.insert("cell_at", cell_at.clone());
        batch.dispatch_folded(&self.kernels.nets_order, &p, cells)?;
        self.ensure_scan(uses + 1)?;
        self.scan.record(
            &mut batch,
            &flags,
            &self.scan_scratch,
            ScanElementCount::from_elements(uses + 1),
        )?;
        let vertex_of = self.scratch.nets_vertex_of.fit(
            &context,
            (cells as u64 * 4).into(),
            "hair scalp vertex numbers".into(),
        )?;
        // At most a vertex a use: sized before the count is known.
        let points = storage(
            &context,
            (uses as u64 * 16).into(),
            "hair scalp points".into(),
        )?;
        let normals = storage(
            &context,
            (uses as u64 * 16).into(),
            "hair scalp normals".into(),
        )?;
        let uvs = storage(&context, (uses as u64 * 16).into(), "hair scalp uvs".into())?;
        let mut p = one(uses);
        p.insert("head_shape", shape.clone());
        p.insert("cells", cell_vertices);
        p.insert("flags", flags.clone());
        p.insert("cell_at", cell_at);
        p.insert("vertex_of", vertex_of.clone());
        p.insert("scalp_points", points.clone());
        p.insert("scalp_normals", normals.clone());
        p.insert("scalp_uvs", uvs.clone());
        batch.dispatch_folded(&self.kernels.nets_vertices, &p, uses)?;
        let tri_index = storage(
            &context,
            (triangles as u64 * 16).into(),
            "hair scalp triangles".into(),
        )?;
        let areas = self.scratch.nets_areas.fit(
            &context,
            (triangles as u64 * 4).into(),
            "hair scalp triangle areas".into(),
        )?;
        let keys = self.scratch.nets_keys.fit(
            &context,
            (uses as u64 * 4).into(),
            "hair scalp edges".into(),
        )?;
        let ids = self.scratch.nets_ids.fit(
            &context,
            (uses as u64 * 4).into(),
            "hair scalp edge ids".into(),
        )?;
        let box_words = self
            .scratch
            .box_words
            .fit(&context, 32_u64.into(), "hair box".into())?;
        batch.clear_buffer(&box_words);
        let mut p = one(triangles);
        p.insert("triangles", tri_cells);
        p.insert("flags", flags.clone());
        p.insert("vertex_of", vertex_of);
        p.insert("scalp_points", points.clone());
        p.insert("tri_index", tri_index.clone());
        p.insert("areas", areas.clone());
        p.insert("edge_keys", keys.clone());
        p.insert("edge_ids", ids.clone());
        p.insert("box_words", box_words.clone());
        batch.dispatch_folded(&self.kernels.nets_triangles, &p, triangles)?;
        let tri_cdf = storage(
            &context,
            (triangles as u64 * 4).into(),
            "hair scalp areas".into(),
        )?;
        let mut p = one(triangles);
        p.insert("areas", areas);
        p.insert("tri_cdf", tri_cdf.clone());
        batch.dispatch(&self.kernels.nets_cdf, &p, [1, 1, 1].into())?;
        // The edges sorted by their vertices, as the host sorts its open
        // edges; how many points each open one gets at either spacing.
        self.sort_scratch.ensure(&context, uses)?;
        self.sort
            .record(&mut batch, &keys, &ids, &mut self.sort_scratch, uses, 32)?;
        let (step, fine_step) = (0.002f32, 0.001f32);
        let steps = self.scratch.nets_steps.fit(
            &context,
            ((uses as u64 + 1) * 4).into(),
            "hair scalp edge steps".into(),
        )?;
        let fine = self.scratch.nets_fine.fit(
            &context,
            ((uses as u64 + 1) * 4).into(),
            "hair scalp fine edge steps".into(),
        )?;
        batch.clear_buffer(&steps);
        batch.clear_buffer(&fine);
        batch.dispatch_folded(
            &self.kernels.nets_open,
            &params! {
                "edge_keys" => keys.clone(),
                "scalp_points" => points.clone(),
                "steps" => steps.clone(),
                "fine" => fine.clone(),
                "count" => uses,
                "pad0" => 0u32,
                "step" => step,
                "fine_step" => fine_step,
            },
            uses,
        )?;
        self.scan.record(
            &mut batch,
            &steps,
            &self.scan_scratch,
            ScanElementCount::from_elements(uses + 1),
        )?;
        self.scan.record(
            &mut batch,
            &fine,
            &self.scan_scratch,
            ScanElementCount::from_elements(uses + 1),
        )?;
        let mut words = vec![
            (flags, uses),
            (steps.clone(), uses),
            (fine.clone(), uses),
            (tri_cdf.clone(), triangles - 1),
        ];
        words.extend((0..6).map(|k| (box_words.clone(), k)));
        let read = self.read_words(batch, &words)?;
        let (vertices, boundary_count, boundary_fine_count) = (read[0], read[1], read[2]);
        if vertices >= super::wgsl::scalp::EDGE_VERTEX_LIMIT {
            return Err(HairGpuError::ScalpVertexLimit);
        }
        let area = f32::from_bits(read[3]);
        let bounds = layout::ordered_bounds(&read[4..10]);

        // The points along the hairline.
        let mut batch = KernelBatch::labelled(&context, "hair scalp edge");
        let boundary = storage(
            &context,
            (boundary_count as u64 * 16).into(),
            "hair scalp edge".into(),
        )?;
        let boundary_fine = storage(
            &context,
            (boundary_fine_count as u64 * 16).into(),
            "hair scalp edge, fine".into(),
        )?;
        for (offsets, out) in [(&steps, &boundary), (&fine, &boundary_fine)] {
            let mut p = one(uses);
            p.insert("edge_keys", keys.clone());
            p.insert("scalp_points", points.clone());
            p.insert("offsets", offsets.clone());
            p.insert("boundary", out.clone());
            batch.dispatch_folded(&self.kernels.nets_boundary, &p, uses)?;
        }
        batch.submit();

        Ok(GpuScalp {
            triangles,
            vertices,
            area,
            tri_cdf,
            tri_index,
            points: layout::trimmed(points, (vertices as u64 * 16).into()),
            normals: layout::trimmed(normals, (vertices as u64 * 16).into()),
            uvs: layout::trimmed(uvs, (vertices as u64 * 16).into()),
            boundary,
            boundary_count,
            boundary_fine,
            boundary_fine_count,
            bounds,
        })
    }
}
