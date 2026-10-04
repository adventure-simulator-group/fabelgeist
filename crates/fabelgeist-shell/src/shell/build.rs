//! Construct admitted shell resources in their established allocation order.
use super::{HostContactSchedule, Shell, ShellBuildError};
use crate::{SelfCollision, ShellConstraintFamily, ShellMaterial, ShellMesh};
use fabelgeist_compute::kernel::{KernelCache, KernelCacheError};
use fabelgeist_gpu::prelude::WgpuContext;
use fabelgeist_xpbd::{
    ConstraintAttachment, ConstraintBuildError, ConstraintEdges, ConstraintIncidence,
    ConstraintSet, ParticleInputCount, Particles,
};

impl Shell {
    pub fn new(
        context: &WgpuContext,
        cache: &KernelCache,
        mesh: &ShellMesh,
        material: ShellMaterial,
    ) -> Result<Self, ShellBuildError> {
        mesh.validate().map_err(ShellBuildError::Mesh)?;
        material.validate().map_err(ShellBuildError::Material)?;
        let inverse_masses = mesh.inverse_masses();
        let particles = Particles::from_positions(context, &mesh.positions, &inverse_masses)
            .map_err(ShellBuildError::Particles)?;
        let stretch = ConstraintSet::distance(
            context,
            cache,
            "stretch".into(),
            &ConstraintEdges::from(mesh.edges.as_slice()),
            &mesh.rest_lengths,
            material.stretch_compliance.into(),
        )
        .map_err(|source: ConstraintBuildError| -> ShellBuildError {
            ShellBuildError::constraint(ShellConstraintFamily::Stretch, source)
        })?;
        // Seams close their initial gap by pulling toward the same zero rest.
        let seam_rest = vec![0.0f32; mesh.seams.len()];
        let seams = ConstraintSet::distance(
            context,
            cache,
            "seams".into(),
            &ConstraintEdges::from(mesh.seams.as_slice()),
            &seam_rest,
            material.seam_compliance.into(),
        )
        .map_err(|source: ConstraintBuildError| -> ShellBuildError {
            ShellBuildError::constraint(ShellConstraintFamily::Seams, source)
        })?;
        let bending = prepare_bending(context, cache, mesh, material)?;
        let self_collision = SelfCollision::new(
            context,
            cache,
            ParticleInputCount::from(mesh.positions.len()).gpu_count(),
            &mesh.adjacency(),
            material.particle_radius(),
        )
        .map_err(
            |source: crate::SelfCollisionBuildError| -> ShellBuildError {
                ShellBuildError::SelfCollision(Box::new(source))
            },
        )?;
        Ok(Self {
            outer_layer: None,
            host_contacts: HostContactSchedule::default(),
            surface_contacts: crate::surface_contact::SurfaceContacts::new(
                mesh.positions.len(),
                mesh.triangles.clone(),
            )
            .with_seams(&mesh.seams),
            particles,
            stretch,
            bending,
            seams,
            self_collision,
            material,
            triangles: mesh.triangles.clone(),
            initial_positions: mesh.positions.clone(),
            inverse_masses,
        })
    }
}

fn prepare_bending(
    context: &WgpuContext,
    cache: &KernelCache,
    mesh: &ShellMesh,
    material: ShellMaterial,
) -> Result<ConstraintSet, ShellBuildError> {
    let bend_records: Vec<[u32; 4]> = mesh.bends.iter().map(crate::BendQuad::particles).collect();
    let bend_incidence = ConstraintIncidence::from_native_records(&bend_records)
        .map_err(ShellBuildError::BendLayout)?;
    let bend_kernel = cache
        .get(
            context,
            &fabelgeist_xpbd::wgsl::constraint_kernel(crate::wgsl::BEND),
        )
        .map_err(|source: KernelCacheError| -> ShellBuildError {
            ShellBuildError::BendKernel(Box::new(source))
        })?;
    let mut bending = ConstraintSet::new(
        context,
        "bending".into(),
        bend_kernel,
        cache,
        &bend_incidence,
        material.bend_compliance.into(),
    )
    .map_err(|source: ConstraintBuildError| -> ShellBuildError {
        ShellBuildError::constraint(ShellConstraintFamily::Bending, source)
    })?;
    // Color permutation must retain the corresponding hinge's complete record.
    let ordered_weights = bending.reorder(&mesh.bend_weights);
    bending
        .attach(
            context,
            ConstraintAttachment::from_records("weights".into(), &ordered_weights),
        )
        .map_err(ShellBuildError::BendAttachment)?;
    Ok(bending)
}
