//! The two authored inputs the server refuses to start without: the tactical
//! scene it simulates and the combat tuning it resolves every attack against.

use std::path::{Path, PathBuf};

use adventuresim_tactical_core::prelude::*;

use crate::bot;

const DEFAULT_SCENE_INPUT: &str = "dense-woodland";
const DEFAULT_COMBAT_CONFIG: &str = "content/tactical/combat.yaml";

pub(crate) fn default_scene_input_path() -> PathBuf {
    bot::resolve_scene_fixture(DEFAULT_SCENE_INPUT).expect("fixture path resolution is infallible")
}

pub(crate) fn default_combat_config_path() -> PathBuf {
    let working_directory_path = PathBuf::from(DEFAULT_COMBAT_CONFIG);
    if working_directory_path.is_file() {
        return working_directory_path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(DEFAULT_COMBAT_CONFIG)
}

pub(crate) fn load_combat_config(path: &Path) -> Result<TacticalCombatConfig, String> {
    const MAX_COMBAT_CONFIG_BYTES: u64 = 64 * 1024;
    let length = std::fs::metadata(path)
        .map_err(|error| format!("could not inspect {}: {error}", path.display()))?
        .len();
    if length == 0 || length > MAX_COMBAT_CONFIG_BYTES {
        return Err("combat config must contain between 1 byte and 64 KiB".into());
    }
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let config: TacticalCombatConfig = serde_saphyr::from_str(&text)
        .map_err(|error| format!("{} is not valid YAML: {error}", path.display()))?;
    config
        .validate()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(config)
}

/// Everything `main` needs loaded and validated before it builds the app.
pub(crate) struct StartupInputs {
    pub(crate) scene: TacticalSceneInput,
    pub(crate) combat: TacticalCombatConfig,
}

impl StartupInputs {
    /// Loads both inputs, falling back to the committed defaults, and installs
    /// the combat runtime snapshot the simulation reads from. The error already
    /// names which input was rejected.
    pub(crate) fn load(scene: Option<&Path>, combat: Option<&Path>) -> Result<Self, String> {
        let scene_path = scene.map_or_else(default_scene_input_path, Path::to_path_buf);
        let scene = TacticalSceneInput::load(&scene_path)
            .map_err(|error| format!("tactical scene input: {error}"))?;
        let combat_path = combat.map_or_else(default_combat_config_path, Path::to_path_buf);
        let combat = load_combat_config(&combat_path)
            .map_err(|error| format!("tactical combat config: {error}"))?;
        combat
            .install_runtime_snapshot()
            .expect("loaded tactical combat config was validated");
        let digest = combat
            .digest()
            .expect("loaded tactical combat config was validated");
        eprintln!(
            "[startup] tactical combat config path={} digest={digest}",
            combat_path.display()
        );
        Ok(Self { scene, combat })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standalone_default_is_the_dense_woodland_fixture() {
        let input = TacticalSceneInput::load(&default_scene_input_path())
            .expect("default tactical scene input should remain valid");

        assert_eq!(input.scene_key, "woodland");
        assert_eq!(
            input.source,
            SceneSource::SyntheticFixture("dense-woodland".into())
        );
    }

    #[test]
    fn committed_combat_config_matches_canonical_defaults() {
        let loaded = load_combat_config(&default_combat_config_path())
            .expect("committed tactical combat config should remain valid");
        assert_eq!(loaded, TacticalCombatConfig::default());
    }

    #[test]
    fn combat_tuning_is_read_from_the_runtime_yaml_file() {
        let canonical = std::fs::read_to_string(default_combat_config_path())
            .expect("committed tactical combat config should be readable");
        let modified = canonical.replacen(
            "armed_attack_energy_transfer: 0.4",
            "armed_attack_energy_transfer: 0.35",
            1,
        );
        assert_ne!(modified, canonical, "test must modify combat resolution");
        let path = std::env::temp_dir().join(format!(
            "fabelgeist-combat-config-runtime-{}.yaml",
            std::process::id()
        ));
        std::fs::write(&path, modified).expect("temporary combat config should be writable");
        let loaded = load_combat_config(&path).expect("modified runtime YAML should load");
        std::fs::remove_file(&path).expect("temporary combat config should be removable");

        assert_eq!(loaded.resolution.armed_attack_energy_transfer, 0.35);
        assert_ne!(loaded, TacticalCombatConfig::default());
    }
}
