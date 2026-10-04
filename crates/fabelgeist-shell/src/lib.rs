//! Shared thin-shell mechanics on XPBD: stretch, bending, attachments and contact.
//! Cloth adds garment construction; metal adds sheet material parameters.
mod bending;
mod ccd;
mod material;
mod mesh;
pub mod outer_layer;
pub mod selfcollision;
mod shell;
pub mod surface_contact;
pub mod topology;
pub mod wgsl;
pub use bending::{
    BendGeometryError, BendPoints, BendRecord, BendRecordValidity, BendRestMeasure, BendWeights,
};
pub use fabelgeist_physics::Collisions;
pub use fabelgeist_xpbd::{
    ArealDensityValidity, ConstraintAttachment, ConstraintAttachmentError, MassValidity,
    ParticleArealDensity, ParticleCapacity, ParticleCount, ParticleError, ParticleInputCount,
    ParticleInputIndex, ParticleInverseMass, ParticleMass, ParticleMassCount, ParticleMobility,
    Solver, SolverSettings,
};
pub use material::{ShellMaterial, ShellMaterialError, ShellMaterialParameter};
pub use mesh::{ShellConstraintFamily, ShellMesh, ShellMeshError, ShellParticleReference};
pub use selfcollision::{
    SelfCollision, SelfCollisionBuffer, SelfCollisionBuildError, SelfCollisionKernel,
    SelfCollisionRecordError,
};
pub use shell::{
    HostContactSchedule, Shell, ShellBuildError, ShellHookError, ShellPositionValidity,
    ShellProjectionError, ShellProjectionStage, ShellStepError,
};
pub use topology::{BendQuad, Topology};

#[cfg(test)]
mod native_vector_fixture;
