use super::*;

impl DrapedGarment {
    /// Verify the emitted triangles, including plate edges crossing cloth interiors.
    pub fn validate_armor(&self, armor: &fabelgeist_armor::Armor) -> Result<()> {
        let points: Vec<_> = self.positions.iter().copied().map(vector).collect();
        let cloth = fabelgeist_bvh::TriangleBvh::new(points.clone(), self.faces.clone());
        let layer = outer_layer(armor, self.fabric.fabric())?;
        let residual = layer.surface_residual(&points, &self.faces);
        anyhow::ensure!(
            residual <= fabelgeist_cloth::outer_layer::CLEARANCE_TOLERANCE,
            "garment does not fit beneath armor: {:.1} mm unresolved clearance",
            residual * 1000.0
        );
        for part in fabelgeist_armor::build(armor).map_err(anyhow::Error::msg)? {
            let plate = fabelgeist_bvh::TriangleBvh::new(
                part.mesh.positions.iter().copied().map(vector).collect(),
                part.mesh.faces,
            );
            anyhow::ensure!(
                !edges_cross(&cloth, &plate) && !edges_cross(&plate, &cloth),
                "garment intersects {}; adjust armor clearance or garment fit",
                part.name
            );
        }
        Ok(())
    }
}

pub(super) fn edges_cross(
    source: &fabelgeist_bvh::TriangleBvh,
    target: &fabelgeist_bvh::TriangleBvh,
) -> bool {
    // Exclude endpoint contact and coplanar contact; retain strict edge crossings.
    const ENDPOINT_TOLERANCE: f32 = 1e-6;
    source.triangles.iter().any(|face| {
        (0..3).any(|i| {
            let a = source.positions[face[i] as usize];
            let b = source.positions[face[(i + 1) % 3] as usize];
            let delta = b - a;
            let length = delta.length();
            if length <= ENDPOINT_TOLERANCE * 2.0 {
                return false;
            }
            let direction = delta / length;
            target
                .raycast(
                    &fabelgeist_bvh::Ray::new(a + direction * ENDPOINT_TOLERANCE, direction),
                    length - ENDPOINT_TOLERANCE * 2.0,
                )
                .is_some()
        })
    })
}

pub(super) fn outer_layer(
    armor: &fabelgeist_armor::Armor,
    fabric: Fabric,
) -> Result<fabelgeist_cloth::outer_layer::OuterLayer> {
    let mut positions = Vec::new();
    let mut faces = Vec::new();
    for part in fabelgeist_armor::build(armor).map_err(anyhow::Error::msg)? {
        let offset = positions.len() as u32;
        // This generator authors a front plate in +Z. Its rear-facing triangles
        // bound the space between the wearer and the metal, including the fauld.
        for face in part.mesh.faces {
            let [a, b, c] = face.map(|i| vector(part.mesh.positions[i as usize]));
            let normal = (b - a).cross(c - a);
            if normal.z < -normal.length() * 1e-5 {
                faces.push(face.map(|i| i + offset));
            }
        }
        positions.extend(part.mesh.positions.into_iter().map(vector));
    }
    anyhow::ensure!(!faces.is_empty(), "armor has no inward surface");
    Ok(fabelgeist_cloth::outer_layer::OuterLayer::new(
        positions,
        faces,
        // The simulated sheet is the mid-surface: reserve half its thickness
        // against each obstacle, just as the body collider's particle radius does.
        fabric.particle_radius(),
        Vec3::new(0.0, 0.0, -1.0),
    ))
}
