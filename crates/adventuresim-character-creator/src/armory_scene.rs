//! The armory's preview meshes, their placement on the wall, and the camera framing them.
use super::*;

/// Which exhibit placement a preview mesh shows.
#[derive(Component, Clone, Copy)]
pub(crate) struct ArmoryMesh {
    pub(crate) exhibit: usize,
    placement: usize,
}

#[derive(Component)]
pub(crate) struct ArmoryBody;

/// Soft front light so pieces at the wall's edges are not left in the dark.
#[derive(Component)]
pub(crate) struct ArmoryLight;

pub(crate) fn setup(mut commands: Commands) {
    commands.spawn((
        ArmoryLight,
        DirectionalLight {
            color: Color::srgb(0.86, 0.9, 1.0),
            illuminance: 1_800.0,
            ..default()
        },
        Transform::from_xyz(0.6, 1.5, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        Visibility::Hidden,
    ));
}

pub(super) fn spawn_exhibit(
    scene: &mut PreviewScene,
    catalog: &EquipmentCatalog,
    index: usize,
    exhibit: &Exhibit,
) -> Result<()> {
    let design = catalog.design(&exhibit.item_id);
    let material = scene
        .equipment_maps
        .material(
            &mut scene.images,
            catalog.material(&exhibit.item_id)?,
            design.as_ref().and_then(ItemDesign::recipe),
            None,
        )
        .map_err(anyhow::Error::msg)?;
    for (placement, fitted) in exhibit.fitted.iter().enumerate() {
        let Ok(armor) = fitted else { continue };
        preview::spawn_armor(
            &mut scene.commands,
            &mut scene.meshes,
            &mut scene.materials,
            armor,
            format!(
                "armory {}--{}",
                exhibit.item_id, exhibit.placements[placement]
            ),
            preview::ArmorShading {
                plate: material.clone(),
                trim: None,
            },
            (
                ArmoryMesh {
                    exhibit: index,
                    placement,
                },
                Transform::default(),
                Visibility::Hidden,
            ),
        )?;
    }
    Ok(())
}

pub(super) fn spawn_body(
    scene: &mut PreviewScene,
    model: &BodyModel,
    generated: &GeneratedCharacter,
) -> (Handle<StandardMaterial>, bool) {
    let mesh = studio_generation::visible_body_mesh(generated, &model.mhr.character.mesh.faces);
    let material = scene.materials.add(StandardMaterial {
        base_color: Color::srgb(0.64, 0.39, 0.30),
        perceptual_roughness: 0.52,
        reflectance: 0.46,
        ..default()
    });
    scene.commands.spawn((
        ArmoryBody,
        Name::new("armory body"),
        Mesh3d(scene.meshes.add(mesh)),
        MeshMaterial3d(material.clone()),
        Visibility::Hidden,
    ));
    (material, false)
}

/// Show what the armory view asks for, hang pieces on the wall, and hide the
/// dressed character while the armory is open.
#[expect(
    clippy::type_complexity,
    reason = "the three preview entity kinds are disjoint queries"
)]
pub(crate) fn display(
    mut armory: ResMut<Armory>,
    studio: Res<Studio>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut character: Query<
        &mut Visibility,
        (
            With<CharacterMesh>,
            Without<ArmoryMesh>,
            Without<ArmoryBody>,
        ),
    >,
    mut pieces: Query<(&ArmoryMesh, &mut Transform, &mut Visibility), Without<ArmoryBody>>,
    mut bodies: Query<&mut Visibility, (With<ArmoryBody>, Without<CharacterMesh>)>,
    mut lights: Query<
        &mut Visibility,
        (
            With<ArmoryLight>,
            Without<CharacterMesh>,
            Without<ArmoryMesh>,
            Without<ArmoryBody>,
        ),
    >,
) {
    let active = studio.tab == studio_ui::StudioTab::Armory;
    let show = |visible: bool| {
        if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    for mut visibility in &mut character {
        visibility.set_if_neq(show(!active));
    }
    let on_body = armory.view == ArmoryView::OnBody && armory.selected.is_some();
    for mut visibility in &mut lights {
        visibility.set_if_neq(show(active));
    }
    for mut visibility in &mut bodies {
        visibility.set_if_neq(show(active && on_body));
    }
    if !active {
        return;
    }
    if armory.arranged != Some(armory.both_sides) {
        armory.arrange();
        armory.arranged = Some(armory.both_sides);
    }
    for (mesh, mut transform, mut visibility) in &mut pieces {
        let selected = armory.selected == Some(mesh.exhibit);
        let exhibit = &armory.exhibits[mesh.exhibit];
        let visible = if on_body {
            selected
        } else {
            (!armory.solo || selected || armory.selected.is_none())
                && mesh.placement < exhibit.shown(armory.both_sides)
        };
        visibility.set_if_neq(show(visible));
        let translation = if on_body { Vec3::ZERO } else { exhibit.offset };
        if transform.translation != translation {
            transform.translation = translation;
        }
    }
    let ghost = armory.ghost_body;
    if let Some((handle, applied)) = &mut armory.body_material
        && *applied != ghost
        && let Some(mut material) = materials.get_mut(&*handle)
    {
        material
            .base_color
            .set_alpha(if ghost { 0.28 } else { 1.0 });
        material.alpha_mode = if ghost {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        };
        *applied = ghost;
    }
}

/// Glide the camera to frame the wall or the selected piece.
pub(crate) fn frame_camera(
    mut armory: ResMut<Armory>,
    studio: Res<Studio>,
    panel: Res<CreatorPanelRight>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut camera: Query<&mut OrbitCamera>,
) {
    // The panel's width is only known once it has been drawn.
    if studio.tab != studio_ui::StudioTab::Armory || !armory.ready() || panel.0 <= 0.0 {
        return;
    }
    let (Some(frame), Ok(window), Ok(mut orbit)) =
        (armory.frame, windows.single(), camera.single_mut())
    else {
        return;
    };
    // Wait for the pieces to be fitted before framing them.
    let Some((lo, hi)) = armory.target(frame) else {
        return;
    };
    armory.frame = None;
    let whole_wall = frame == Frame::Wall || armory.selected.is_none();
    let angles = whole_wall.then_some((0.0, 0.0));
    let yaw = angles.map_or(orbit.yaw, |(yaw, _)| yaw);
    let (focus, radius) = framing(window, panel.0, yaw, (lo + hi) * 0.5, hi - lo);
    orbit.goal = Some(OrbitGoal {
        focus,
        radius,
        angles,
    });
}
