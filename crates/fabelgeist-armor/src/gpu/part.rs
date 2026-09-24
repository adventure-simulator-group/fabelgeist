//! A plate part built on the device, from carrier to closed shell.
//!
//! The host lays the part out -- how many carrier vertices each shell has,
//! how they are connected, how each shell is thickened -- from the design
//! alone. The device does everything that depends on where the points are:
//! chart kernels write the carriers into one arena, fitting kernels move
//! them, and the shell stage below thickens every shell at once and computes
//! the final normals. Nothing is read back until [`PartBuild::read`].

use fabelgeist_compute::{KernelBatch, NormalWeighting, VertexNormals};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

mod layout;
mod shell_kernels;

use super::staging::{Staged, StagedResults, Staging};
use super::{ArmorGpu, device_error};
use crate::{ArmorComponent, ArmorHinge, GenerateError, PlateFace, SurfaceGrid};
pub(crate) use layout::{
    Extrusion, GridShape, HingeSlot, PartLayout, SHELL_MIRRORED, SHELL_WORDS, ShellSpec,
};
use shell_kernels::ShellKernels;

/// The device buffers of one part.
pub(crate) struct PartBuild {
    layout: PartLayout,
    pub carriers: Buffer,
    pub heights: Buffer,
    pub shells: Buffer,
    pub hinges: Buffer,
    pub status: Buffer,
    shell_of: Buffer,
    authored_carrier_triangles: Buffer,
    carrier_triangle_shells: Buffer,
    carrier_triangles: Buffer,
    sources: Buffer,
    authored_final_indices: Buffer,
    final_triangle_shells: Buffer,
    final_indices: Buffer,
    /// The outer wall, then the inner wall, one point per carrier each.
    walls: Buffer,
    positions: Buffer,
    area: VertexNormals,
    angle: VertexNormals,
    normals: VertexNormals,
    carrier_triangle_count: u32,
    hinge_count: u32,
    /// The frame that places the part once thickened, when its shells
    /// are evaluated in its own frame.
    pub placement: Option<Buffer>,
    final_triangle_count: u32,
}

/// Hinge records: origin and axis, each padded to four floats.
pub(crate) const HINGE_WORDS: u32 = 8;

impl PartBuild {
    pub(crate) fn new(
        gpu: &ArmorGpu,
        layout: PartLayout,
        hinges: u32,
    ) -> Result<Self, GenerateError> {
        let carriers = layout.carrier_count;
        let finals = layout.final_count;
        let (carrier_indices, carrier_shells) = layout.carrier_indices();
        let (final_indices, final_shells) = layout.final_indices();
        let context = gpu.context();
        let carrier_triangle_count = carrier_shells.len() as u32;
        let final_triangle_count = final_shells.len() as u32;
        Ok(Self {
            carriers: gpu.scratch(carriers as u64 * 12, "part carriers")?,
            heights: gpu.scratch(carriers as u64 * 4, "part heights")?,
            shells: gpu.upload(&shell_table(&layout))?,
            hinges: gpu.scratch(hinges.max(1) as u64 * HINGE_WORDS as u64 * 4, "part hinges")?,
            status: gpu.scratch(4, "part status")?,
            shell_of: gpu.upload(&layout.shell_of_carrier())?,
            authored_carrier_triangles: gpu.upload(&carrier_indices)?,
            carrier_triangle_shells: gpu.upload(&carrier_shells)?,
            carrier_triangles: gpu
                .scratch(carrier_indices.len() as u64 * 4, "part carrier triangles")?,
            sources: gpu.upload(&layout.sources())?,
            authored_final_indices: gpu.upload(&final_indices)?,
            final_triangle_shells: gpu.upload(&final_shells)?,
            final_indices: gpu.scratch(final_indices.len() as u64 * 4, "part triangles")?,
            walls: gpu.scratch(carriers as u64 * 24, "part walls")?,
            positions: gpu.scratch(finals as u64 * 12, "part positions")?,
            area: VertexNormals::new(context, carriers, carrier_triangle_count)
                .map_err(device_error)?,
            angle: VertexNormals::new(context, carriers, carrier_triangle_count)
                .map_err(device_error)?,
            normals: VertexNormals::new(context, finals, final_triangle_count)
                .map_err(device_error)?,
            carrier_triangle_count,
            final_triangle_count,
            hinge_count: hinges,
            placement: None,
            layout,
        })
    }

    pub(crate) fn layout(&self) -> &PartLayout {
        &self.layout
    }

    /// Thicken every shell and compute the final normals.
    pub(crate) fn record_shells(
        &mut self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
    ) -> Result<(), GenerateError> {
        let carriers = self.layout.carrier_count;
        let kernels = ShellKernels::get(gpu)?;
        for (authored, shells, wound, count) in [
            (
                &self.authored_carrier_triangles,
                &self.carrier_triangle_shells,
                &self.carrier_triangles,
                self.carrier_triangle_count,
            ),
            (
                &self.authored_final_indices,
                &self.final_triangle_shells,
                &self.final_indices,
                self.final_triangle_count,
            ),
        ] {
            let mut winding = PassParameters::new();
            winding.insert("count", count);
            pad(&mut winding);
            winding.insert("authored", authored.clone());
            winding.insert("triangle_shells", shells.clone());
            winding.insert("shells", self.shells.clone());
            winding.insert("wound", wound.clone());
            batch
                .dispatch_items(&kernels.winding, &winding, count)
                .map_err(device_error)?;
        }
        for (weighting, output) in [
            (NormalWeighting::Area, &mut self.area),
            (NormalWeighting::Angle, &mut self.angle),
        ] {
            gpu.normals(weighting)
                .record(
                    batch,
                    &self.carriers,
                    &self.carrier_triangles,
                    carriers,
                    self.carrier_triangle_count,
                    output,
                )
                .map_err(device_error)?;
        }
        let mut walls = PassParameters::new();
        walls.insert("count", carriers);
        pad(&mut walls);
        walls.insert("carriers", self.carriers.clone());
        walls.insert("heights", self.heights.clone());
        walls.insert("shell_of", self.shell_of.clone());
        walls.insert("shells", self.shells.clone());
        walls.insert("area_normals", self.area.normals.clone());
        walls.insert("angle_normals", self.angle.normals.clone());
        walls.insert("walls", self.walls.clone());
        walls.insert("status", self.status.clone());
        batch
            .dispatch_items(&kernels.walls, &walls, carriers)
            .map_err(device_error)?;

        let finals = self.layout.final_count;
        let mut assemble = PassParameters::new();
        assemble.insert("count", finals);
        assemble.insert("carriers", carriers);
        assemble.insert("pad1", 0u32);
        assemble.insert("pad2", 0u32);
        assemble.insert("sources", self.sources.clone());
        assemble.insert("walls", self.walls.clone());
        assemble.insert("positions", self.positions.clone());
        batch
            .dispatch_items(&kernels.assemble, &assemble, finals)
            .map_err(device_error)?;
        if let Some(frame) = &self.placement {
            super::placement::record(
                gpu,
                batch,
                &super::placement::Placement {
                    frame,
                    positions: &self.positions,
                    count: finals,
                    hinges: &self.hinges,
                    hinge_count: self.hinge_count,
                    status: &self.status,
                },
            )?;
        }

        gpu.normals(NormalWeighting::Area)
            .record(
                batch,
                &self.positions,
                &self.final_indices,
                finals,
                self.final_triangle_count,
                &mut self.normals,
            )
            .map_err(device_error)?;
        Ok(())
    }

    /// The final positions on the device, for stages that follow the shell.
    pub(crate) fn positions(&self) -> &Buffer {
        &self.positions
    }

    pub(crate) fn final_count(&self) -> u32 {
        self.layout.final_count
    }

    pub(crate) fn final_indices(&self) -> &Buffer {
        &self.final_indices
    }

    pub(crate) fn final_triangle_count(&self) -> u32 {
        self.final_triangle_count
    }

    /// Read the part back after the batch that built it has been submitted.
    /// Stage everything the finished part is read from.
    pub(crate) fn stage<'a>(&'a self, staging: &mut Staging<'a>) -> PartSlots {
        PartSlots {
            area: staging.stage(&self.area.status),
            angle: staging.stage(&self.angle.status),
            shell: staging.stage(&self.status),
            normals_status: staging.stage(&self.normals.status),
            positions: staging.stage(&self.positions),
            normals: staging.stage(&self.normals.normals),
            hinges: staging.stage(&self.hinges),
            indices: staging.stage(&self.final_indices),
        }
    }

    /// The finished part, from its staged results.
    pub(crate) fn finish(
        &self,
        results: &StagedResults,
        slots: PartSlots,
    ) -> Result<BuiltPart, GenerateError> {
        // Carrier normals are checked: a carrier triangle without
        // area is degenerate, an angle-weighted carrier that cannot be
        // normalised is not a surface, and neither is an extrusion that does
        // not leave its carrier.
        if results.status(slots.area) != 0 {
            return Err(GenerateError::Degenerate);
        }
        if results.status(slots.angle) != 0 && self.layout.uses_angle_normals()
            || results.status(slots.shell) != 0
        {
            return Err(GenerateError::InvalidSurface);
        }
        if results.status(slots.normals_status) != 0 {
            return Err(GenerateError::Degenerate);
        }
        let count = self.layout.final_count as usize;
        let mut positions: Vec<[f32; 3]> = results.get(slots.positions);
        positions.truncate(count);
        let mut normals: Vec<[f32; 3]> = results.get(slots.normals);
        normals.truncate(count);
        let hinges: Vec<[f32; 4]> = results.get(slots.hinges);
        let components = self
            .layout
            .components()
            .into_iter()
            .map(|(mut component, slot)| {
                component.hinge = slot.map(|HingeSlot(slot)| ArmorHinge {
                    origin: std::array::from_fn(|i| hinges[slot as usize * 2][i]),
                    axis: std::array::from_fn(|i| hinges[slot as usize * 2 + 1][i]),
                });
                component
            })
            .collect();
        let mut indices: Vec<u32> = results.get(slots.indices);
        indices.truncate(self.final_triangle_count as usize * 3);
        Ok(BuiltPart {
            positions,
            normals,
            indices,
            faces: self.layout.final_faces(),
            components,
            grids: self.layout.grids(),
        })
    }
}

/// Where a part's results are in a staged readback.
#[derive(Clone, Copy, Debug)]
pub struct PartSlots {
    area: Staged,
    angle: Staged,
    shell: Staged,
    normals_status: Staged,
    positions: Staged,
    normals: Staged,
    hinges: Staged,
    indices: Staged,
}

/// A finished part, back on the host.
#[derive(Clone, Debug)]
pub struct BuiltPart {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    /// The plate face of each triangle.
    pub faces: Vec<PlateFace>,
    pub components: Vec<ArmorComponent>,
    /// The outer face of every plate laid out on a grid.
    pub grids: Vec<SurfaceGrid>,
}

fn shell_table(layout: &PartLayout) -> Vec<f32> {
    let mut table = Vec::with_capacity(layout.shells.len() * SHELL_WORDS as usize);
    for shell in &layout.shells {
        let mut record = [0.0f32; SHELL_WORDS as usize];
        record[0] = f32::from_bits(shell.extrusion.code());
        record[1] = shell.thickness;
        table.extend(record);
    }
    table
}

fn pad(parameters: &mut PassParameters) {
    for name in ["pad0", "pad1", "pad2"] {
        parameters.insert(name, 0u32);
    }
}
