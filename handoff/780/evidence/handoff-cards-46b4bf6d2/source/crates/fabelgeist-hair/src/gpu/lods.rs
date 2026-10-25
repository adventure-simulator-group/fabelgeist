//! [`GpuGroomer::lods`]: a groom grown on the GPU made ready to draw there.

use fabelgeist_compute::KernelBatch;

use crate::gpu::error::Result;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_math::Vec3;
use fabelgeist_spatial_hash::{CellScale, HashLayout};

use super::grow::params;
use super::head::GpuHeadCover;
use super::{GpuGroom, GpuGroomer, storage, upload};
use crate::lod::LodSettings;

mod atlas;
mod cards;
mod levels;
mod records;

pub use records::{GpuCards, GpuLevel, GpuLods};
use levels::LevelBake;
use cards::CardPass;

/// A grid over a box: its origin, cell and size in cells.
#[derive(Clone, Copy)]
struct Grid {
    lo: Vec3,
    cell: f32,
    dims: [u32; 3],
}

impl Grid {
    fn over(lo: Vec3, hi: Vec3, cell: f32) -> Self {
        let extent = hi - lo;
        Self {
            lo,
            cell,
            dims: [extent.x, extent.y, extent.z].map(|e| ((e / cell).ceil() as u32).max(1)),
        }
    }

    fn cells(&self) -> u32 {
        self.dims[0] * self.dims[1] * self.dims[2]
    }

    fn insert(&self, p: &mut PassParameters, count: u32) {
        p.insert("nx", self.dims[0]);
        p.insert("ny", self.dims[1]);
        p.insert("nz", self.dims[2]);
        p.insert("count", count);
        p.insert("lo_x", self.lo.x);
        p.insert("lo_y", self.lo.y);
        p.insert("lo_z", self.lo.z);
        p.insert("cell", self.cell);
    }
}

mod prepare;
use prepare::CoverDensity;

impl GpuGroomer {
    /// [`GpuGroomer::lods`], the coarse levels and cards kept off `body`
    /// (as `head_shape` lays it out), or off nothing without one.
    pub fn lods_against(
        &mut self,
        groom: &GpuGroom,
        center: Vec3,
        settings: &LodSettings,
        body: Option<&Buffer>,
    ) -> Result<GpuLods> {
        let context = self.context.clone();
        groom.ensure_device(&context)?;
        let atlas = self.atlas_for(settings.atlas)?;
        let count = groom.layout.native_strands();
        let has_body = u32::from(body.is_some());
        let head = match body {
            Some(body) => body.clone(),
            None => upload(&context, &[[0.0f32; 4]; 17], "hair no body".into())?,
        };
        let bounds = match groom.bounds {
            Some(bounds) => bounds,
            None => self.measure_bounds(groom)?,
        };
        let mut batch = KernelBatch::labelled(&context, "hair levels");

        let order = self.record_order(&context, &mut batch, groom)?;
        // Occlusion: the hair's optical density, the body opaque, and each
        // cell's sky visibility.
        let pad = Vec3::splat(0.02);
        let (lo, hi) = (bounds[0] - pad, bounds[1] + pad);
        let resolution = settings.occlusion.resolution.max(4) as u32;
        let occlusion = Grid::over(lo, hi, (hi - lo).max_component() / resolution as f32);
        let visibility = storage(
            &context,
            (occlusion.cells() as u64 * 4).into(),
            "hair visibility".into(),
        )?;
        self.record_density(
            &mut batch,
            groom,
            occlusion,
            settings.occlusion.strength,
            has_body,
            &head,
        )?;
        let density = self.scratch.density.fit(
            &context,
            (occlusion.cells() as u64 * 4).into(),
            "hair density".into(),
        )?;
        let mut p = params! {
            "density" => density.clone(),
            "visibility" => visibility.clone(),
            "directions" => settings.occlusion.directions.max(4) as u32,
            "steps" => resolution,
            "pad0" => 0u32,
            "pad1" => 0u32,
        };
        occlusion.insert(&mut p, count);
        batch.dispatch_folded(&self.kernels.occlusion, &p, occlusion.cells())?;

        let levels = self.record_levels(
            &mut batch,
            LevelBake {
                groom,
                grid: occlusion,
                visibility: &visibility,
                order: &order,
                head: &head,
                body: has_body,
            },
            settings,
        )?;

        // The cards, a cut at a time: each needs its number of cards back
        // to size its buffers.
        let mut cards = Vec::with_capacity(settings.cards.len());
        for card in &settings.cards {
            let (built, next) = self.record_cards(
                &context,
                batch,
                CardPass {
                    groom,
                    settings: card,
                    center,
                    grid: occlusion,
                    visibility: &visibility,
                    head: &head,
                    body: has_body,
                    variants: atlas.variants(),
                },
            )?;
            cards.push(built);
            batch = next;
        }

        let cover = self.record_cover(&mut batch, groom, bounds)?;
        batch.submit()?;

        self.atlas = Some(atlas.clone());
        Ok(GpuLods {
            levels,
            cards,
            atlas,
            bounds,
            center,
            cover,
        })
    }

    /// Segment density into `grid`'s cells (optical density times
    /// `strength`), the body's cells opaque when there is one, into the
    /// density scratch.
    fn record_density(
        &mut self,
        batch: &mut KernelBatch,
        groom: &GpuGroom,
        grid: Grid,
        strength: f32,
        body: u32,
        head: &Buffer,
    ) -> Result<()> {
        let context = self.context.clone();
        let deposits = self.scratch.deposits.fit(
            &context,
            (grid.cells() as u64 * 4).into(),
            "hair deposits".into(),
        )?;
        batch.clear_buffer(&deposits);
        let mut p = params! {
            "positions" => groom.positions.clone(),
            "strands" => groom.attributes.clone(),
            "deposits" => deposits.clone(),
            "points" => groom.layout.native_points(),
            "strength" => strength,
            "pad0" => 0u32,
            "pad1" => 0u32,
        };
        grid.insert(&mut p, groom.layout.native_strands());
        batch.dispatch_folded(&self.kernels.splat, &p, groom.layout.segment_invocations())?;
        let density = self.scratch.density.fit(
            &context,
            (grid.cells() as u64 * 4).into(),
            "hair density".into(),
        )?;
        let mut p = params! {
            "deposits" => deposits,
            "head_shape" => head.clone(),
            "density" => density,
            "body" => body,
            "pad0" => 0u32,
            "pad1" => 0u32,
            "pad2" => 0u32,
        };
        grid.insert(&mut p, groom.layout.native_strands());
        batch.dispatch_folded(&self.kernels.densify, &p, grid.cells())?;
        Ok(())
    }


    /// How much hair covers each vertex of the head: the roots around it
    /// (on the spatial hash grid over the roots), the hair over it (a
    /// density grid of four millimetres), smoothed over the mesh.
    fn record_cover(
        &mut self,
        batch: &mut KernelBatch,
        groom: &GpuGroom,
        bounds: [Vec3; 2],
    ) -> Result<Option<GpuHeadCover>> {
        let Some(mesh) = &self.head_mesh else {
            return Ok(None);
        };
        let cover = mesh.allocate_cover(&self.context)?;
        let (vertices, head_points, head_normals, start, adjacency) = (
            mesh.layout.vertices(),
            mesh.points.clone(),
            mesh.normals.clone(),
            mesh.adjacency_start.clone(),
            mesh.adjacency.clone(),
        );
        let context = self.context.clone();
        let out = cover.buffer();
        let cover_bytes = out.size;
        if vertices.is_empty() {
            return Ok(Some(cover));
        }
        let count = groom.layout.native_strands();
        let radius = 0.004f32;
        let roots = groom.layout.hash_points();
        self.grid.ensure_capacity(
            &context,
            HashLayout::try_from(roots).map_err(anyhow::Error::new)?,
        )?;
        let recorded_grid = self.grid.record_build(
            batch,
            &groom.roots,
            roots,
            CellScale::from_cell_size(radius),
        )?;
        let root_counts = self.scratch.root_counts.fit(
            &context,
            vertices.word_bytes(),
            "hair cover roots".into(),
        )?;
        let totals =
            self.scratch
                .totals
                .fit(&context, 16_u64.into(), "hair cover totals".into())?;
        batch.clear_buffer(&totals);
        let mut p = params! {
            "head_points" => head_points.clone(),
            "root_counts" => root_counts.clone(),
            "totals" => totals.clone(),
            "count" => count,
            "pad" => 0u32,
            "vertices" => vertices,
            "radius" => radius,
            "pad0" => 0u32,
            "pad1" => 0u32,
        };
        recorded_grid.bind(&mut p);
        batch.dispatch_folded(&self.kernels.cover_count, &p, vertices.into())?;

        let reach = 0.024f32;
        let CoverDensity { grid, density } =
            self.record_cover_density(batch, groom, bounds, reach)?;
        let raw =
            self.scratch
                .cover
                .fit(&context, cover_bytes, "hair cover".into())?;
        let mut p = params! {
            "head_points" => head_points,
            "head_normals" => head_normals,
            "root_counts" => root_counts,
            "totals" => totals,
            "grid_values" => density,
            "cover" => raw.clone(),
            "vertices" => vertices,
            "reach" => reach,
            "pad0" => 0u32,
            "pad1" => 0u32,
        };
        grid.insert(&mut p, count);
        batch.dispatch_folded(&self.kernels.cover_finish, &p, vertices.into())?;

        // Smoothed over the mesh, three passes, into a buffer of its own.
        let spare = storage(&context, cover_bytes, "hair cover smoothing".into())?;
        let passes = [(&raw, &spare), (&spare, &raw), (&raw, out)];
        for (from, to) in passes {
            batch.dispatch_folded(
                &self.kernels.cover_smooth,
                &params! {
                    "adjacency_start" => start.clone(),
                    "adjacency" => adjacency.clone(),
                    "cover_in" => (*from).clone(),
                    "cover_out" => (*to).clone(),
                    "vertices" => vertices,
                    "pad0" => 0u32,
                    "pad1" => 0u32,
                    "pad2" => 0u32,
                },
                vertices.into(),
            )?;
        }
        Ok(Some(cover))
    }
}
