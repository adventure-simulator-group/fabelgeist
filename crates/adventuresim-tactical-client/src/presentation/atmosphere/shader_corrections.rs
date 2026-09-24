//! Correct Bevy 0.19.1 atmosphere lookup coordinates and per-sample transport.
//!
//! Replace the canonical assets, retaining their handles and import identities.
//! This corrects both visible sky and IBL consumers of the same library.
use super::*;
use bevy::shader::{Shader, Source};

const SOURCES: [(&str, &str); 3] = [
    (
        "embedded://bevy_pbr/atmosphere/functions.wgsl",
        include_str!("../../../../../assets/shaders/tactical_atmosphere_functions.wgsl"),
    ),
    (
        "embedded://bevy_pbr/atmosphere/sky_view_lut.wgsl",
        include_str!("../../../../../assets/shaders/tactical_atmosphere_sky_view_lut.wgsl"),
    ),
    (
        "embedded://bevy_pbr/atmosphere/render_sky.wgsl",
        include_str!("../../../../../assets/shaders/tactical_atmosphere_render_sky.wgsl"),
    ),
];

#[derive(Resource, Default)]
pub(in crate::presentation) struct AtmosphereShaderCorrections {
    shaders: [Handle<Shader>; 3],
    pub(super) installed: bool,
    pub(super) revision: u64,
}

impl AtmosphereShaderCorrections {
    pub(super) fn install(app: &mut App) {
        if app.get_sub_app(RenderApp).is_none() {
            return;
        }
        let server = app.world().resource::<AssetServer>();
        let shaders = SOURCES.map(|(path, _)| server.load(path));
        app.insert_resource(Self {
            shaders,
            installed: false,
            revision: 0,
        })
        .add_systems(
            PostUpdate,
            correct_loaded_shaders
                .before(cache_initialized_atmosphere)
                .before(bevy::asset::AssetEventSystems),
        );
    }

    fn apply(&mut self, assets: &mut Assets<Shader>) {
        // Wait for all asynchronous loads, so an outstanding upstream load
        // cannot overwrite a correction or expose a half-corrected mapping.
        if self.shaders.iter().any(|handle| !assets.contains(handle)) {
            self.installed = false;
            return;
        }
        let mut replaced = false;
        for (handle, (_, source)) in self.shaders.iter().zip(SOURCES) {
            if assets.get(handle).unwrap().source.as_str() != source {
                // Imports are unchanged. Preserve loader-owned dependencies,
                // shader definitions, validation policy and import identity.
                assets.get_mut(handle).unwrap().source = Source::Wgsl(source.into());
                replaced = true;
            }
        }
        if replaced {
            self.revision += 1;
        }
        self.installed = true;
    }
}

fn correct_loaded_shaders(
    mut corrections: ResMut<AtmosphereShaderCorrections>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    corrections.apply(&mut shaders);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correction_waits_for_all_loads_and_keeps_consumer_handles() {
        let mut assets = Assets::<Shader>::default();
        let library = assets.add(Shader::from_wgsl(
            "#define_import_path bevy_pbr::atmosphere::functions\n// original library",
            SOURCES[0].0,
        ));
        let kernel = assets.reserve_handle();
        let render = assets.add(Shader::from_wgsl("// original render", SOURCES[2].0));
        let mut corrections = AtmosphereShaderCorrections {
            shaders: [library.clone(), kernel.clone(), render],
            installed: false,
            revision: 0,
        };
        corrections.apply(&mut assets);
        assert!(!corrections.installed);
        assets
            .insert(
                kernel.id(),
                Shader::from_wgsl("// original kernel", SOURCES[1].0),
            )
            .unwrap();
        corrections.apply(&mut assets);
        assert!(corrections.installed);
        assert_eq!(assets.len(), 3);
        assert!(assets.get(&library).is_some());
        assert!(assets.get(&kernel).is_some());
        assert_eq!(
            &assets.get(&library).unwrap().import_path,
            &bevy::shader::ShaderImport::Custom("bevy_pbr::atmosphere::functions".into())
        );
        assert_eq!(corrections.revision, 1);
        corrections.apply(&mut assets);
        assert_eq!(corrections.revision, 1);
        assets.get_mut(&kernel).unwrap().source = Source::Wgsl("// reloaded kernel".into());
        corrections.apply(&mut assets);
        assert_eq!(corrections.revision, 2);
        assert_eq!(assets.get(&kernel).unwrap().source.as_str(), SOURCES[1].1);
    }
}
