//! Scene-file admission must compare weather temperatures without overflow.
use std::{
    fs,
    io::{Seek, Write},
    path::PathBuf,
    time::SystemTime,
};

use adventuresim_tactical_core::scene_input::{
    SceneInputError, SceneValidationError, TacticalSceneInput,
};

struct TemporaryScene(PathBuf);

impl Drop for TemporaryScene {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("scene.json"));
        let _ = fs::remove_dir(&self.0);
    }
}

#[test]
fn scene_weather_temperature_comparison_handles_integer_boundaries() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/tactical-scenes/flat-dry-grassland.json");
    assert!(TacticalSceneInput::load(&fixture).is_ok());
    let original = fs::read_to_string(fixture).unwrap();
    let temperature_field = "\"temperature_deci_c\":120,";
    let dew_point_field = "\"dew_point_deci_c\":36,";
    assert_eq!(original.matches(temperature_field).count(), 1);
    assert_eq!(original.matches(dew_point_field).count(), 1);

    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "fabelgeist-weather-admission-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&path).unwrap();
    let scene = TemporaryScene(path);
    let scene_path = scene.0.join("scene.json");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&scene_path)
        .unwrap();

    let cases = [
        (120, 125, true),
        (120, 126, false),
        (i32::MAX - 5, i32::MAX, true),
        (i32::MAX - 6, i32::MAX, false),
        (i32::MAX - 4, i32::MAX, true),
        (i32::MAX, i32::MAX, true),
        (i32::MIN, i32::MIN, true),
        (i32::MIN, i32::MIN + 5, true),
        (i32::MIN, i32::MIN + 6, false),
    ];
    for (temperature, dew_point, accepted) in cases {
        let input = original
            .replace(
                temperature_field,
                &format!("\"temperature_deci_c\":{temperature},"),
            )
            .replace(
                dew_point_field,
                &format!("\"dew_point_deci_c\":{dew_point},"),
            );
        file.set_len(0).unwrap();
        file.rewind().unwrap();
        file.write_all(input.as_bytes()).unwrap();
        let result = TacticalSceneInput::load(&scene_path);
        if accepted {
            assert!(result.is_ok(), "{temperature}, {dew_point}: {result:?}");
        } else {
            assert!(
                matches!(
                    &result,
                    Err(SceneInputError::Validation(SceneValidationError::Weather))
                ),
                "{temperature}, {dew_point}: {result:?}"
            );
        }
    }
}
