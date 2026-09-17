//! Animated cloth integration and coupled outfit contacts.
use super::*;
type Motion = (Vec<Vec3>, Vec<Vec3>, Vec<[u32; 3]>);
type ContactSurface = (Vec<Vec3>, Vec<[u32; 3]>, fabelgeist_bvh::TriangleBvh);

struct OutfitContacts {
    masses: Vec<f32>,
    contacts: Option<fabelgeist_cloth::surface_contact::SurfaceContacts>,
}
impl OutfitContacts {
    fn new(
        cloth: &mut Query<(&Mesh3d, &mut ClothSkin)>,
        body: &Option<Motion>,
        settings: &SimulationSettings,
    ) -> Self {
        let mut faces = Vec::new();
        let mut cloth_count = 0;
        for (_, mut skin) in cloth {
            if !skin.simulating {
                skin.previous = skin.current.clone();
                skin.simulating = true;
            }
            faces.extend(skin.faces.iter().map(|f| f.map(|v| v + cloth_count as u32)));
            cloth_count += skin.current.len();
        }
        let mut masses = vec![1.0; cloth_count];
        if let Some((_, end, body_faces)) = &body {
            faces.extend(body_faces.iter().map(|f| f.map(|v| v + cloth_count as u32)));
            masses.resize(cloth_count + end.len(), 0.0);
        }
        let contacts = settings
            .self_collision
            .then(|| fabelgeist_cloth::surface_contact::SurfaceContacts::new(masses.len(), faces));
        Self { masses, contacts }
    }
    fn resolve(
        &self,
        cloth: &mut Query<(&Mesh3d, &mut ClothSkin)>,
        mut previous: Vec<fabelgeist_math::Vec3>,
        body: &Option<Motion>,
        body_surface: &Option<ContactSurface>,
        start_fraction: f32,
        settings: &SimulationSettings,
    ) {
        if let Some(contacts) = &self.contacts {
            let mut positions: Vec<_> = cloth
                .iter()
                .flat_map(|(_, skin)| skin.current.iter().copied().map(collision_vector))
                .collect();
            if let Some((start, end, _)) = &body {
                previous.extend(
                    start
                        .iter()
                        .zip(end)
                        .map(|(a, b)| collision_vector(a.lerp(*b, start_fraction))),
                );
            }
            if let Some((current, _, _)) = &body_surface {
                positions.extend(current.iter().copied().map(collision_vector));
            }
            contacts.solve(
                &mut positions,
                &previous,
                &self.masses,
                settings.cloth_thickness,
                settings.contact_iterations,
            );
            let mut offset = 0;
            for (_, mut skin) in cloth {
                for v in 0..skin.current.len() {
                    let p = positions[offset];
                    let corrected = Vec3::new(p.x, p.y, p.z);
                    let normal = (corrected - skin.current[v]).normalize_or_zero();
                    let velocity = skin.current[v] - skin.previous[v];
                    let velocity = velocity - normal * velocity.dot(normal).min(0.0);
                    skin.current[v] = corrected;
                    skin.previous[v] = corrected - velocity;
                    offset += 1;
                }
            }
        }
    }
}

pub(super) fn simulate_outfit(
    cloth: &mut Query<(&Mesh3d, &mut ClothSkin)>,
    body: &Option<Motion>,
    targets: &[Option<Vec<Vec3>>],
    delta: f32,
    simulation: &SimulationSettings,
) {
    let count = simulation.substeps.max(1);
    let step = delta.min(1.0 / 30.0) / count as f32;
    let mut settings = simulation.clone();
    settings.substeps = 1;
    let contacts = OutfitContacts::new(cloth, body, &settings);
    for substep in 0..count {
        let t0 = substep as f32 / count as f32;
        let t1 = (substep + 1) as f32 / count as f32;
        let body_surface = body.as_ref().map(|(start, end, faces)| {
            let current: Vec<_> = start.iter().zip(end).map(|(a, b)| a.lerp(*b, t1)).collect();
            let bvh = fabelgeist_bvh::TriangleBvh::new(
                current.iter().copied().map(collision_vector).collect(),
                faces.clone(),
            );
            (current, faces.clone(), bvh)
        });
        // All cloth and body motion describe the SAME interval. A frame-end
        // correction misses crossings that occur between solver substeps.
        let previous: Vec<_> = cloth
            .iter()
            .flat_map(|(_, skin)| skin.current.iter().copied().map(collision_vector))
            .collect();
        for (index, (_, mut skin)) in cloth.iter_mut().enumerate() {
            simulate(
                &mut skin,
                targets[index].as_deref(),
                body_surface.as_ref(),
                step,
                &settings,
            );
        }
        contacts.resolve(cloth, previous, body, &body_surface, t0, &settings);
    }
}

pub(super) fn simulate(
    skin: &mut ClothSkin,
    targets: Option<&[Vec3]>,
    body: Option<&(Vec<Vec3>, Vec<[u32; 3]>, fabelgeist_bvh::TriangleBvh)>,
    dt: f32,
    settings: &SimulationSettings,
) {
    let substeps = settings.substeps.max(1);
    let step = dt / substeps as f32;
    let follow = match skin.preset.form() {
        GarmentForm::Legged => 0.22,
        GarmentForm::Skirted => 0.075,
        GarmentForm::Upper | GarmentForm::Fitted => 0.14,
    };
    for _ in 0..substeps {
        for v in 0..skin.current.len() {
            let position = skin.current[v];
            let velocity = (position - skin.previous[v]) * (1.0 - settings.damping);
            skin.previous[v] = position;
            skin.current[v] = position
                + velocity
                + Vec3::new(0.0, -settings.gravity, 0.0) * step * step
                + targets.map_or(Vec3::ZERO, |targets| {
                    (targets[v] - position) * (follow * settings.follow_strength).min(1.0)
                });
        }
        for _ in 0..settings.iterations {
            for &[a, b] in &skin.edges {
                let (a, b) = (a as usize, b as usize);
                let delta = skin.current[b] - skin.current[a];
                let length = delta.length();
                // Fabric rest lengths do not change when the skeleton moves.
                let target_length = (Vec3::from_array(skin.positions[b])
                    - Vec3::from_array(skin.positions[a]))
                .length();
                if length > 1e-7 {
                    let correction =
                        delta * (1.0 - target_length / length) * (0.5 * settings.stretch_stiffness);
                    skin.current[a] += correction;
                    skin.current[b] -= correction;
                }
            }
            if let Some((body_positions, body_faces, tree)) = body {
                for point in &mut skin.current {
                    let Some((triangle, closest, distance)) = tree.closest_point(
                        fabelgeist_math::Vec3::new(point.x, point.y, point.z),
                        settings.collision_distance,
                    ) else {
                        continue;
                    };
                    let [a, b, c] =
                        body_faces[triangle as usize].map(|v| body_positions[v as usize]);
                    let normal = (b - a).cross(c - a).normalize_or_zero();
                    let closest = Vec3::new(closest.x, closest.y, closest.z);
                    let signed = (*point - closest).dot(normal);
                    let margin = settings.collision_margin;
                    if signed < margin && distance < settings.collision_distance {
                        *point += normal * (margin - signed);
                    }
                }
            }
        }
    }
}
