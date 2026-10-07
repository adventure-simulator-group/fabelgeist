//! A part's shells as the host describes them, and their recording.

use fabelgeist_compute::KernelBatch;
use fabelgeist_compute::KernelBatchLabel;
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_gpu::prelude::BufferUpload;

use super::chart::{self, ChartInputs, ChartKernel, ChartSlots, PlateChart};
use super::coord::{self, CoordKernel, CoordShell};
use super::part::{HingeSlot, PartBuild, PartLayout};
use super::{ArmorGpu, BuiltPart, DevicePart};
use crate::{ArmorComponentRole, GenerateError, PartFrame};

enum RecipeShell {
    Chart(PlateChart, ChartSlots, ChartKernel),
    Coord(CoordShell, CoordKernel),
}

/// Every shell of a part, each with the kernel of its shape.
#[derive(Default)]
pub(crate) struct PartRecipe {
    layout: PartLayout,
    shells: Vec<(usize, RecipeShell)>,
    hinges: u32,
}

impl PartRecipe {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push_chart(
        &mut self,
        chart: PlateChart,
        kernel: ChartKernel,
    ) -> Result<(), GenerateError> {
        let (spec, slots) = chart.layout();
        let shell = self.layout.push(spec)?;
        self.shells
            .push((shell, RecipeShell::Chart(chart, slots, kernel)));
        Ok(())
    }

    pub(crate) fn push_coord(
        &mut self,
        shell: CoordShell,
        kernel: CoordKernel,
    ) -> Result<(), GenerateError> {
        let index = self.layout.push(shell.spec())?;
        self.shells.push((index, RecipeShell::Coord(shell, kernel)));
        Ok(())
    }

    /// Preserve an explicitly authored coordinate grid in the exported shell.
    pub(crate) fn push_grid_coord(
        &mut self,
        shell: CoordShell,
        kernel: CoordKernel,
        grid: super::part::GridShape,
    ) -> Result<(), GenerateError> {
        if grid.rows < 2
            || grid.columns < 2
            || grid.rows.checked_mul(grid.columns) != u32::try_from(shell.coords.len()).ok()
        {
            return Err(GenerateError::InvalidSurface);
        }
        let mut spec = shell.spec();
        spec.grid = Some(grid);
        let index = self.layout.push(spec)?;
        self.shells.push((index, RecipeShell::Coord(shell, kernel)));
        Ok(())
    }

    /// A slot for a hinge that a later coordinate shell will place.
    pub(crate) fn hinge(&mut self) -> HingeSlot {
        self.hinges += 1;
        HingeSlot(self.hinges - 1)
    }

    /// Tag the shells pushed since the last component with a role.
    pub(crate) fn component(&mut self, role: ArmorComponentRole, hinge: Option<HingeSlot>) {
        self.layout.component(role, hinge);
    }

    /// Tag and connect the newly authored plate in proximal-to-distal order.
    pub(crate) fn mounted_component(&mut self, role: ArmorComponentRole, mount: crate::PlateMount) {
        self.component(role, None);
        self.layout.mount(mount);
    }

    /// Allocate the part and record every shell into its carriers.
    pub(crate) fn record(
        self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        design: &[f32],
        frames: &[&Buffer],
    ) -> Result<DevicePart, GenerateError> {
        let build = PartBuild::new(gpu, self.layout, self.hinges)?;
        let design = gpu.upload(BufferUpload::from_elements(design))?;
        let inputs = ChartInputs {
            design: &design,
            frames,
        };
        for (shell, recipe) in &self.shells {
            match recipe {
                RecipeShell::Chart(chart, slots, kernel) => {
                    chart::record(gpu, batch, &build, *shell, chart, slots, kernel, &inputs)?;
                }
                RecipeShell::Coord(coord_shell, kernel) => {
                    let coords = gpu.upload(BufferUpload::from_elements(&coord_shell.coords))?;
                    coord::record(
                        batch,
                        &build,
                        *shell,
                        coord_shell,
                        &coords,
                        kernel,
                        &design,
                        frames,
                    )?;
                }
            }
        }
        Ok(DevicePart(build))
    }

    /// Evaluate and thicken the part in a host frame, and read it back.
    pub(crate) fn build(
        self,
        gpu: &ArmorGpu,
        design: &[f32],
        frame: &PartFrame,
    ) -> Result<BuiltPart, GenerateError> {
        frame.validate()?;
        let frames = gpu.upload(BufferUpload::from_elements(&frame_words(frame)))?;
        let mut batch = gpu.batch(KernelBatchLabel::from("armor part"));
        let mut part = self.record(gpu, &mut batch, design, &[&frames])?;
        part.record_shells(gpu, &mut batch)?;
        batch.submit();
        part.read(gpu)
    }
}

/// A part's own frame -- identity axes at the origin with `half_extents` --
/// padded to where a fit buffer's wearer profile starts.
pub(crate) fn own_frame_words(half_extents: [f32; 3]) -> Vec<f32> {
    let own = PartFrame {
        origin: [0.0; 3],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents,
    };
    let mut words = frame_words(&own).to_vec();
    words.resize(super::close_helmet::FIT_PROFILE_WORD, 0.0);
    words
}

/// A frame as the device stores it: origin, axes, half extents.
pub fn frame_words(frame: &PartFrame) -> [f32; 15] {
    let mut words = [0.0; 15];
    words[..3].copy_from_slice(&frame.origin);
    for (axis, values) in frame.axes.iter().enumerate() {
        words[3 + axis * 3..6 + axis * 3].copy_from_slice(values);
    }
    words[12..].copy_from_slice(&frame.half_extents);
    words
}
