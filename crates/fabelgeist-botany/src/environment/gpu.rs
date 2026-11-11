//! The environment passes as compute kernels.
//!
//! One step is two submissions: the light pass (hash build, shadow deposit
//! and propagation, per-module light) and, when modules were created, the
//! orientation pass (hash build over the grown set, pattern search per new
//! module). Each ends in one readback. Buffers are kept and grown by half
//! again when they run out, so a steady population allocates nothing.

use anyhow::Result;
use fabelgeist_compute::{Kernel, KernelBatch, KernelCache};
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferDefinition};
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_math::Vec4;
use fabelgeist_spatial_hash::gpu::{GpuHashGrid, HashKernels};
use std::sync::Arc;

use super::shadow::FIXED_POINT;
use super::{EnvironmentSettings, OrientRequest, Oriented, ShadowGrid, SphereSet, wgsl};
use fabelgeist_gpu::prelude::BufferUpload;

struct Kernels {
    light: Arc<Kernel>,
    splat: Arc<Kernel>,
    propagate: Arc<Kernel>,
    propagate_all: Arc<Kernel>,
    orient: Arc<Kernel>,
}

/// Layers at most this many cells go through the one-dispatch propagation.
const SINGLE_DISPATCH_LAYER: u32 = 1 << 16;

/// A buffer that grows to fit, by half again each time.
#[derive(Default)]
struct Growable {
    buffer: Option<Buffer>,
    label: &'static str,
}

impl Growable {
    fn new(label: &'static str) -> Self {
        Self {
            buffer: None,
            label,
        }
    }

    fn fit(&mut self, context: &WgpuContext, bytes: u64) -> Result<Buffer> {
        let bytes = bytes.max(16);
        if self.buffer.as_ref().is_none_or(|b| b.buffer.size() < bytes) {
            let grown = bytes.max(self.buffer.as_ref().map_or(0, |b| b.buffer.size() * 3 / 2));
            self.buffer = Some(Buffer::new(
                context,
                (grown).into(),
                BufferDefinition::storage().with_label((self.label).into()),
            )?);
        }
        let mut buffer = self.buffer.clone().expect("allocated above");
        buffer.size = (bytes).into();
        Ok(buffer)
    }
}

pub struct GpuEnvironment {
    context: WgpuContext,
    kernels: Kernels,
    grid: GpuHashGrid,
    points: Growable,
    parents: Growable,
    light: Growable,
    density: Growable,
    shadow: Growable,
    requests: Growable,
    rotations: Growable,
    crowding: Growable,
    solids: Growable,
}

impl GpuEnvironment {
    pub fn new(context: WgpuContext) -> Result<Self> {
        let cache = KernelCache::new();
        let kernels = Kernels {
            light: cache.get(&context, &wgsl::light())?,
            splat: cache.get(&context, &wgsl::splat())?,
            propagate: cache.get(&context, &wgsl::propagate())?,
            propagate_all: cache.get(&context, &wgsl::propagate_all())?,
            orient: cache.get(&context, &wgsl::orient())?,
        };
        let grid = GpuHashGrid::new(&context, HashKernels::new(&context, &cache)?, 1024)?;
        Ok(Self {
            context,
            kernels,
            grid,
            points: Growable::new("module spheres"),
            parents: Growable::new("module parents"),
            light: Growable::new("module light"),
            density: Growable::new("shadow deposits"),
            shadow: Growable::new("shadow"),
            requests: Growable::new("orientation requests"),
            rotations: Growable::new("orientation results"),
            crowding: Growable::new("orientation crowding"),
            solids: Growable::new("obstacle solids"),
        })
    }

    pub fn context(&self) -> &WgpuContext {
        &self.context
    }

    /// Upload the set and record the hash build over it.
    fn record_set(&mut self, batch: &mut KernelBatch, set: &SphereSet) -> Result<(Buffer, Buffer)> {
        let count = set.len() as u32;
        let points = self.points.fit(&self.context, count as u64 * 16)?;
        let parents = self.parents.fit(&self.context, count as u64 * 4)?;
        points.write(&self.context, BufferUpload::from_elements(&set.spheres))?;
        parents.write(&self.context, BufferUpload::from_elements(&set.parents))?;
        self.grid.ensure_capacity(&self.context, count)?;
        self.grid
            .record_build(batch, &points, count, set.cell_size())?;
        Ok((points, parents))
    }

    fn neighbour_parameters(
        &self,
        points: &Buffer,
        parents: &Buffer,
        count: u32,
    ) -> PassParameters {
        let mut parameters = PassParameters::new();
        parameters.insert("points", points.clone());
        parameters.insert("parents", parents.clone());
        self.grid.bind(&mut parameters);
        parameters.insert("count", count);
        parameters.insert("table_size", self.grid.table_size());
        parameters.insert("inverse_cell", self.grid.inverse_cell());
        parameters
    }

    pub async fn light(
        &mut self,
        set: &SphereSet,
        shadow: Option<&ShadowGrid>,
        settings: &EnvironmentSettings,
    ) -> Result<Vec<[f32; 2]>> {
        let count = set.len() as u32;
        if count == 0 {
            return Ok(Vec::new());
        }
        let context = self.context.clone();
        let mut batch = KernelBatch::labelled(&context, "botany light");
        let (points, parents) = self.record_set(&mut batch, set)?;

        let cells = shadow.map_or(1, |g| g.cell_count() as u64);
        let density = self.density.fit(&context, cells * 4)?;
        let field = self.shadow.fit(&context, cells * 4)?;
        batch.clear_buffer(&field);

        if let Some(grid) = shadow {
            match &grid.occluders {
                Some(occluders) if occluders.len() == grid.cell_count() => {
                    density.write(&context, BufferUpload::from_elements(occluders))?
                }
                _ => batch.clear_buffer(&density),
            }
            let [nx, ny, nz] = grid.dims;

            let mut splat = PassParameters::new();
            splat.insert("points", points.clone());
            splat.insert("density", density.clone());
            splat.insert("count", count);
            splat.insert("nx", nx);
            splat.insert("ny", ny);
            splat.insert("nz", nz);
            splat.insert("origin_x", grid.origin[0]);
            splat.insert("origin_y", grid.origin[1]);
            splat.insert("origin_z", grid.origin[2]);
            splat.insert("cell", grid.cell);
            splat.insert("strength", grid.strength);
            splat.insert("fixed_point", FIXED_POINT);
            splat.insert("inverse_cell", 1.0 / grid.cell);
            splat.insert("pad1", 0u32);
            batch.dispatch_folded(&self.kernels.splat, &splat, count)?;

            if nx * nz <= SINGLE_DISPATCH_LAYER {
                let mut propagate = PassParameters::new();
                propagate.insert("density", density.clone());
                propagate.insert("shadow", field.clone());
                propagate.insert("nx", nx);
                propagate.insert("nz", nz);
                propagate.insert("ny", ny);
                propagate.insert("pad", 0u32);
                propagate.insert("decay", grid.decay);
                propagate.insert("fixed_point", FIXED_POINT);
                propagate.insert("pad1", 0u32);
                propagate.insert("pad2", 0u32);
                batch.dispatch(&self.kernels.propagate_all, &propagate, [1, 1, 1])?;
            } else {
                for layer in (0..ny.saturating_sub(1)).rev() {
                    let mut propagate = PassParameters::new();
                    propagate.insert("density", density.clone());
                    propagate.insert("shadow", field.clone());
                    propagate.insert("nx", nx);
                    propagate.insert("nz", nz);
                    propagate.insert("layer", layer);
                    propagate.insert("pad", 0u32);
                    propagate.insert("decay", grid.decay);
                    propagate.insert("fixed_point", FIXED_POINT);
                    propagate.insert("pad1", 0u32);
                    propagate.insert("pad2", 0u32);
                    batch.dispatch_items(&self.kernels.propagate, &propagate, nx * nz)?;
                }
            }
        }

        let light = self.light.fit(&context, count as u64 * 8)?;
        let mut parameters = self.neighbour_parameters(&points, &parents, count);
        parameters.insert("shadow", field.clone());
        parameters.insert("light", light.clone());
        parameters.insert("sensitivity", settings.light_sensitivity);
        let (origin, inverse, dims, enabled) = match shadow {
            Some(g) => (g.origin, 1.0 / g.cell, g.dims, 1u32),
            None => ([0.0; 3], 1.0, [1, 1, 1], 0u32),
        };
        parameters.insert("origin_x", origin[0]);
        parameters.insert("origin_y", origin[1]);
        parameters.insert("origin_z", origin[2]);
        parameters.insert("inverse_shadow_cell", inverse);
        parameters.insert("nx", dims[0]);
        parameters.insert("ny", dims[1]);
        parameters.insert("nz", dims[2]);
        parameters.insert("shadow_enabled", enabled);
        batch.dispatch_folded(&self.kernels.light, &parameters, count)?;
        batch.submit();

        let values: Vec<[f32; 2]> = light.read(&context).await?;
        Ok(values)
    }

    pub async fn orient(
        &mut self,
        set: &SphereSet,
        requests: &[OrientRequest],
        settings: &EnvironmentSettings,
    ) -> Result<Vec<Oriented>> {
        let context = self.context.clone();
        let mut batch = KernelBatch::labelled(&context, "botany orientation");
        let (points, parents) = self.record_set(&mut batch, set)?;

        let count = requests.len() as u32;
        let request_buffer = self
            .requests
            .fit(&context, std::mem::size_of_val(requests) as u64)?;
        request_buffer.write(&context, BufferUpload::from_elements(requests))?;
        let rotations = self.rotations.fit(&context, count as u64 * 16)?;

        let mut parameters = self.neighbour_parameters(&points, &parents, count);
        parameters.insert("requests", request_buffer);
        parameters.insert("rotations", rotations.clone());
        let crowding = self.crowding.fit(&context, count as u64 * 4)?;
        parameters.insert("crowding_out", crowding.clone());
        // The scene's solids, or one that is nowhere.
        let nowhere = [fabelgeist_buildings::Solid {
            shape: u32::MAX,
            ..Default::default()
        }];
        let solids = match &set.obstacles {
            Some(o) if !o.index.solids.is_empty() => &o.index.solids[..],
            _ => &nowhere[..],
        };
        let solid_buffer = self
            .solids
            .fit(&context, std::mem::size_of_val(solids) as u64)?;
        solid_buffer.write(&context, BufferUpload::from_elements(solids))?;
        parameters.insert("solids", solid_buffer);
        // A workgroup per request, folded into two dimensions past the
        // 65535-group limit of one.
        let groups_x = count.min(65_535);
        let groups_y = count.div_ceil(groups_x);
        parameters.insert("epsilon", settings.orientation_epsilon);
        parameters.insert("steps", settings.orientation_steps);
        parameters.insert("groups_x", groups_x);
        parameters.insert("pad1", 0u32);
        parameters.insert("pad2", 0u32);
        batch.dispatch(&self.kernels.orient, &parameters, [groups_x, groups_y, 1])?;
        batch.submit();

        let values: Vec<[f32; 4]> = rotations.read(&context).await?;
        let crowding: Vec<f32> = crowding.read(&context).await?;
        Ok(values
            .into_iter()
            .zip(crowding)
            .map(|(q, crowding)| Oriented {
                rotation: Vec4::new(q[0], q[1], q[2], q[3]),
                crowding,
            })
            .collect())
    }
}
