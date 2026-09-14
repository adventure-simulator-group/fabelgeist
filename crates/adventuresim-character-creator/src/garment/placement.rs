use super::*;

pub(super) async fn prepare(
    input: &DrapeInput,
    build: &fabelgeist_garment_fit::GarmentBuild,
    settings: &FitSettings,
    fabric: Fabric,
) -> Result<(Fit, fabelgeist_bvh::TriangleBvh)> {
    let context = fabelgeist_gpu::prelude::WgpuContext::new().await?;
    let mut fit = Fit::new(context, build, fabric, settings)?;
    // Pattern placement is measured from the floor and the pelvis centre.
    // Keep the actual body size: translate the garment, never scale the body.
    let floor = input
        .positions
        .iter()
        .map(|p| p[1])
        .fold(f32::INFINITY, f32::min);
    let hip = |name: &str| input.joints[input.names.iter().position(|n| n == name).unwrap()];
    let (left, right) = (hip("l_upleg"), hip("r_upleg"));
    let mut pose = fabelgeist_garment_fit::GarmentPose::for_mesh(&build.mesh);
    pose.garment.offset = Vec3::new(
        (left[0] + right[0]) * 0.5,
        floor,
        (left[2] + right[2]) * 0.5,
    );
    fit.set_pose(pose)?;
    let vertices = input
        .positions
        .iter()
        .copied()
        .map(vector)
        .collect::<Vec<_>>();
    let collision = collision_surface(input);
    let mut placed = fit.placed_positions();
    clear_panels(
        &mut placed,
        &build.mesh,
        &collision,
        settings.body_offset_cm * 0.01 + fabric.particle_radius(),
    );
    fit.cloth.particles.write_positions(&fit.context, &placed)?;
    fit.set_body(&vertices, &input.faces, settings, &fabric)
        .await?;
    if !input.obstacles.is_empty() {
        fit.set_collision_mesh(
            &collision.positions,
            &collision.triangles,
            settings,
            &fabric,
        )?;
    }
    Ok((fit, collision))
}

/// The wearer and every inner garment, as one surface the cloth must clear.
pub(super) fn collision_surface(input: &DrapeInput) -> fabelgeist_bvh::TriangleBvh {
    let mut vertices: Vec<_> = input.positions.iter().copied().map(vector).collect();
    let mut faces = input.faces.clone();
    for garment in &input.obstacles {
        let offset = vertices.len() as u32;
        vertices.extend(garment.positions.iter().copied().map(vector));
        faces.extend(garment.faces.iter().map(|face| face.map(|i| i + offset)));
    }
    fabelgeist_bvh::TriangleBvh::new(vertices, faces)
}
