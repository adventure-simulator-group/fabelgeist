use super::*;
use bevy::{
    animation::{AnimatedBy, AnimationTargetId},
    gltf::Gltf,
    mesh::{
        VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
};

#[derive(Resource, Default)]
pub struct WalkPreview {
    requested: Handle<Gltf>,
    graph: Option<Handle<AnimationGraph>>,
    node: Option<AnimationNodeIndex>,
    pub player: Option<Entity>,
    skeleton: Option<Entity>,
    pub joints: Vec<Entity>,
    pub inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
    pub playing: bool,
    pub physics: bool,
    pub ignore_cloth_weights: bool,
    pub simulation: SimulationSettings,
}

#[derive(Clone, Debug)]
pub struct SimulationSettings {
    pub self_collision: bool,
    pub cloth_thickness: f32,
    pub contact_iterations: u32,
    pub gravity: f32,
    pub damping: f32,
    pub stretch_stiffness: f32,
    pub follow_strength: f32,
    pub collision_margin: f32,
    pub collision_distance: f32,
    pub substeps: u32,
    pub iterations: u32,
}
impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            self_collision: true,
            cloth_thickness: 0.004,
            contact_iterations: 3,
            gravity: 9.81,
            damping: 0.015,
            stretch_stiffness: 0.96,
            follow_strength: 1.0,
            collision_margin: 0.006,
            collision_distance: 0.08,
            substeps: 2,
            iterations: 3,
        }
    }
}
impl WalkPreview {
    pub fn ready(&self) -> bool {
        self.player.is_some() && self.graph.is_some() && self.node.is_some()
    }

    pub fn start_or_resume(&self, player: &mut AnimationPlayer) {
        if player.playing_animations().next().is_none() {
            player
                .play(self.node.expect("ready walk preview has a graph node"))
                .repeat();
        } else {
            player.resume_all();
        }
    }
}

#[derive(Component)]
struct PreviewSkeleton;

pub fn request(mut preview: ResMut<WalkPreview>, assets: Res<AssetServer>) {
    preview.requested = assets.load("animations/biped/unarmed/walk.glb");
}

pub fn prepare(
    mut commands: Commands,
    mut preview: ResMut<WalkPreview>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    if preview.graph.is_none()
        && let Some(gltf) = gltfs.get(&preview.requested)
        && let Some(clip) = gltf.animations.first()
    {
        let (graph, node) = AnimationGraph::from_clip(clip.clone());
        preview.graph = Some(graphs.add(graph));
        preview.node = Some(node);
    }
    let (Some(player), Some(graph)) = (preview.player, preview.graph.clone()) else {
        return;
    };
    if let Ok(mut entity) = commands.get_entity(player) {
        entity.try_insert(AnimationGraphHandle(graph));
    }
}

pub fn rebuild(
    preview: &mut WalkPreview,
    commands: &mut Commands,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    model: &BodyModel,
    generated: &GeneratedCharacter,
) {
    if let Some(root) = preview.skeleton.take() {
        commands.entity(root).despawn();
    }
    if let Some(player) = preview.player.take() {
        commands.entity(player).despawn();
    }
    preview.playing = false;

    let player = commands
        .spawn((
            Name::new("Character walk player"),
            AnimationPlayer::default(),
        ))
        .id();
    if let Some(graph) = preview.graph.clone() {
        commands.entity(player).insert(AnimationGraphHandle(graph));
    }
    let skeleton_name = Name::new("Skeleton");
    let skeleton = commands
        .spawn((
            PreviewSkeleton,
            skeleton_name.clone(),
            Transform::default(),
            AnimationTargetId::from_names([&skeleton_name].into_iter()),
            AnimatedBy(player),
            Visibility::Hidden,
        ))
        .id();

    let character = &model.mhr.character;
    let globals: Vec<_> = generated
        .global_joint_states
        .iter()
        .copied()
        .map(mhr_state)
        .collect();
    let mut joints = Vec::with_capacity(character.skeleton.len());
    for (index, name) in character.skeleton.names.iter().enumerate() {
        let parent = character.skeleton.parents[index];
        let local = if parent < 0 {
            globals[index]
        } else {
            globals[parent as usize].inverse().compose(&globals[index])
        };
        let mut path = vec![skeleton_name.clone()];
        let mut lineage = vec![index];
        let mut ancestor = parent;
        while ancestor >= 0 {
            lineage.push(ancestor as usize);
            ancestor = character.skeleton.parents[ancestor as usize];
        }
        for joint in lineage.into_iter().rev() {
            path.push(Name::new(character.skeleton.names[joint].clone()));
        }
        let entity = commands
            .spawn((
                Name::new(name.clone()),
                bevy_transform(local),
                AnimationTargetId::from_names(path.iter()),
                AnimatedBy(player),
            ))
            .id();
        let parent_entity = if parent < 0 {
            skeleton
        } else {
            joints[parent as usize]
        };
        commands.entity(parent_entity).add_child(entity);
        joints.push(entity);
    }
    let matrices = globals
        .into_iter()
        .map(|state| {
            let inverse = state.inverse();
            Mat4::from_scale_rotation_translation(
                Vec3::splat(inverse.scale as f32),
                Quat::from_xyzw(
                    inverse.rotation[0] as f32,
                    inverse.rotation[1] as f32,
                    inverse.rotation[2] as f32,
                    inverse.rotation[3] as f32,
                ),
                Vec3::new(
                    inverse.translation[0] as f32,
                    inverse.translation[1] as f32,
                    inverse.translation[2] as f32,
                ),
            )
        })
        .collect::<Vec<_>>();
    preview.inverse_bindposes = inverse_bindposes.add(SkinnedMeshInverseBindposes::from(matrices));
    preview.player = Some(player);
    preview.skeleton = Some(skeleton);
    preview.joints = joints;
}

fn mhr_state(value: [f32; 8]) -> fabelgeist_mhr::math::Transform {
    fabelgeist_mhr::math::Transform {
        translation: [value[0] as f64, value[1] as f64, value[2] as f64],
        rotation: [
            value[3] as f64,
            value[4] as f64,
            value[5] as f64,
            value[6] as f64,
        ],
        scale: value[7] as f64,
    }
}

fn bevy_transform(value: fabelgeist_mhr::math::Transform) -> Transform {
    Transform {
        translation: Vec3::new(
            value.translation[0] as f32,
            value.translation[1] as f32,
            value.translation[2] as f32,
        ),
        rotation: Quat::from_xyzw(
            value.rotation[0] as f32,
            value.rotation[1] as f32,
            value.rotation[2] as f32,
            value.rotation[3] as f32,
        ),
        scale: Vec3::splat(value.scale as f32),
    }
}

pub fn skin_mesh(mesh: &mut Mesh, indices: &[[u32; 8]], weights: &[[f32; 8]]) {
    let mut joint_indices = Vec::with_capacity(indices.len());
    let mut joint_weights = Vec::with_capacity(weights.len());
    for (indices, weights) in indices.iter().zip(weights) {
        let mut influences: Vec<_> = indices
            .iter()
            .copied()
            .zip(weights.iter().copied())
            .collect();
        influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        let ids = std::array::from_fn::<_, 4, _>(|i| influences[i].0 as u16);
        let mut values = std::array::from_fn::<_, 4, _>(|i| influences[i].1);
        let total: f32 = values.iter().sum();
        if total > 0.0 {
            for weight in &mut values {
                *weight /= total;
            }
        } else {
            values[0] = 1.0;
        }
        joint_indices.push(ids);
        joint_weights.push(values);
    }
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(joint_indices),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights);
}

pub fn skin(preview: &WalkPreview) -> Option<SkinnedMesh> {
    (!preview.joints.is_empty()).then(|| SkinnedMesh {
        inverse_bindposes: preview.inverse_bindposes.clone(),
        joints: preview.joints.clone(),
    })
}

// Preserve all eight exported influences. Reducing each vertex independently
// to four can discard the opposite leg precisely at the centre transition.
#[derive(Component)]
pub struct BodySkin {
    pub positions: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub indices: Vec<[u32; 8]>,
    pub weights: Vec<[f32; 8]>,
}

#[derive(Component)]
pub struct ClothSkin {
    name: String,
    positions: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    edges: Vec<[u32; 2]>,
    indices: Vec<[u32; 8]>,
    weights: Vec<[f32; 8]>,
    current: Vec<Vec3>,
    previous: Vec<Vec3>,
    simulating: bool,
}

impl ClothSkin {
    pub fn new(
        name: String,
        positions: Vec<[f32; 3]>,
        _normals: Vec<[f32; 3]>,
        faces: Vec<[u32; 3]>,
        indices: Vec<[u32; 8]>,
        weights: Vec<[f32; 8]>,
    ) -> Self {
        let mut edges = std::collections::BTreeSet::new();
        for face in &faces {
            for [a, b] in [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]] {
                edges.insert(if a < b { [a, b] } else { [b, a] });
            }
        }
        let current: Vec<Vec3> = positions.iter().copied().map(Vec3::from_array).collect();
        Self {
            name,
            positions,
            faces,
            edges: edges.into_iter().collect(),
            indices,
            weights,
            previous: current.clone(),
            current,
            simulating: false,
        }
    }
}

#[derive(Default)]
pub struct BodyMotion {
    rig: Option<Entity>,
    positions: Vec<Vec3>,
}

pub fn deform_cloth(
    preview: Res<WalkPreview>,
    time: Res<Time>,
    binds: Res<Assets<SkinnedMeshInverseBindposes>>,
    joints: Query<&GlobalTransform>,
    bodies: Query<&BodySkin>,
    mut cloth: Query<(&Mesh3d, &mut ClothSkin)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut body_motion: Local<BodyMotion>,
) {
    if !preview.playing {
        return;
    }
    let Some(bind) = binds.get(&preview.inverse_bindposes) else {
        return;
    };
    let Some(matrices) = preview
        .joints
        .iter()
        .zip(bind.iter())
        .map(|(entity, inverse)| {
            joints
                .get(*entity)
                .ok()
                .map(|global| global.to_matrix() * *inverse)
        })
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    let rig = preview.joints.first().copied();
    let body = bodies.iter().next().map(|body| {
        let end = skinned_positions(&body.positions, &body.indices, &body.weights, &matrices);
        let start = if body_motion.rig == rig && body_motion.positions.len() == end.len() {
            body_motion.positions.clone()
        } else {
            body.positions
                .iter()
                .copied()
                .map(Vec3::from_array)
                .collect()
        };
        (start, end, body.faces.clone())
    });
    let targets: Vec<_> = cloth
        .iter()
        .map(|(_, skin)| {
            if preview.physics && preview.ignore_cloth_weights {
                None
            } else {
                Some(skinned_positions(
                    &skin.positions,
                    &skin.indices,
                    &skin.weights,
                    &matrices,
                ))
            }
        })
        .collect();

    if preview.physics {
        let count = preview.simulation.substeps.max(1);
        let step = time.delta_secs().min(1.0 / 30.0) / count as f32;
        let mut settings = preview.simulation.clone();
        settings.substeps = 1;
        let mut faces = Vec::new();
        let mut cloth_count = 0;
        for (_, mut skin) in &mut cloth {
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
        let contacts = preview
            .simulation
            .self_collision
            .then(|| fabelgeist_cloth::surface_contact::SurfaceContacts::new(masses.len(), faces));
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
            let mut previous: Vec<_> = cloth
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
            if let Some(contacts) = &contacts {
                let mut positions: Vec<_> = cloth
                    .iter()
                    .flat_map(|(_, skin)| skin.current.iter().copied().map(collision_vector))
                    .collect();
                if let Some((start, end, _)) = &body {
                    previous.extend(
                        start
                            .iter()
                            .zip(end)
                            .map(|(a, b)| collision_vector(a.lerp(*b, t0))),
                    );
                }
                if let Some((current, _, _)) = &body_surface {
                    positions.extend(current.iter().copied().map(collision_vector));
                }
                contacts.solve(
                    &mut positions,
                    &previous,
                    &masses,
                    settings.cloth_thickness,
                    settings.contact_iterations,
                );
                let mut offset = 0;
                for (_, mut skin) in &mut cloth {
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
    } else {
        for (index, (_, mut skin)) in cloth.iter_mut().enumerate() {
            let target = targets[index]
                .as_ref()
                .expect("weighted animation computes targets");
            skin.current.clone_from(target);
            skin.previous.clone_from(target);
            skin.simulating = false;
        }
    }
    if let Some((_, end, _)) = body {
        body_motion.rig = rig;
        body_motion.positions = end;
    } else {
        body_motion.positions.clear();
        body_motion.rig = None;
    }
    for (handle, skin) in &cloth {
        if let Some(mut mesh) = meshes.get_mut(handle) {
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                skin.current
                    .iter()
                    .map(|p| p.to_array())
                    .collect::<Vec<_>>(),
            );
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                surface_normals(&skin.current, &skin.faces)
                    .iter()
                    .map(|n| n.to_array())
                    .collect::<Vec<_>>(),
            );
        }
    }
}

fn collision_vector(p: Vec3) -> fabelgeist_math::Vec3 {
    fabelgeist_math::Vec3::new(p.x, p.y, p.z)
}
fn skinned_positions(
    positions: &[[f32; 3]],
    indices: &[[u32; 8]],
    weights: &[[f32; 8]],
    matrices: &[Mat4],
) -> Vec<Vec3> {
    positions
        .iter()
        .enumerate()
        .map(|(v, p)| blend_point(Vec3::from_array(*p), indices[v], weights[v], matrices))
        .collect()
}

fn simulate(
    skin: &mut ClothSkin,
    targets: Option<&[Vec3]>,
    body: Option<&(Vec<Vec3>, Vec<[u32; 3]>, fabelgeist_bvh::TriangleBvh)>,
    dt: f32,
    settings: &SimulationSettings,
) {
    let substeps = settings.substeps.max(1);
    let step = dt / substeps as f32;
    let follow = if skin.name.starts_with("Trousers") {
        0.22
    } else if skin.name.starts_with("Skirt") || skin.name.starts_with("Dress") {
        0.075
    } else {
        0.14
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

fn surface_normals(positions: &[Vec3], faces: &[[u32; 3]]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; positions.len()];
    for face in faces {
        let [a, b, c] = face.map(|v| v as usize);
        let normal = (positions[b] - positions[a]).cross(positions[c] - positions[a]);
        normals[a] += normal;
        normals[b] += normal;
        normals[c] += normal;
    }
    normals.into_iter().map(|n| n.normalize_or_zero()).collect()
}

fn blend_point(point: Vec3, indices: [u32; 8], weights: [f32; 8], matrices: &[Mat4]) -> Vec3 {
    indices
        .into_iter()
        .zip(weights)
        .filter(|(_, w)| *w > 0.0)
        .fold(Vec3::ZERO, |sum, (joint, weight)| {
            sum + matrices[joint as usize].transform_point3(point) * weight
        })
}

#[cfg(test)]
mod deformation_tests {
    use super::*;

    #[test]
    fn cloth_preserves_opposite_leg_motion_in_last_four_influences() {
        let matrices = (0..8)
            .map(|i| Mat4::from_translation(Vec3::Z * if i < 4 { 1.0 } else { -1.0 }))
            .collect::<Vec<_>>();
        let point = Vec3::new(0.0, 0.5, 0.0);
        let centre = blend_point(point, [0, 1, 2, 3, 4, 5, 6, 7], [0.125; 8], &matrices);
        assert!((centre - point).length() < 1e-6);
        let left = blend_point(
            point,
            [0, 1, 2, 3, 4, 5, 6, 7],
            [0.2, 0.2, 0.2, 0.2, 0.05, 0.05, 0.05, 0.05],
            &matrices,
        );
        assert!((left.z - 0.6).abs() < 1e-6);
    }

    #[test]
    fn simulated_edges_preserve_the_draped_rest_length() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let mut skin = ClothSkin::new(
            "Trousers".into(),
            positions.clone(),
            vec![[0.0, 0.0, 1.0]; 3],
            vec![[0, 1, 2]],
            vec![[0; 8]; 3],
            vec![[0.0; 8]; 3],
        );
        skin.current[1].x = 3.0;
        skin.previous.clone_from(&skin.current);
        let targets = positions
            .into_iter()
            .map(Vec3::from_array)
            .collect::<Vec<_>>();
        simulate(
            &mut skin,
            Some(&targets),
            None,
            0.0,
            &SimulationSettings::default(),
        );
        assert!((skin.current[1] - skin.current[0]).length() < 1.2);
    }

    #[test]
    fn unweighted_physics_preserves_rest_shape_and_ignores_invalid_skin_weights() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let mut skin = ClothSkin::new(
            "Trousers".into(),
            positions.clone(),
            vec![[0.0, 0.0, 1.0]; 3],
            vec![[0, 1, 2]],
            vec![[u32::MAX; 8]; 3],
            vec![[f32::NAN; 8]; 3],
        );
        // A translated garment must not be pulled back toward its rest location.
        for point in &mut skin.current {
            *point += Vec3::X * 5.0;
        }
        skin.previous = skin.current.clone();
        simulate(
            &mut skin,
            None,
            None,
            1.0 / 60.0,
            &SimulationSettings::default(),
        );
        for (v, rest) in positions.iter().enumerate() {
            assert!((skin.current[v].x - rest[0] - 5.0).abs() < 1e-5);
            assert!(skin.current[v].y < rest[1]);
            assert!(skin.current[v].is_finite());
        }
        assert!(((skin.current[1] - skin.current[0]).length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn animation_contact_world_separates_two_garments_without_launching_them() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world.insert_resource(Assets::<Mesh>::default());
        let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
        let inverse_bindposes = binds.add(SkinnedMeshInverseBindposes::from(vec![Mat4::IDENTITY]));
        world.insert_resource(binds);
        let joint = world.spawn(GlobalTransform::IDENTITY).id();
        world.insert_resource(WalkPreview {
            playing: true,
            physics: true,
            ignore_cloth_weights: true,
            joints: vec![joint],
            inverse_bindposes,
            simulation: SimulationSettings {
                gravity: 0.0,
                ..Default::default()
            },
            ..Default::default()
        });
        let mut entities = Vec::new();
        for height in [-0.001, 0.001] {
            let positions = vec![
                [-1.0, height, -1.0],
                [0.0, height, 1.0],
                [1.0, height, -1.0],
            ];
            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );
            let handle = world.resource_mut::<Assets<Mesh>>().add(mesh);
            let skin = ClothSkin::new(
                "Dress".into(),
                positions,
                vec![],
                vec![[0, 1, 2]],
                vec![[0; 8]; 3],
                vec![[0.0; 8]; 3],
            );
            entities.push(world.spawn((Mesh3d(handle), skin)).id());
        }
        world.run_system_once(deform_cloth).unwrap();
        let a = world.get::<ClothSkin>(entities[0]).unwrap();
        let b = world.get::<ClothSkin>(entities[1]).unwrap();
        let gap = (b.current.iter().map(|p| p.y).sum::<f32>()
            - a.current.iter().map(|p| p.y).sum::<f32>())
            / 3.0;
        assert!(gap > 0.0035, "garments stayed only {gap} m apart");
        for skin in [a, b] {
            for (p, previous) in skin.current.iter().zip(&skin.previous) {
                assert!(
                    (*p - *previous).length() < 1e-5,
                    "contact injected velocity"
                );
            }
        }
    }

    #[test]
    fn animated_body_ccd_retains_pose_history_across_frames() {
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world.insert_resource(Assets::<Mesh>::default());
        let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
        let inverse_bindposes = binds.add(SkinnedMeshInverseBindposes::from(vec![Mat4::IDENTITY]));
        world.insert_resource(binds);
        let joint = world.spawn(GlobalTransform::IDENTITY).id();
        world.insert_resource(WalkPreview {
            playing: true,
            physics: true,
            ignore_cloth_weights: true,
            joints: vec![joint],
            inverse_bindposes,
            simulation: SimulationSettings {
                gravity: 0.0,
                ..Default::default()
            },
            ..Default::default()
        });
        world.spawn(BodySkin {
            positions: vec![[-10.0, -10.0, -1.0], [10.0, -10.0, -1.0], [0.0, 10.0, -1.0]],
            faces: vec![[0, 1, 2]],
            indices: vec![[0; 8]; 3],
            weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 3],
        });
        let mesh = world.resource_mut::<Assets<Mesh>>().add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        ));
        let cloth = world
            .spawn((
                Mesh3d(mesh),
                ClothSkin::new(
                    "Dress".into(),
                    vec![[-0.1, -0.1, 0.0], [0.1, -0.1, 0.0], [0.0, 0.1, 0.0]],
                    vec![],
                    vec![[0, 1, 2]],
                    vec![[0; 8]; 3],
                    vec![[0.0; 8]; 3],
                ),
            ))
            .id();
        let mut schedule = Schedule::default();
        schedule.add_systems(deform_cloth);
        schedule.run(&mut world);
        // Each jump exceeds the discrete body's collision-search distance.
        // Neither final frame alone would discover the swept plane crossing.
        for displacement in [2.0, 4.0] {
            *world.get_mut::<GlobalTransform>(joint).unwrap() =
                GlobalTransform::from_translation(Vec3::Z * displacement);
            schedule.run(&mut world);
            let skin = world.get::<ClothSkin>(cloth).unwrap();
            assert!(
                skin.current.iter().all(|p| p.z > displacement - 1.0),
                "moving body tunneled through cloth: {:?}",
                skin.current
            );
            assert!(skin.current.iter().all(|p| p.is_finite()));
        }
    }
}
