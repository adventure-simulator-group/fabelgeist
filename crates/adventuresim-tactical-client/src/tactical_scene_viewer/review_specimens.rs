//! Named Recipe-only diagnostic geometry, excluded before Grown generation.
use super::*;
use crate::presentation::{
    TreeContentRequest, oak_bark_material, oak_leaf_material, oak_review_terminal_specimen,
    terrain_heightmap_image,
};

#[derive(Default)]
pub(super) struct ReviewSpecimens {
    pub focus: Option<Vec3>,
    pub camera: Option<Vec3>,
    pub entities: Vec<Entity>,
    pub leaves: Vec<(Entity, TreeLeafRepresentation)>,
}

/// Native capture geometry boundary: world metres and admitted canopy/azimuth.
pub(super) struct Request<'a> {
    pub tree: Vec3,
    pub terrain: &'a SceneTerrain,
    pub canopy_bps: u16,
    pub tree_review_azimuth_degrees: f32,
}

pub(super) struct Assets<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub meshes: &'a mut bevy::asset::Assets<Mesh>,
    pub materials: &'a mut bevy::asset::Assets<StandardMaterial>,
    pub bark_materials: &'a mut bevy::asset::Assets<TacticalTreeBarkMaterial>,
    pub leaf_card_materials: &'a mut bevy::asset::Assets<TacticalTreeLeafCardMaterial>,
    pub images: &'a mut bevy::asset::Assets<Image>,
    pub procedural_assets: &'a ProceduralTextureAssets,
}

struct Leaves {
    textured_leaf_mesh: Mesh,
    leaf_card_mesh: Mesh,
    leaf_material: Handle<TacticalTreeLeafCardMaterial>,
    specimen_origin: Vec3,
    local_focus: Vec3,
    camera_direction: Vec3,
    tree_review_azimuth_degrees: f32,
}

impl Assets<'_, '_, '_> {
    pub(super) fn spawn(&mut self, request: Request<'_>) -> ReviewSpecimens {
        let Request {
            tree,
            terrain,
            canopy_bps,
            tree_review_azimuth_degrees,
        } = request;
        self.backdrop(tree);
        let Self {
            commands,
            meshes,
            materials,
            bark_materials,
            leaf_card_materials,
            images,
            procedural_assets,
        } = self;
        let mut output = ReviewSpecimens::default();
        let specimen_origin = tree + Vec3::Y * 15.0;
        let (
            branch_mesh,
            textured_leaf_mesh,
            leaf_card_mesh,
            bud_mesh,
            local_focus,
            camera_direction,
        ) = oak_review_terminal_specimen(tree, canopy_bps);
        let bark_material = bark_materials.add(oak_bark_material(
            procedural_assets,
            images.add(terrain_heightmap_image(terrain)),
            Vec2::new(terrain.minimum_height(), terrain.maximum_height()),
            terrain,
        ));
        let leaf_material = leaf_card_materials.add(oak_leaf_material(procedural_assets));
        let bud_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.36, 0.27, 0.1),
            perceptual_roughness: 0.92,
            ..default()
        });
        output.entities.push(
            commands
                .spawn((
                    Name::new("Isolated production oak terminal shoot"),
                    TreeReviewSpecimen,
                    Mesh3d(meshes.add(branch_mesh)),
                    MeshMaterial3d(bark_material),
                    Visibility::Hidden,
                    Transform::from_translation(specimen_origin),
                ))
                .id(),
        );
        output.entities.push(
            commands
                .spawn((
                    Name::new("Isolated production oak terminal bud"),
                    TreeReviewSpecimen,
                    Mesh3d(meshes.add(bud_mesh)),
                    MeshMaterial3d(bud_material),
                    Visibility::Hidden,
                    Transform::from_translation(specimen_origin),
                ))
                .id(),
        );
        self.leaves(
            Leaves {
                textured_leaf_mesh,
                leaf_card_mesh,
                leaf_material,
                specimen_origin,
                local_focus,
                camera_direction,
                tree_review_azimuth_degrees,
            },
            &mut output,
        );
        output
    }

    fn backdrop(&mut self, tree: Vec3) {
        let Self {
            commands,
            meshes,
            materials,
            ..
        } = self;
        commands.spawn((
            Name::new("Neutral tree review backdrop"),
            TreeReviewBackdrop,
            Mesh3d(meshes.add(Cuboid::new(40.0, 26.0, 0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.78, 0.82, 0.84),
                unlit: true,
                ..default()
            })),
            Visibility::Hidden,
            Transform {
                translation: tree + Vec3::new(-8.0, 3.0, -8.0),
                rotation: Quat::from_rotation_y(core::f32::consts::FRAC_PI_4),
                ..default()
            },
        ));
    }

    fn leaves(&mut self, leaves: Leaves, output: &mut ReviewSpecimens) {
        let Leaves {
            textured_leaf_mesh,
            leaf_card_mesh,
            leaf_material,
            specimen_origin,
            local_focus,
            camera_direction,
            tree_review_azimuth_degrees,
        } = leaves;
        let Self {
            commands,
            meshes,
            materials,
            ..
        } = self;
        let textured_entity = commands
            .spawn((
                Name::new("Isolated cambered textured oak terminal leaves"),
                TreeReviewSpecimen,
                TreeLeafRepresentation::TexturedMesh,
                Mesh3d(meshes.add(textured_leaf_mesh)),
                MeshMaterial3d(leaf_material.clone()),
                Visibility::Hidden,
                Transform::from_translation(specimen_origin),
            ))
            .id();
        output
            .leaves
            .push((textured_entity, TreeLeafRepresentation::TexturedMesh));
        let card_entity = commands
            .spawn((
                Name::new("Isolated flat-card oak terminal leaves"),
                TreeReviewSpecimen,
                TreeLeafRepresentation::AlphaCard,
                Mesh3d(meshes.add(leaf_card_mesh)),
                MeshMaterial3d(leaf_material),
                Visibility::Hidden,
                Transform::from_translation(specimen_origin),
            ))
            .id();
        output
            .leaves
            .push((card_entity, TreeLeafRepresentation::AlphaCard));
        output.focus = Some(specimen_origin + local_focus);
        let review_rotation = Quat::from_rotation_y(tree_review_azimuth_degrees.to_radians());
        output.camera =
            Some(specimen_origin + local_focus + review_rotation * camera_direction * 0.68);
        output.entities.push(
            commands
                .spawn((
                    Name::new("Ten centimetre tree review scale bar"),
                    TreeReviewSpecimen,
                    Mesh3d(meshes.add(Cuboid::new(0.1, 0.012, 0.012))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.12, 0.12, 0.12),
                        unlit: true,
                        ..default()
                    })),
                    Visibility::Hidden,
                    Transform::from_translation(
                        specimen_origin + local_focus + Vec3::new(-0.12, -0.13, 0.0),
                    ),
                ))
                .id(),
        );
    }
}

pub(super) fn enabled(profile: &str, content: &TreeContentRequest) -> bool {
    #[cfg(feature = "grown-review")]
    if matches!(
        profile,
        grown_capture::PROFILE | grown_capture::GENERATED_PROFILE
    ) {
        return false;
    }
    #[cfg(not(feature = "grown-review"))]
    let _ = profile;
    matches!(content, TreeContentRequest::Recipe)
}
