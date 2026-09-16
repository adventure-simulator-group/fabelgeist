//! Cache pebble assets and place only patches clear of managed ground.
use super::*;
use adventuresim_tactical_core::prelude::GroundSurface;
use bevy::prelude::Handle;

pub(in crate::presentation::ground_scatter) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pebble_materials: &mut Assets<TacticalPebbleMaterial>,
    billboard_materials: &mut Assets<TacticalPebbleBillboardMaterial>,
    terrain: &SceneTerrain,
    ground: &SceneGround,
    base_seed: u64,
) {
    let half_extent = ground.grid_scale() * 0.5;
    let PatchAssets {
        hero_meshes,
        near_meshes,
        billboard_meshes,
        pebble_counts,
        stone_material,
        woodland_stone_material,
        billboard_material,
    } = PatchAssets::new(meshes, pebble_materials, billboard_materials, half_extent);

    for (index, sample) in ground.samples().iter().enumerate() {
        let Some(PatchPlacement {
            variant,
            woodland,
            transform,
        }) = PatchPlacement::new(ground, terrain, index, *sample, base_seed, half_extent)
        else {
            continue;
        };
        commands.spawn((
            Name::new(if woodland {
                "Tactical woodland hero pebble patch"
            } else {
                "Tactical loose-stone hero pebble patch"
            }),
            GroundScatterLayer::LooseStone,
            LooseStonePebblePatch::hero(pebble_counts[variant]),
            NotShadowCaster,
            Mesh3d(hero_meshes[variant].clone()),
            MeshMaterial3d(if woodland {
                woodland_stone_material.clone()
            } else {
                stone_material.clone()
            }),
            pebble_lod_visibility(PebbleMeshLod::Hero),
            transform,
        ));
        commands.spawn((
            Name::new(if woodland {
                "Tactical woodland near pebble patch"
            } else {
                "Tactical loose-stone near pebble patch"
            }),
            GroundScatterLayer::LooseStone,
            LooseStonePebblePatch {
                physical_pebbles: 0,
            },
            NotShadowCaster,
            Mesh3d(near_meshes[variant].clone()),
            MeshMaterial3d(if woodland {
                woodland_stone_material.clone()
            } else {
                stone_material.clone()
            }),
            pebble_lod_visibility(PebbleMeshLod::Near),
            transform,
        ));
        if !woodland {
            commands.spawn((
                Name::new("Tactical loose-stone billboard pebble patch"),
                GroundScatterLayer::LooseStone,
                LooseStonePebblePatch {
                    physical_pebbles: 0,
                },
                // The shader yaws each pebble quad toward the camera, so the
                // mesh's static bounds would mis-cull; this rotation-safe box
                // restores frustum culling for off-screen patches.
                bevy::camera::primitives::Aabb {
                    center: bevy::math::Vec3A::new(0.0, 0.1, 0.0),
                    half_extents: bevy::math::Vec3A::new(half_extent + 0.3, 0.4, half_extent + 0.3),
                },
                NotShadowCaster,
                Mesh3d(billboard_meshes[variant].clone()),
                MeshMaterial3d(billboard_material.clone()),
                pebble_lod_visibility(PebbleMeshLod::Billboard),
                transform,
            ));
        }
    }
}

struct PatchAssets {
    hero_meshes: Vec<Handle<Mesh>>,
    near_meshes: Vec<Handle<Mesh>>,
    billboard_meshes: Vec<Handle<Mesh>>,
    pebble_counts: Vec<usize>,
    stone_material: Handle<TacticalPebbleMaterial>,
    woodland_stone_material: Handle<TacticalPebbleMaterial>,
    billboard_material: Handle<TacticalPebbleBillboardMaterial>,
}
impl PatchAssets {
    fn new(
        meshes: &mut Assets<Mesh>,
        pebble_materials: &mut Assets<TacticalPebbleMaterial>,
        billboard_materials: &mut Assets<TacticalPebbleBillboardMaterial>,
        half_extent: f32,
    ) -> Self {
        let mut hero_meshes = Vec::new();
        let mut near_meshes = Vec::new();
        let mut billboard_meshes = Vec::new();
        let mut pebble_counts = Vec::new();
        for density in PebbleDensity::ALL {
            for variant in 0..MESH_VARIANTS {
                let seed = splitmix64(0x7065_6262_6c65_0000 ^ variant);
                let hero = pebble_patch_mesh(seed, PebbleMeshLod::Hero, half_extent, density);
                pebble_counts.push(hero.count_vertices() / HERO_PEBBLE_VERTICES);
                hero_meshes.push(meshes.add(hero));
                near_meshes.push(meshes.add(pebble_patch_mesh(
                    seed,
                    PebbleMeshLod::Near,
                    half_extent,
                    density,
                )));
                billboard_meshes.push(meshes.add(pebble_billboard_patch_mesh(
                    seed,
                    half_extent,
                    density,
                )));
            }
        }
        let stone_material = pebble_materials.add(TacticalPebbleMaterial::new(rock_color(
            RockLithology::Granite,
        )));
        let woodland_stone_material =
            pebble_materials.add(TacticalPebbleMaterial::new(Color::srgb_u8(104, 91, 70)));
        let billboard_material = billboard_materials.add(TacticalPebbleBillboardMaterial {
            color: Vec4::from_array(
                rock_color(RockLithology::Granite)
                    .to_linear()
                    .to_f32_array(),
            ),
            lighting: Vec3::new(0.35, 0.86, 0.25).normalize().extend(1.0),
            ambient: Vec4::new(1.0, 1.0, 1.0, 0.28),
        });

        Self {
            hero_meshes,
            near_meshes,
            billboard_meshes,
            pebble_counts,
            stone_material,
            woodland_stone_material,
            billboard_material,
        }
    }
}
struct PatchPlacement {
    variant: usize,
    woodland: bool,
    transform: Transform,
}
impl PatchPlacement {
    fn new(
        ground: &SceneGround,
        terrain: &SceneTerrain,
        index: usize,
        sample: GroundSurface,
        base_seed: u64,
        half_extent: f32,
    ) -> Option<Self> {
        if !matches!(
            sample.cover,
            GroundCover::LooseStone | GroundCover::LeafLitter
        ) {
            return None;
        }
        let grid_x = index % ground.grid_width();
        let grid_z = index / ground.grid_width();
        let position = Vec2::new(
            grid_x as f32 * ground.grid_scale() - ground.width() * 0.5,
            grid_z as f32 * ground.grid_scale() - ground.depth() * 0.5,
        );
        // The patch can rotate and tilt. Exclude its full existing bounds,
        // including pebble overhang, from exact managed-ground ownership.
        let envelope_radius = Vec3::new(half_extent + 0.3, 0.4, half_extent + 0.3).length();
        if ground.urban.overlaps_disk(position, envelope_radius) {
            return None;
        }
        let (Some(height), Some(normal)) =
            (terrain.height_at(position), terrain.normal_at(position))
        else {
            return None;
        };
        if normal.y < 0.72 {
            return None;
        }
        let hash = splitmix64(base_seed ^ index as u64 ^ 0x7374_6f6e_655f_7363);
        let woodland = sample.cover == GroundCover::LeafLitter;
        let density = if woodland {
            // Every woodland cell gets a sparse candidate patch. Individual
            // survival still leaves irregular gaps, but keeping the patches
            // continuous yields roughly one visible 3--8 cm stone per square
            // metre instead of making rocks disappear from review frames.
            PebbleDensity::Woodland
        } else {
            let coverage = scree_patch_coverage(base_seed, position, normal);
            if coverage >= 0.61 {
                PebbleDensity::Dense
            } else if coverage >= 0.34 {
                PebbleDensity::Sparse
            } else {
                return None;
            }
        };
        let variant = density.asset_offset() + (hash % MESH_VARIANTS) as usize;
        let yaw = Quat::from_rotation_y(
            unit_hash(splitmix64(hash ^ 0x55d8_093b)) * core::f32::consts::TAU,
        );
        let transform = Transform::from_xyz(
            position.x,
            height + if woodland { -0.006 } else { 0.006 },
            position.y,
        )
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, normal) * yaw)
        .with_scale(Vec3::splat(if woodland { 0.58 } else { 1.0 }));

        Some(Self {
            variant,
            woodland,
            transform,
        })
    }
}
