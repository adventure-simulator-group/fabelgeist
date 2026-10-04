//! How a part is laid out on the host, from its design alone: every shell's
//! place in the carrier arena and the final mesh, its thickening, and the
//! components its shells make up.

use super::super::shell_plan::{INNER_BIT, ShellPlan};
use crate::{
    ArmorComponent, ArmorComponentRole, BoundaryNormals, GenerateError, PlateFace, SurfaceGrid,
};

/// How a shell's inner wall leaves its carrier. The device holds the
/// geometric half -- origins and axes -- because some depend on the fit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Extrusion {
    Normal,
    AngleWeightedNormal,
    CappedAxis,
    Along,
    Radial,
}

impl Extrusion {
    pub(crate) fn code(self) -> u32 {
        match self {
            Self::Normal => 0,
            Self::AngleWeightedNormal => 1,
            Self::CappedAxis => 2,
            Self::Along => 3,
            Self::Radial => 4,
        }
    }
}

/// One shell as the host describes it.
#[derive(Clone, Debug)]
pub(crate) struct ShellSpec {
    pub carrier_count: u32,
    /// Carrier triangles, indexed within the shell.
    pub carrier_indices: Vec<u32>,
    pub boundary_normals: BoundaryNormals,
    pub thickness: f32,
    pub extrusion: Extrusion,
    /// The rows and columns the first carriers make up, row by row, when
    /// the shell was laid out on a grid.
    pub grid: Option<GridShape>,
}

/// A carrier grid's size: rows along the plate, columns across it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GridShape {
    pub rows: u32,
    pub columns: u32,
    pub cyclic: bool,
}

#[derive(Clone, Debug)]
pub(super) struct LaidShell {
    pub(super) first_carrier: u32,
    pub(super) carrier_count: u32,
    pub(super) first_final: u32,
    pub(super) plan: ShellPlan,
    pub(super) thickness: f32,
    pub(super) extrusion: Extrusion,
    pub(super) grid: Option<GridShape>,
}

/// Floats per shell in the device's shell table: kind, thickness, whether
/// the shell was placed through a reflection, then the extrusion origin and
/// axis (or direction), each padded to four. A reflected shell's triangles
/// are rewound, which only the device can decide: a fit may reflect a frame.
pub(crate) const SHELL_WORDS: u32 = 12;
pub(crate) const SHELL_MIRRORED: u32 = 2;

/// Where every shell of a part lives, and the final topology.
#[derive(Clone, Debug, Default)]
pub(crate) struct PartLayout {
    pub(super) shells: Vec<LaidShell>,
    pub(super) carrier_count: u32,
    pub(super) final_count: u32,
    /// Roles cover consecutive runs of shells.
    components: Vec<ComponentLayout>,
}

#[derive(Clone, Debug)]
struct ComponentLayout {
    role: ArmorComponentRole,
    shells: std::ops::Range<usize>,
    hinge: Option<HingeSlot>,
    mount: Option<crate::PlateMount>,
}

/// A hinge is placed by the fit, so the device transforms it; the slot is
/// where in the hinge buffer its world origin and axis are written.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HingeSlot(pub u32);

impl PartLayout {
    /// Add a shell; returns its index.
    pub(crate) fn push(&mut self, spec: ShellSpec) -> Result<usize, GenerateError> {
        let plan = ShellPlan::new(
            spec.carrier_count,
            &spec.carrier_indices,
            spec.boundary_normals,
        )?;
        let final_count = plan.sources.len() as u32;
        self.shells.push(LaidShell {
            first_carrier: self.carrier_count,
            carrier_count: spec.carrier_count,
            first_final: self.final_count,
            plan,
            thickness: spec.thickness,
            extrusion: spec.extrusion,
            grid: spec.grid,
        });
        self.carrier_count += spec.carrier_count;
        self.final_count += final_count;
        Ok(self.shells.len() - 1)
    }

    /// Tag the shells added since the last component with a role.
    pub(crate) fn component(&mut self, role: ArmorComponentRole, hinge: Option<HingeSlot>) {
        let start = self
            .components
            .last()
            .map_or(0, |component| component.shells.end);
        self.components.push(ComponentLayout {
            role,
            shells: start..self.shells.len(),
            hinge,
            mount: None,
        });
    }

    pub(crate) fn mount(&mut self, mount: crate::PlateMount) {
        self.components
            .last_mut()
            .expect("mount follows its component")
            .mount = Some(mount);
    }

    pub(crate) fn shell_count(&self) -> usize {
        self.shells.len()
    }

    pub(crate) fn first_carrier(&self, shell: usize) -> u32 {
        self.shells[shell].first_carrier
    }

    pub(crate) fn carrier_count(&self, shell: usize) -> u32 {
        self.shells[shell].carrier_count
    }

    pub(crate) fn total_carriers(&self) -> u32 {
        self.carrier_count
    }

    /// A shell's carrier triangles as authored, indexed within the shell.
    pub(crate) fn carrier_triangles(&self, shell: usize) -> &[u32] {
        let plan = &self.shells[shell].plan;
        &plan.indices[..plan.carrier_triangles * 3]
    }

    pub(super) fn uses_angle_normals(&self) -> bool {
        self.shells
            .iter()
            .any(|shell| shell.extrusion == Extrusion::AngleWeightedNormal)
    }

    /// The final triangles, indexed into the whole part, and the shell of
    /// each, as wound before any reflection.
    pub(super) fn final_indices(&self) -> (Vec<u32>, Vec<u32>) {
        let mut indices = Vec::new();
        let mut shells = Vec::new();
        for (index, shell) in self.shells.iter().enumerate() {
            indices.extend(shell.plan.indices.iter().map(|i| i + shell.first_final));
            shells.extend(std::iter::repeat_n(
                index as u32,
                shell.plan.indices.len() / 3,
            ));
        }
        (indices, shells)
    }

    /// The plate face of every final triangle, in [`Self::final_indices`] order.
    pub(super) fn final_faces(&self) -> Vec<PlateFace> {
        self.shells
            .iter()
            .flat_map(|shell| shell.plan.faces())
            .collect()
    }

    /// The carriers' own triangles, indexed into the arena, and their shells.
    pub(super) fn carrier_indices(&self) -> (Vec<u32>, Vec<u32>) {
        let mut indices = Vec::new();
        let mut shells = Vec::new();
        for (index, shell) in self.shells.iter().enumerate() {
            let carrier = &shell.plan.indices[..shell.plan.carrier_triangles * 3];
            indices.extend(carrier.iter().map(|i| i + shell.first_carrier));
            shells.extend(std::iter::repeat_n(
                index as u32,
                shell.plan.carrier_triangles,
            ));
        }
        (indices, shells)
    }

    pub(super) fn sources(&self) -> Vec<u32> {
        self.shells
            .iter()
            .flat_map(|shell| {
                shell.plan.sources.iter().map(|source| {
                    let packed = source.packed();
                    let inner = packed & INNER_BIT;
                    ((packed & !INNER_BIT) + shell.first_carrier) | inner
                })
            })
            .collect()
    }

    /// The outer face of every shell laid out on a grid. A shell's outer
    /// wall is its carriers, so its grid is its first final vertices.
    pub(super) fn grids(&self) -> Vec<SurfaceGrid> {
        self.shells
            .iter()
            .filter_map(|shell| {
                let grid = shell.grid?;
                let mut surface = SurfaceGrid::regular(
                    grid.rows,
                    grid.columns,
                    grid.cyclic,
                    (0..grid.rows * grid.columns)
                        .map(|i| shell.first_final + i)
                        .collect(),
                );
                surface.samples = shell
                    .plan
                    .sources
                    .iter()
                    .enumerate()
                    .filter(|(_, source)| source.packed() & INNER_BIT == 0)
                    .map(|(i, source)| crate::SurfaceSample {
                        vertex: shell.first_final + i as u32,
                        column: crate::SurfaceColumn::at(source.packed() % grid.columns),
                        carrier_column: crate::SurfaceColumn::at(source.packed() % grid.columns),
                        edge_distances: [crate::SurfaceEdgeDistance::Rail; 2],
                    })
                    .collect();
                Some(surface)
            })
            .collect()
    }

    pub(super) fn shell_of_carrier(&self) -> Vec<u32> {
        self.shells
            .iter()
            .enumerate()
            .flat_map(|(i, shell)| std::iter::repeat_n(i as u32, shell.carrier_count as usize))
            .collect()
    }

    pub(super) fn components(&self) -> Vec<(ArmorComponent, Option<HingeSlot>)> {
        let mut index_starts = Vec::with_capacity(self.shells.len() + 1);
        let mut at = 0;
        for shell in &self.shells {
            index_starts.push(at);
            at += shell.plan.indices.len();
        }
        index_starts.push(at);
        self.components
            .iter()
            .map(|component| {
                let shells = &component.shells;
                let first = &self.shells[shells.start];
                let last = &self.shells[shells.end - 1];
                (
                    ArmorComponent {
                        role: component.role,
                        vertices: first.first_final as usize
                            ..(last.first_final as usize + last.plan.sources.len()),
                        indices: index_starts[shells.start]..index_starts[shells.end],
                        hinge: None,
                        mount: component.mount,
                        material: None,
                    },
                    component.hinge,
                )
            })
            .collect()
    }
}
