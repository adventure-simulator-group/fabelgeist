//! Fabric-specific configuration of the shared shell solver.
use crate::{Fabric, GarmentMesh};
use anyhow::Result;
use fabelgeist_compute::KernelCache;
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_shell::{Shell, ShellMesh};

use fabelgeist_compute::KernelBatch;
#[cfg(test)]
use fabelgeist_math::Vec3;

pub struct Cloth {
    pub shell: Shell,
    pub fabric: Fabric,
}
impl std::ops::Deref for Cloth {
    type Target = Shell;
    fn deref(&self) -> &Shell {
        &self.shell
    }
}
impl std::ops::DerefMut for Cloth {
    fn deref_mut(&mut self) -> &mut Shell {
        &mut self.shell
    }
}
impl Cloth {
    pub fn new(
        context: &WgpuContext,
        cache: &KernelCache,
        mesh: &GarmentMesh,
        fabric: Fabric,
    ) -> Result<Self> {
        let shell_mesh = ShellMesh {
            positions: mesh.positions.clone(),
            triangles: mesh.triangles.clone(),
            edges: mesh.edges.clone(),
            rest_lengths: mesh.rest_lengths.clone(),
            bends: mesh.bends.clone(),
            bend_weights: mesh.bend_weights.clone(),
            seams: mesh.seams.clone(),
            masses: mesh.masses.clone(),
        };
        Ok(Self {
            shell: Shell::new(context, cache, &shell_mesh, fabric.into())?,
            fabric,
        })
    }
    pub fn apply_fabric(&mut self) {
        self.shell.material = self.fabric.into();
        self.shell.apply_material();
    }
    pub fn settings(&self) -> fabelgeist_shell::SolverSettings {
        fabelgeist_shell::SolverSettings {
            damping: self.fabric.damping,
            ..Default::default()
        }
    }
    pub fn record_step(
        &mut self,
        batch: &mut KernelBatch,
        solver: &fabelgeist_shell::Solver,
        collisions: &mut fabelgeist_shell::Collisions,
        delta: f32,
    ) -> Result<()> {
        self.shell.material.thickness = self.fabric.thickness;
        self.shell.record_step(batch, solver, collisions, delta)
    }
    pub fn step(
        &mut self,
        context: &WgpuContext,
        solver: &fabelgeist_shell::Solver,
        collisions: &mut fabelgeist_shell::Collisions,
        delta: f32,
    ) -> Result<()> {
        self.shell.material.thickness = self.fabric.thickness;
        self.shell.step(context, solver, collisions, delta)
    }
    pub async fn step_interleaved(
        &mut self,
        context: &WgpuContext,
        solver: &fabelgeist_shell::Solver,
        collisions: &mut fabelgeist_shell::Collisions,
        delta: f32,
    ) -> Result<()> {
        self.shell.material.thickness = self.fabric.thickness;
        self.shell
            .step_interleaved(context, solver, collisions, delta)
            .await
    }
}
#[cfg(test)]
mod tests;
