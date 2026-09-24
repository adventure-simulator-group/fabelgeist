//! Offline captures, comparisons and GLB reloads use the studio renderer.
use crate::app::Studio;
use adventuresim_heraldry::{
    bake::{Baked, Resolution},
    document::{Document, Tincture},
};
use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    camera::RenderTarget,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::ExitCondition,
    winit::WinitPlugin,
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
const CAPTURE_WIDTH: u32 = 1600;
const CAPTURE_HEIGHT: u32 = 1000;
const WARMUP_FRAMES: u32 = 90;
const CAPTURE_DEADLINE: u32 = 1800;
#[derive(Resource)]
struct Capture {
    target: Handle<Image>,
    output: PathBuf,
    frame: u32,
    glb: Option<Handle<WorldAsset>>,
    loaded: bool,
    saved: Arc<AtomicBool>,
}
pub fn capture(
    document: Document,
    comparison: Option<Document>,
    resolution: Resolution,
    output: &Path,
    reload: Option<&Path>,
    mixer: Option<Tincture>,
) -> Result<(), String> {
    document.validate().map_err(|e| e.to_string())?;
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let reload = reload
        .map(std::fs::canonicalize)
        .transpose()
        .map_err(|e| e.to_string())?;
    let root = reload
        .as_ref()
        .and_then(|p| p.parent())
        .map(Path::to_owned)
        .unwrap_or(std::env::current_dir().map_err(|e| e.to_string())?);
    let mut studio = Studio::new(document);
    studio.resolution = resolution;
    studio.current = Some(Arc::new(
        Baked::generate(&studio.document, resolution).map_err(|e| e.to_string())?,
    ));
    if let Some(d) = comparison {
        let bake = Baked::generate(&d, resolution).map_err(|e| e.to_string())?;
        studio.pinned = Some((d, Arc::new(bake)));
    }
    if let Some(tincture) = mixer {
        studio
            .mixer
            .open(tincture, studio.document.surface.palette[tincture]);
    }
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .set(AssetPlugin {
                file_path: root.to_string_lossy().into_owned(),
                ..default()
            })
            .disable::<WinitPlugin>(),
    )
    .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
        1.0 / 30.0,
    )))
    .insert_resource(studio);
    if mixer.is_some() {
        app.add_plugins(bevy_egui::EguiPlugin::default())
            .add_systems(bevy_egui::EguiPrimaryContextPass, crate::ui::draw);
    }
    crate::scene::setup(app.world_mut());
    let target = target(&mut app)?;
    for camera in app
        .world_mut()
        .query_filtered::<Entity, With<bevy_egui::PrimaryEguiContext>>()
        .iter(app.world())
        .collect::<Vec<_>>()
    {
        app.world_mut()
            .entity_mut(camera)
            .insert(RenderTarget::Image(target.clone().into()));
    }
    let glb = if let Some(path) = &reload {
        let scene = app.world().resource::<AssetServer>().load(
            GltfAssetLabel::Scene(0)
                .from_asset(path.file_name().unwrap().to_string_lossy().into_owned()),
        );
        app.world_mut().resource_mut::<Studio>().scene_dirty = false;
        app.world_mut().spawn(WorldAssetRoot(scene.clone()));
        Some(scene)
    } else {
        None
    };
    let saved = Arc::new(AtomicBool::new(false));
    app.insert_resource(Capture {
        target,
        output: output.to_owned(),
        frame: 0,
        glb,
        loaded: reload.is_none(),
        saved: saved.clone(),
    })
    .add_systems(Update, (crate::scene::update, screenshot).chain())
    .run();
    if !saved.load(Ordering::Relaxed) {
        return Err("Renderer did not produce the requested capture".into());
    }
    Ok(())
}
fn screenshot(
    mut commands: Commands,
    mut state: ResMut<Capture>,
    server: Res<AssetServer>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frame += 1;
    if state
        .glb
        .as_ref()
        .is_some_and(|h| matches!(server.load_state(h.id()), bevy::asset::LoadState::Failed(_)))
    {
        exit.write(AppExit::error());
        return;
    }
    if !state.loaded
        && state
            .glb
            .as_ref()
            .is_some_and(|h| server.is_loaded_with_dependencies(h.id()))
    {
        state.loaded = true;
        state.frame = 0;
    }
    if state.frame == CAPTURE_DEADLINE {
        error!("Capture exceeded its render deadline");
        exit.write(AppExit::error());
    }
    if state.loaded && state.frame == WARMUP_FRAMES {
        let output = state.output.clone();
        let saved = state.saved.clone();
        commands
            .spawn(Screenshot::image(state.target.clone()))
            .observe(
                move |capture: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                    let result = capture
                        .image
                        .clone()
                        .try_into_dynamic()
                        .map_err(|e| e.to_string())
                        .and_then(|image| image.save(&output).map_err(|e| e.to_string()));
                    match result {
                        Ok(()) => {
                            saved.store(true, Ordering::Relaxed);
                            exit.write(AppExit::Success);
                        }
                        Err(e) => {
                            error!("Capture failed: {e}");
                            exit.write(AppExit::error());
                        }
                    }
                },
            );
    }
}

fn target(app: &mut App) -> Result<Handle<Image>, String> {
    let target = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            CAPTURE_WIDTH,
            CAPTURE_HEIGHT,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    let camera = app
        .world_mut()
        .query_filtered::<Entity, With<crate::scene::PreviewCamera>>()
        .single(app.world())
        .map_err(|e| e.to_string())?;
    app.world_mut()
        .entity_mut(camera)
        .insert(RenderTarget::Image(target.clone().into()));
    Ok(target)
}
