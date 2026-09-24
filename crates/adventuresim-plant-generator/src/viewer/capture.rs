use super::*;
use bevy::{
    app::AppExit,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
};
const CAPTURE_WARMUP_SECONDS: f64 = 3.0;

pub(super) fn capture(
    mut commands: Commands,
    options: Res<Options>,
    editor: Res<Editor>,
    mut frame: Local<u32>,
    time: Res<Time<Real>>,
) {
    let Some(output) = options.output.as_ref() else {
        return;
    };
    if time.elapsed_secs_f64() < CAPTURE_WARMUP_SECONDS {
        return;
    }
    *frame += 1;
    if *frame != 60 && *frame != 90 {
        return;
    }
    std::fs::create_dir_all(output).expect("create fresh capture directory");
    let image_index = if *frame == 60 { 0 } else { 1 };
    let path = output.join(format!("{}-{image_index}.png", options.view.slug()));
    let (eye, target) = camera(&options, &editor.parameters);
    let mesh = editor
        .parameters
        .generate(options.seed, editor.lod)
        .expect("validated specimen");
    let manifest = serde_json::json!({
        "preset":match editor.origin {RecipeOrigin::Preset=>Some(editor.parameters.family().name(editor.preset)),RecipeOrigin::Custom=>None},"parameters":editor.parameters,
        "seed":options.seed,"view":options.view,"lod":editor.lod,
        "camera":{"eye":eye.to_array(),"target":target.to_array()},
        "dimensions":[1400,1100],"settled_frames":[60,90],
        "minimum_warmup_seconds":CAPTURE_WARMUP_SECONDS,
        "vertices":mesh.positions.len(),"triangles":mesh.indices.len()/3,
        "renderer":"Bevy StandardMaterial PBR; shared production geometry",
        "scope":"specimen geometry; tactical habitat context reviewed separately"
    });
    std::fs::write(
        output.join(format!("{}.json", options.view.slug())),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .expect("write manifest");
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            save_to_disk(&path)(captured);
            if image_index == 1 {
                exit.write(AppExit::Success);
            }
        },
    );
}
