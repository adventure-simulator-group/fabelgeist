//! Distant rock and tree presentation from the admitted source terrain.
use super::*;
use fabelgeist_determinism::Seed;

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(super) fn spawn_vista_rocks(
    commands: &mut Commands,
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_half_extent: Vec2,
    playable_terrain: &SceneTerrain,
    scene_seed: Seed,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let recipe = RockRecipe {
        seed: 0x7669_7374_615f_726f.into(),
        archetype: RockArchetype::Rounded,
        lithology: RockLithology::Granite,
        dimensions_cm: [120, 92, 108],
        collision_radius_cm: 80,
    };
    let near_mesh = meshes.add(super::obstacles::rock::procedural_rock_mesh(recipe));
    let far_mesh = meshes.add(vista_rock_mesh());
    let material = materials.add(StandardMaterial {
        base_color: super::obstacles::rock::rock_color(RockLithology::Granite),
        perceptual_roughness: 0.92,
        ..default()
    });
    let spacing = 24.0;
    let outer = playable_half_extent + Vec2::splat(420.0);
    let minimum = (-outer / spacing).floor().as_ivec2();
    let maximum = (outer / spacing).ceil().as_ivec2();
    for z in minimum.y..=maximum.y {
        for x in minimum.x..=maximum.x {
            let (hash, jitter) = streams::rock_cell(scene_seed, x, z, spacing);
            let point = Vec2::new(x as f32, z as f32) * spacing + jitter;
            if point.x.abs() <= playable_half_extent.x + 2.0
                && point.y.abs() <= playable_half_extent.y + 2.0
            {
                continue;
            }
            let Some(sample) = sample_vista_environment(lod, point) else {
                continue;
            };
            let exposed = bps(sample.hilly_bps)
                * (1.0 - bps(sample.water_bps))
                * (1.0 - bps(sample.wetland_bps) * 0.75)
                * (1.0 - bps(sample.canopy_bps) * 0.42);
            if streams::ROCK_PRESENCE.rng(hash, &[]).inclusive_unit_f32() > exposed * 0.46 {
                continue;
            }
            let lift = 0.08;
            let Some(mut transform) = vista_scatter_transform(
                lod,
                coarser_lod,
                playable_terrain,
                playable_half_extent,
                point,
                hash,
                lift,
            ) else {
                continue;
            };
            let scale = 0.55 + streams::ROCK_SCALE.rng(hash, &[]).inclusive_unit_f32() * 1.35;
            transform.scale = Vec3::new(scale, scale * 0.72, scale * 0.9);
            spawn_rock_bands(
                commands, lod.level, &near_mesh, &far_mesh, &material, transform,
            );
        }
    }
}

pub(super) fn sample_vista_environment(lod: &VistaLod, world: Vec2) -> Option<EnvironmentalSample> {
    let width = usize::from(lod.width);
    let depth = usize::from(lod.depth);
    let origin = Vec2::new(
        lod.origin_east_metres as f32,
        lod.origin_north_metres as f32,
    );
    let coordinate = (world - origin) / lod.spacing_metres
        + Vec2::new((width - 1) as f32 * 0.5, (depth - 1) as f32 * 0.5);
    if coordinate.x < 0.0
        || coordinate.y < 0.0
        || coordinate.x > (width - 1) as f32
        || coordinate.y > (depth - 1) as f32
    {
        return None;
    }
    let nearest = coordinate.round().as_uvec2();
    lod.environment
        .get(nearest.y as usize * width + nearest.x as usize)
        .copied()
}

pub(super) fn vista_rock_mesh() -> Mesh {
    let vertices = [
        Vec3::new(-0.58, -0.36, -0.48),
        Vec3::new(0.52, -0.36, -0.44),
        Vec3::new(0.61, -0.28, 0.42),
        Vec3::new(-0.49, -0.31, 0.55),
        Vec3::new(-0.37, 0.31, -0.33),
        Vec3::new(0.32, 0.42, -0.29),
        Vec3::new(0.39, 0.27, 0.31),
        Vec3::new(-0.31, 0.36, 0.38),
    ];
    let faces = [
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
        [3, 4, 7],
    ];
    let mut positions = Vec::with_capacity(faces.len() * 3);
    let mut normals = Vec::with_capacity(faces.len() * 3);
    for [a, b, c] in faces {
        let normal = (vertices[b] - vertices[a])
            .cross(vertices[c] - vertices[a])
            .normalize_or_zero();
        positions.extend([
            vertices[a].to_array(),
            vertices[b].to_array(),
            vertices[c].to_array(),
        ]);
        normals.extend([normal.to_array(); 3]);
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh
}

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(super) fn spawn_vista_trees(
    commands: &mut Commands,
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_half_extent: Vec2,
    scene_digest: &str,
    environment: Option<&SceneEnvironment>,
    terrain: &SceneTerrain,
    meshes: &mut Assets<Mesh>,
    tree_materials: &mut Assets<TacticalTreeImpostorMaterial>,
    images: &mut Assets<Image>,
    cache: &mut VistaTreePresentationCache,
    prepared: Option<&PreparedTreeImpostorAsset>,
) {
    let width = usize::from(lod.width);
    let depth = usize::from(lod.depth);
    let center = Vec2::new((width - 1) as f32, (depth - 1) as f32) * 0.5;
    let scene_seed = stable_text_seed(scene_digest);
    for z in 0..depth - 1 {
        for x in 0..width - 1 {
            let sample = lod.environment[z * width + x];
            let canopy = bps(sample.canopy_bps)
                * (1.0 - bps(sample.water_bps))
                * (1.0 - bps(sample.cultivation_bps) * 0.85);
            // A regional source cell represents a stand, not individual
            // stems. Keep a physical-area-scaled silhouette sample; the
            // terrain material carries the remaining aggregate canopy.
            let candidate_count = vista_tree_candidate_count(
                canopy,
                lod.spacing_metres,
                streams::tree_count_seed(scene_seed, x, z),
            )
            .min(if lod.spacing_metres <= 250.0 { 24 } else { 3 });
            if candidate_count == 0 {
                continue;
            }
            let cell_min = (Vec2::new(x as f32, z as f32) - center) * lod.spacing_metres;
            for candidate in 0..candidate_count {
                let hash = streams::tree_seed(scene_seed, x, z, candidate);
                let local = cell_min + streams::tree_jitter(hash) * lod.spacing_metres;
                if local.x.abs() <= playable_half_extent.x + 7.0
                    && local.y.abs() <= playable_half_extent.y + 7.0
                {
                    continue;
                }
                let world = local
                    + Vec2::new(
                        lod.origin_east_metres as f32,
                        lod.origin_north_metres as f32,
                    );
                let Some(height) = tree_root_height(terrain, lod, coarser_lod, world) else {
                    continue;
                };
                // One calibrated whole-tree atlas avoids baking for every source cell.
                let variant_seed =
                    crate::presentation::obstacles::tree::specimen::oak_variant_seed(0);
                let species = vista_tree_species(environment, local);
                let cached = ensure_vista_tree_variant(
                    variant_seed,
                    0.5,
                    species,
                    meshes,
                    tree_materials,
                    images,
                    cache,
                    prepared,
                );
                // Each atlas represents the visible crown mass of a small
                // stand at regional distance, not a survey-accurate stem.
                let scale = vista_tree_scale(
                    lod.spacing_metres,
                    streams::TREE_SCALE.rng(hash, &[]).inclusive_unit_f32(),
                );
                let card_height = cached
                    .provenance
                    .records
                    .first()
                    .map_or(12.0, |record| record.projected_bounds.w);
                commands.spawn((
                    Name::new(format!("Distant vista {} billboard", species.name())),
                    VistaTerrain(lod.level),
                    VistaTreePresentation,
                    // The impostor shader yaws the card toward the camera, so
                    // the mesh's static bounds would mis-cull near screen
                    // edges. This rotation-safe box restores frustum culling:
                    // off-screen stands previously always rendered through
                    // `NoFrustumCulling`, roughly half the vista vertex cost.
                    billboard_bounds(card_height),
                    NotShadowCaster,
                    Mesh3d(cached.mesh.clone()),
                    MeshMaterial3d(cached.material.clone()),
                    cached.provenance.clone(),
                    vista_tree_visibility(lod.spacing_metres, card_height, scale),
                    Transform::from_xyz(local.x, height, local.y).with_scale(Vec3::splat(scale)),
                ));
            }
        }
    }
}

pub(super) const VISTA_TREE_MAX_ANGULAR_HEIGHT_RADIANS: f32 = 16.0_f32.to_radians();

pub(super) fn vista_tree_scale(spacing_metres: f32, variation: f32) -> f32 {
    // Candidate count already represents regional stand density. Each card
    // must remain one plausible tree; scaling a single trunk into a 32-metre
    // stand creates the near-ring columns seen from the playable scene.
    let coarse_scale = if spacing_metres <= 250.0 { 1.0 } else { 1.25 };
    (0.85 + variation.clamp(0.0, 1.0) * 0.4) * coarse_scale
}

pub(super) fn vista_tree_visibility(
    spacing_metres: f32,
    unscaled_card_height: f32,
    scale: f32,
) -> VisibilityRange {
    let scaled_height = unscaled_card_height.max(0.0) * scale.max(0.0);
    // Keep regional stand cards out of the near field. The first visible
    // sample is at most 16 degrees; the fade completes farther away. Using
    // this distance as the end of the fade admitted partially visible 17-19
    // degree cards, contradicting the background-size contract.
    let first_visible_distance =
        scaled_height / (2.0 * (VISTA_TREE_MAX_ANGULAR_HEIGHT_RADIANS * 0.5).tan());
    VisibilityRange {
        start_margin: first_visible_distance..(first_visible_distance * 1.12),
        end_margin: if spacing_metres <= 250.0 {
            1_600.0..1_900.0
        } else {
            4_600.0..5_200.0
        },
        use_aabb: false,
    }
}

pub(super) fn vista_tree_candidate_count(canopy: f32, spacing_metres: f32, seed: Seed) -> usize {
    let expected = canopy.clamp(0.0, 1.0) * spacing_metres * spacing_metres / 3_200.0;
    expected.floor() as usize
        + usize::from(
            streams::TREE_COUNT_FRACTION
                .rng(seed, &[])
                .inclusive_unit_f32()
                < expected.fract(),
        )
}

fn spawn_rock_bands(
    commands: &mut Commands,
    level: VistaLevelIndex,
    near_mesh: &Handle<Mesh>,
    far_mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    transform: Transform,
) {
    for (name, mesh, visibility) in [
        (
            "Tactical vista rock mesh",
            near_mesh.clone(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 90.0..112.0,
                use_aabb: false,
            },
        ),
        (
            "Tactical vista rock low LOD",
            far_mesh.clone(),
            VisibilityRange {
                start_margin: 88.0..110.0,
                end_margin: 360.0..430.0,
                use_aabb: false,
            },
        ),
    ] {
        commands.spawn((
            Name::new(name),
            VistaTerrain(level),
            VistaRockPresentation,
            NotShadowCaster,
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            visibility,
            transform,
        ));
    }
}

fn billboard_bounds(card_height: f32) -> bevy::camera::primitives::Aabb {
    bevy::camera::primitives::Aabb {
        center: bevy::math::Vec3A::new(0.0, card_height * 0.5, 0.0),
        half_extents: bevy::math::Vec3A::new(
            card_height * 0.8,
            card_height * 0.6,
            card_height * 0.8,
        ),
    }
}
