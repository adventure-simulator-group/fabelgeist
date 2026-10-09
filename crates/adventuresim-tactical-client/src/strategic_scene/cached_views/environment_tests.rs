use super::*;
use bevy::{camera::Exposure, core_pipeline::tonemapping::Tonemapping};

#[test]
fn new_and_reused_capture_cameras_inherit_environment_before_the_first_render() {
    let mut app = App::new();
    install(&mut app);
    app.insert_resource(super::super::status::SceneAssetsReady(true))
        .init_resource::<Assets<Image>>();
    app.world_mut().spawn((
        crate::presentation::TacticalGameplayCamera,
        Exposure { ev100: 12.5 },
        Tonemapping::None,
        bevy::pbr::DistanceFog::default(),
        Msaa::Sample4,
        bevy::light::ShadowFilteringMethod::Hardware2x2,
        EnvironmentMapLight {
            intensity: 123.0,
            ..default()
        },
        bevy::pbr::AtmosphereSettings::default(),
    ));
    app.add_systems(Update, |mut commands: Commands, mut cache: CacheWorld| {
        let spec = ViewSpec {
            cache: Some(ViewKey::Street),
            rect: CanvasRect {
                x: 0,
                y: 0,
                width: 64,
                height: 64,
                full_width: 64,
                full_height: 64,
                offset_x: 0.0,
                offset_y: 0.0,
            },
            transform: Transform::IDENTITY,
            projection: Projection::default(),
            layers: RenderLayers::default(),
        };
        cache.sync(
            &mut commands,
            &[spec],
            &RetainedScene::default(),
            None,
            &Window::default(),
        );
    });
    let check = |app: &App| {
        let camera = app.world().resource::<CachedViews>().frames[&ViewKey::Street]
            .camera
            .expect("capture started");
        let view = app.world().entity(camera);
        assert_eq!(view.get::<Msaa>(), Some(&Msaa::Sample4));
        assert_eq!(
            view.get::<bevy::light::ShadowFilteringMethod>(),
            Some(&bevy::light::ShadowFilteringMethod::Hardware2x2)
        );
        assert_eq!(view.get::<Tonemapping>(), Some(&Tonemapping::None));
        assert_eq!(view.get::<Exposure>().unwrap().ev100, 12.5);
        assert_eq!(view.get::<EnvironmentMapLight>().unwrap().intensity, 123.0);
        assert!(view.contains::<bevy::pbr::AtmosphereSettings>());
        camera
    };
    app.update();
    let first = check(&app);
    app.world_mut()
        .resource_mut::<CachedViews>()
        .frames
        .get_mut(&ViewKey::Street)
        .unwrap()
        .remaining = 0;
    app.update();
    app.world_mut()
        .resource_mut::<CachedViews>()
        .frames
        .get_mut(&ViewKey::Street)
        .unwrap()
        .remaining = CAPTURE_SETTLED_FRAMES;
    app.update();
    assert_eq!(
        check(&app),
        first,
        "the bounded camera pool reuses its camera"
    );
}
