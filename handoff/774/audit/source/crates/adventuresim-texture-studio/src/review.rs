//! Native artistic review uses the editor's scene, material bindings and portable documents.
use crate::{app::Studio, document::Document};
use adventuresim_procedural_textures::BakedRecipe;
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
    time::Duration,
};

const CAPTURE_WIDTH: u32 = 1600;
const CAPTURE_HEIGHT: u32 = 1000;
const WARMUP_FRAMES: u32 = 120;

#[derive(Resource)]
struct Capture {
    target: Handle<Image>,
    output: PathBuf,
    frame: u32,
}

pub fn capture(document: Document, before: Option<Document>, output: &Path) -> Result<(), String> {
    document.texture.validate().map_err(|e| e.to_string())?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut studio = Studio::new(document);
    studio.current = Some(BakedRecipe::generate(
        studio.document.recipe,
        &studio.document.texture,
    ));
    if let Some(before) = before {
        before.texture.validate().map_err(|e| e.to_string())?;
        studio.pinned = Some((
            BakedRecipe::generate(before.recipe, &before.texture),
            before,
        ));
    }
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>(),
    )
    .add_plugins(adventuresim_procedural_materials::ProceduralMaterialsPlugin)
    .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
        1.0 / 30.0,
    )))
    .insert_resource(studio);
    crate::scene::setup(app.world_mut());
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
        .query_filtered::<Entity, With<Camera3d>>()
        .single(app.world())
        .map_err(|e| e.to_string())?;
    app.world_mut()
        .entity_mut(camera)
        .insert(RenderTarget::Image(target.clone().into()));
    app.insert_resource(Capture {
        target,
        output: output.to_owned(),
        frame: 0,
    })
    .add_systems(Update, (crate::scene::update, screenshot).chain())
    .run();
    if !output.is_file() {
        return Err("Renderer did not produce the requested capture".into());
    }
    Ok(())
}

fn screenshot(mut commands: Commands, mut state: ResMut<Capture>) {
    state.frame += 1;
    if state.frame == WARMUP_FRAMES {
        let output = state.output.clone();
        commands
            .spawn(Screenshot::image(state.target.clone()))
            .observe(
                move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                    let result = captured
                        .image
                        .clone()
                        .try_into_dynamic()
                        .map_err(|e| e.to_string())
                        .and_then(|image| image.save(&output).map_err(|e| e.to_string()));
                    match result {
                        Ok(()) => {
                            exit.write(AppExit::Success);
                        }
                        Err(error) => {
                            error!("Capture failed: {error}");
                            exit.write(AppExit::error());
                        }
                    }
                },
            );
    }
}
