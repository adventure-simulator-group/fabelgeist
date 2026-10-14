//! Shared thin-shell mechanics on XPBD: stretch, bending, attachments and contact.
//! Cloth adds garment construction; metal adds sheet material parameters.
mod ccd;
mod mesh;
pub mod outer_layer;
pub mod selfcollision;
mod shell;
pub mod surface_contact;
pub mod topology;
pub mod wgsl;
pub use fabelgeist_physics::Collisions;
pub use fabelgeist_xpbd::{Solver, SolverSettings, SubstepCount};
pub use mesh::ShellMesh;
pub use selfcollision::SelfCollision;
pub use shell::{HostContactSchedule, Shell};
pub use topology::{BendQuad, Topology};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellMaterial {
    pub stretch_compliance: f32,
    pub bend_compliance: f32,
    /// Compliance of zero-length attachment constraints (including cloth seams).
    pub seam_compliance: f32,
    pub thickness: f32,
    /// Suggested world-contact coefficient; configure it on physics colliders.
    pub friction: f32,
    pub damping: f32,
}
impl ShellMaterial {
    pub fn particle_radius(&self) -> f32 {
        self.thickness * 0.5
    }
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            [
                self.stretch_compliance,
                self.bend_compliance,
                self.seam_compliance,
                self.thickness,
                self.friction,
                self.damping
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.0),
            "invalid shell material"
        );
        anyhow::ensure!(self.thickness > 0.0, "shell thickness must be positive");
        Ok(())
    }
}

pub use fabelgeist_xpbd::{
    ArealDensityValidity, MassValidity, ParticleArealDensity, ParticleInverseMass, ParticleMass,
    ParticleMobility,
};

#[cfg(test)]
mod native_vector_fixture;
