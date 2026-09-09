use super::*;

pub(super) fn setup_viewer(
    mut commands: Commands,
    sequence: Res<CaptureSequence>,
    proportions: Res<CaptureBodyProportions>,
) {
    let default_player = Player::default();
    let mut generator = TerrainGenerator::new(0xA11C_E5E1);
    generator.period = 200.0;
    let terrain = generator.generate(100, if sequence.uses_flat_grid() { 0 } else { 30 }, 100);
    let spawn_height =
        terrain.height_at(Vec2::ZERO).unwrap_or_default() + CAPTURE_ROOT_GROUND_OFFSET_METRES;
    commands.spawn((
        Name::new(if sequence.uses_flat_grid() {
            "Animation review flat-grid scene"
        } else {
            "Animation review hills scene"
        }),
        SceneId("hills".to_owned()),
        SceneEnvironmentFixture::TemperateHills.snapshot("hills"),
        terrain,
        Transform::default(),
    ));

    let subject = commands
        .spawn((
            Name::new(default_player.name),
            CaptureSubject,
            Player::default(),
            CharacterId(default_tactical_character_id()),
            CharacterLook::default(),
            SkeletonState::default(),
            Transform::from_xyz(0.0, spawn_height, 0.0),
            Collider::cylinder(0.4, 1.9),
            CollisionMargin(0.01),
            tactical_character_controller(),
        ))
        .id();
    if let Some(proportions) = proportions.0 {
        commands.entity(subject).insert(
            crate::animation::skeletal_proportions::CharacterSkeletalProportions(proportions),
        );
    }
    commands.spawn((
        Name::new("Animation review fill light"),
        DirectionalLight {
            illuminance: 35_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-8.0, 12.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        CaptureLabel,
        Text::new("Loading authored animation rig..."),
        TextFont::from_font_size(22.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            ..default()
        },
    ));
}

pub(super) fn write_body_proportions(
    output: &std::path::Path,
    explicit: Option<adventuresim_core::character_proportions::CharacterProportions>,
) {
    let proportions = explicit.unwrap_or_else(|| {
        adventuresim_core::character_proportions::CharacterProportions::from_character_id(
            default_tactical_character_id(),
        )
    });
    fs::write(
        output.join("body-proportions.json"),
        serde_json::to_vec_pretty(&proportions).expect("finite body proportions"),
    )
    .expect("write captured body proportions");
}
