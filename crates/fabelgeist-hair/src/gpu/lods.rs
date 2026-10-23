//! [`GpuGroomer::lods`]: a groom grown on the GPU made ready to draw there.

use fabelgeist_compute::KernelBatch;
use fabelgeist_compute::ScanElementCount;
use std::sync::Arc;

use crate::gpu::error::Result;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_math::Vec3;
use fabelgeist_spatial_hash::CellScale;

use super::grow::params;
use super::head::GpuHeadCover;
use super::{GpuGroom, GpuGroomer, storage, upload};
use crate::cards::{CardAtlas, CardSettings};
use crate::lod::LodSettings;

/// One strand level on the GPU, in the shuffled order, as the renderer
/// draws it: positions with their sky visibility in `w`, and a
/// [`crate::lod::StrandInfo`] a strand.
pub struct GpuLevel {
    pub points: u32,
    pub count: u32,
    pub positions: Buffer,
    pub infos: Buffer,
}

/// One cut of cards: [`crate::cards::CardVertex`] vertices and a triangle
/// list.
pub struct GpuCards {
    pub cards: u32,
    pub segments: u32,
    pub vertices: Buffer,
    pub indices: Buffer,
}

impl GpuCards {
    pub fn quads(&self) -> u32 {
        self.cards * self.segments
    }

    pub fn index_count(&self) -> u32 {
        self.quads() * 6
    }
}

/// A groom ready to draw at any distance, all of it on the GPU: the GPU
/// counterpart of [`crate::HairLods`].
pub struct GpuLods {
    pub levels: Vec<GpuLevel>,
    pub cards: Vec<GpuCards>,
    pub atlas: Arc<CardAtlas>,
    pub bounds: [Vec3; 2],
    pub center: Vec3,
    /// How much hair covers each vertex of the head set with
    /// [`GpuGroomer::set_head_mesh`]: roots growing, hair over it.
    pub cover: Option<GpuHeadCover>,
}

impl GpuLods {
    pub fn strand_count(&self) -> usize {
        self.levels.first().map_or(0, |l| l.count as usize)
    }

    /// How many strands, levels and card levels there are.
    pub fn counts(&self) -> crate::lod::LodCounts {
        crate::lod::LodCounts {
            strands: self.strand_count(),
            levels: self.levels.len(),
            cards: self.cards.len(),
        }
    }
}

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
use prepare::{CardBuffers, CoverDensity};

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
        let count = groom.strands;
        let points = groom.points;
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

        // The strand levels.
        let mut counts = vec![points];
        for &p in &settings.strand_points {
            let p = p as u32;
            if p >= 2 && p < *counts.last().unwrap() && p <= 32 {
                counts.push(p);
            }
        }
        let mut levels = Vec::with_capacity(counts.len());
        for &n in &counts {
            let positions = storage(
                &context,
                (count as u64 * n as u64 * 16).into(),
                "hair level positions".into(),
            )?;
            let infos = storage(
                &context,
                (count as u64 * 16).into(),
                "hair level infos".into(),
            )?;
            let mut p = params! {
                "positions" => groom.positions.clone(),
                "strands" => groom.attributes.clone(),
                "order" => order.clone(),
                "grid_values" => visibility.clone(),
                "head_shape" => head.clone(),
                "level_positions" => positions.clone(),
                "infos" => infos.clone(),
                "points" => points,
                "level_points" => n,
                "body" => has_body,
                "pad0" => 0u32,
                "clearance" => 0.0012f32,
                "pad1" => 0u32,
                "pad2" => 0u32,
                "pad3" => 0u32,
            };
            occlusion.insert(&mut p, count);
            batch.dispatch_folded(&self.kernels.level, &p, count)?;
            levels.push(GpuLevel {
                points: n,
                count,
                positions,
                infos,
            });
        }

        // The cards, a cut at a time: each needs its number of cards back
        // to size its buffers.
        let mut cards = Vec::with_capacity(settings.cards.len());
        for card in &settings.cards {
            let (built, next) = self.record_cards(
                &context,
                batch,
                groom,
                card,
                center,
                occlusion,
                &visibility,
                &head,
                has_body,
                settings.atlas_variants,
            )?;
            cards.push(built);
            batch = next;
        }

        let cover = self.record_cover(&mut batch, groom, bounds)?;
        batch.submit()?;

        let [w, h] = settings.atlas_size;
        let key = (settings.atlas_variants, [w, h]);
        let atlas = match &self.atlas {
            Some((v, size, atlas)) if (*v, *size) == key => atlas.clone(),
            _ => {
                let atlas = Arc::new(CardAtlas::paint(settings.atlas_variants, w, h, 0xA71A5));
                self.atlas = Some((key.0, key.1, atlas.clone()));
                atlas
            }
        };
        Ok(GpuLods {
            levels,
            cards,
            atlas,
            bounds,
            center,
            cover,
        })
    }

    /// The box every point of `groom` is in, measured on the GPU.
    pub fn measure_bounds(&mut self, groom: &GpuGroom) -> Result<[Vec3; 2]> {
        let context = self.context.clone();
        let words = self
            .scratch
            .box_words
            .fit(&context, 32_u64.into(), "hair box".into())?;
        let mut batch = KernelBatch::labelled(&context, "hair box");
        batch.clear_buffer(&words);
        let count = groom.strands * groom.points;
        batch.dispatch_folded(
            &self.kernels.bounds,
            &params! {
                "positions" => groom.positions.clone(),
                "box_words" => words.clone(),
                "count" => count,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            count,
        )?;
        let read: Vec<(Buffer, u32)> = (0..6).map(|k| (words.clone(), k)).collect();
        let values = self.read_words(batch, &read)?;
        // Back from ordered words: a set top bit was a positive float.
        let float = |key: u32| {
            f32::from_bits(if key & 0x8000_0000 != 0 {
                key & 0x7FFF_FFFF
            } else {
                !key
            })
        };
        let v = |k: usize, least: bool| {
            if least {
                float(!values[k])
            } else {
                float(values[k])
            }
        };
        Ok([
            Vec3::new(v(0, true), v(1, true), v(2, true)),
            Vec3::new(v(3, false), v(4, false), v(5, false)),
        ])
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
            "points" => groom.points,
            "strength" => strength,
            "pad0" => 0u32,
            "pad1" => 0u32,
        };
        grid.insert(&mut p, groom.strands);
        batch.dispatch_folded(&self.kernels.splat, &p, groom.strands * (groom.points - 1))?;
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
        grid.insert(&mut p, groom.strands);
        batch.dispatch_folded(&self.kernels.densify, &p, grid.cells())?;
        Ok(())
    }

    /// One cut of cards. Records the locks' statistics, reads back how many
    /// cards there are, and records building them into a new batch, which
    /// it hands back.
    #[allow(clippy::too_many_arguments)]
    fn record_cards<'a>(
        &mut self,
        context: &'a fabelgeist_gpu::globals::WgpuContext,
        mut batch: KernelBatch<'a>,
        groom: &GpuGroom,
        card: &CardSettings,
        center: Vec3,
        grid: Grid,
        visibility: &Buffer,
        head: &Buffer,
        body: u32,
        variants: u32,
    ) -> Result<(GpuCards, KernelBatch<'a>)> {
        let count = groom.strands;
        let segments = card.segments.clamp(1, 31) as u32;
        let rows = segments + 1;
        let guide_of = self.scratch.card_guides.fit(
            context,
            (count as u64 * 4).into(),
            "hair card locks".into(),
        )?;
        self.record_map(
            &mut batch,
            &groom.roots,
            &groom.attributes,
            &guide_of,
            count,
            groom.root_bounds,
            card.size,
            1.0,
            0xCA4D,
        )?;
        let stride = 4 + rows as u64 * 4;
        let locks = self.scratch.locks.fit(
            context,
            (count as u64 * stride * 4).into(),
            "hair card lock stats".into(),
        )?;
        batch.clear_buffer(&locks);
        let card_params = |mut p: PassParameters| {
            p.insert("count", count);
            p.insert("points", groom.points);
            p.insert("segments", segments);
            p.insert("min_strands", card.min_strands as u32);
            p
        };
        batch.dispatch_folded(
            &self.kernels.card_accumulate,
            &card_params(params! {
                "positions" => groom.positions.clone(),
                "guide_of" => guide_of.clone(),
                "locks" => locks.clone(),
            }),
            count,
        )?;
        let flags = self.scratch.flags.fit(
            context,
            ((count as u64 + 1) * 4).into(),
            "hair card flags".into(),
        )?;
        batch.clear_buffer(&flags);
        batch.dispatch_folded(
            &self.kernels.card_flags,
            &card_params(params! {
                "locks" => locks.clone(),
                "flags" => flags.clone(),
            }),
            count,
        )?;
        self.ensure_scan(count + 1)?;
        self.scan.record(
            &mut batch,
            &flags,
            &self.scan_scratch,
            ScanElementCount::from_elements(count + 1),
        )?;
        let cards = self.read_word(batch, &flags, count)?;

        let mut batch = KernelBatch::labelled(context, "hair cards");
        batch.dispatch_folded(
            &self.kernels.card_spread,
            &card_params(params! {
                "positions" => groom.positions.clone(),
                "guide_of" => guide_of,
                "locks" => locks.clone(),
                "strands" => groom.attributes.clone(),
                "center_x" => center.x,
                "center_y" => center.y,
                "center_z" => center.z,
                "pad0" => 0u32,
            }),
            count,
        )?;
        let CardBuffers { vertices, indices } = CardBuffers::new(context, cards, rows, segments)?;
        let mut p = params! {
            "locks" => locks,
            "slots" => flags,
            "strands" => groom.attributes.clone(),
            "grid_values" => visibility.clone(),
            "head_shape" => head.clone(),
            "card_vertices" => vertices.clone(),
            "card_indices" => indices.clone(),
            "count" => count,
            "points" => groom.points,
            "segments" => segments,
            "min_strands" => card.min_strands as u32,
            "body" => body,
            "center_x" => center.x,
            "center_y" => center.y,
            "center_z" => center.z,
            "size" => card.size,
            "width_scale" => card.width_scale,
            "variants" => variants.max(1),
            "pad0" => 0u32,
            "pad1" => 0u32,
        };
        grid.insert(&mut p, count);
        // The grid's `count` is the cards kernel's too: the strands.
        p.insert("count", count);
        batch.dispatch_folded(&self.kernels.card_build, &p, count)?;
        Ok((
            GpuCards {
                cards,
                segments,
                vertices,
                indices,
            },
            batch,
        ))
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
        let count = groom.strands;
        let radius = 0.004f32;
        self.grid.ensure_capacity(&context, count)?;
        let recorded_grid = self.grid.record_build(
            batch,
            &groom.roots,
            count,
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
