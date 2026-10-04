//! A plate part built on the device, from carrier to closed shell.
//!
//! The host lays the part out -- how many carrier vertices each shell has,
//! how they are connected, how each shell is thickened -- from the design
//! alone. The device does everything that depends on where the points are:
//! chart kernels write the carriers into one arena, fitting kernels move
//! them, and the shell stage below thickens every shell at once and computes
//! the final normals. Nothing is read back until [`PartBuild::read`].

use fabelgeist_compute::{KernelBatch, NormalWeighting, VertexNormals};
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};
use fabelgeist_gpu::prelude::{ReadbackSlot, ReadbackStatusWord};

mod layout;
mod shell_kernels;

use super::staging::{StagedResults, Staging};
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
            carriers: gpu.scratch((carriers as u64 * 12).into(), ("part carriers").into())?,
            heights: gpu.scratch((carriers as u64 * 4).into(), ("part heights").into())?,
            shells: gpu.upload(BufferUpload::from_elements(&shell_table(&layout)))?,
            hinges: gpu.scratch(
                (hinges.max(1) as u64 * HINGE_WORDS as u64 * 4).into(),
                ("part hinges").into(),
            )?,
            status: gpu.scratch((4u64).into(), ("part status").into())?,
            shell_of: gpu.upload(BufferUpload::from_elements(&layout.shell_of_carrier()))?,
            authored_carrier_triangles: gpu
                .upload(BufferUpload::from_elements(&carrier_indices))?,
            carrier_triangle_shells: gpu.upload(BufferUpload::from_elements(&carrier_shells))?,
            carrier_triangles: gpu.scratch(
                (carrier_indices.len() as u64 * 4).into(),
                ("part carrier triangles").into(),
            )?,
            sources: gpu.upload(BufferUpload::from_elements(&layout.sources()))?,
            authored_final_indices: gpu.upload(BufferUpload::from_elements(&final_indices))?,
            final_triangle_shells: gpu.upload(BufferUpload::from_elements(&final_shells))?,
            final_indices: gpu.scratch(
                (final_indices.len() as u64 * 4).into(),
                ("part triangles").into(),
            )?,
            walls: gpu.scratch((carriers as u64 * 24).into(), ("part walls").into())?,
            positions: gpu.scratch((finals as u64 * 12).into(), ("part positions").into())?,
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
            winding.insert("count".into(), (count).into());
            pad(&mut winding);
            winding.insert("authored".into(), (authored.clone()).into());
            winding.insert("triangle_shells".into(), (shells.clone()).into());
            winding.insert("shells".into(), (self.shells.clone()).into());
            winding.insert("wound".into(), (wound.clone()).into());
            batch
                .dispatch_items(&kernels.winding, &winding, (count).into())
                .map_err(crate::GenerateError::from)?;
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
        walls.insert("count".into(), (carriers).into());
        pad(&mut walls);
        walls.insert("carriers".into(), (self.carriers.clone()).into());
        walls.insert("heights".into(), (self.heights.clone()).into());
        walls.insert("shell_of".into(), (self.shell_of.clone()).into());
        walls.insert("shells".into(), (self.shells.clone()).into());
        walls.insert("area_normals".into(), (self.area.normals.clone()).into());
        walls.insert("angle_normals".into(), (self.angle.normals.clone()).into());
        walls.insert("walls".into(), (self.walls.clone()).into());
        walls.insert("status".into(), (self.status.clone()).into());
        batch
            .dispatch_items(&kernels.walls, &walls, (carriers).into())
            .map_err(crate::GenerateError::from)?;

        let finals = self.layout.final_count;
        let mut assemble = PassParameters::new();
        assemble.insert("count".into(), (finals).into());
        assemble.insert("carriers".into(), (carriers).into());
        assemble.insert("pad1".into(), (0u32).into());
        assemble.insert("pad2".into(), (0u32).into());
        assemble.insert("sources".into(), (self.sources.clone()).into());
        assemble.insert("walls".into(), (self.walls.clone()).into());
        assemble.insert("positions".into(), (self.positions.clone()).into());
        batch
            .dispatch_items(&kernels.assemble, &assemble, (finals).into())
            .map_err(crate::GenerateError::from)?;
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
        if results.status(slots.area)? != ReadbackStatusWord::CLEAR {
            return Err(GenerateError::Degenerate);
        }
        if results.status(slots.angle)? != ReadbackStatusWord::CLEAR
            && self.layout.uses_angle_normals()
            || results.status(slots.shell)? != ReadbackStatusWord::CLEAR
        {
            return Err(GenerateError::InvalidSurface);
        }
        if results.status(slots.normals_status)? != ReadbackStatusWord::CLEAR {
            return Err(GenerateError::Degenerate);
        }
        let count = self.layout.final_count as usize;
        let mut positions: Vec<[f32; 3]> = results.get(slots.positions)?;
        positions.truncate(count);
        let mut normals: Vec<[f32; 3]> = results.get(slots.normals)?;
        normals.truncate(count);
        let hinges: Vec<[f32; 4]> = results.get(slots.hinges)?;
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
        let mut indices: Vec<u32> = results.get(slots.indices)?;
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
    area: ReadbackSlot,
    angle: ReadbackSlot,
    shell: ReadbackSlot,
    normals_status: ReadbackSlot,
    positions: ReadbackSlot,
    normals: ReadbackSlot,
    hinges: ReadbackSlot,
    indices: ReadbackSlot,
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

impl BuiltPart {
    /// Add `other`'s plates after this part's, as one part. Every plate
    /// keeps its own vertices, faces, components and grids.
    pub fn append(&mut self, other: BuiltPart) {
        let vertices = self.positions.len();
        let indices = self.indices.len();
        let offset = vertices as u32;
        self.positions.extend(other.positions);
        self.normals.extend(other.normals);
        self.indices
            .extend(other.indices.iter().map(|i| i + offset));
        self.faces.extend(other.faces);
        self.components
            .extend(other.components.into_iter().map(|mut component| {
                component.vertices =
                    component.vertices.start + vertices..component.vertices.end + vertices;
                component.indices =
                    component.indices.start + indices..component.indices.end + indices;
                component
            }));
        self.grids.extend(other.grids.into_iter().map(|mut grid| {
            for vertex in &mut grid.vertices {
                *vertex += offset;
            }
            grid
        }));
    }
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
        parameters.insert(name.into(), (0u32).into());
    }
}
