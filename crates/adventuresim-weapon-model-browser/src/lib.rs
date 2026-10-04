//! Browser transport for the authoritative weapon recipe library.
//!
//! Requests use the library's recipe schema. Geometry, anchors, validation,
//! material masses and physical properties come directly from that library;
//! this boundary owns no construction or physical calculation.

use adventuresim_weapon_model::{
    GENERATOR_VERSION, MELEE_CATALOG_IDS, PRESET_IDS, SCHEMA_VERSION, WeaponDesign,
    WeaponHolderDesign, default_design, derive_material_masses, encode, generate, generate_holder,
    numeric_editor_fields, preset_design,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
enum Request {
    Model {
        recipe: adventuresim_weapon_model::recipe::Recipe,
        detail: adventuresim_weapon_model::Detail,
    },
    Catalog {},
    AuthoringCatalog {},
    MuseumStudies {},
    ValidateModel {
        recipe: adventuresim_weapon_model::recipe::Recipe,
        controls: Vec<serde_json::Value>,
        detail: adventuresim_weapon_model::Detail,
    },
    Compose {
        haft: String,
        head: String,
    },
    CompositionControls {
        recipe: adventuresim_weapon_model::recipe::Recipe,
    },
    Preset {
        id: String,
    },
    GameplayDesign {
        id: String,
    },
    Generate {
        design: WeaponDesign,
    },
    GenerateHolder {
        design: WeaponHolderDesign,
    },
    EditorFields {
        design: WeaponDesign,
    },
}

#[derive(Serialize)]
struct Catalog {
    schema_version: u16,
    generator_version: u16,
    presets: &'static [&'static str],
    gameplay: &'static [&'static str],
}

#[derive(Debug, thiserror::Error)]
enum RequestError {
    #[error("invalid construction: {0}")]
    Construction(String),
    #[error("invalid request: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unknown weapon preset: {0}")]
    UnknownPreset(String),
    #[error("unknown gameplay weapon: {0}")]
    UnknownGameplayDesign(String),
    #[error("invalid weapon: {0}")]
    Generation(#[from] adventuresim_weapon_model::GenerateError),
    #[error("invalid recipe: {0}")]
    Encoding(#[from] adventuresim_weapon_model::CodecError),
    #[error("invalid material calculation: {0:?}")]
    Materials(Vec<adventuresim_weapon_model::ValidationError>),
}

impl Request {
    fn execute(self) -> Result<String, RequestError> {
        let value = match self {
            Self::Model { recipe, detail } => serde_json::to_value(
                adventuresim_weapon_model::generate_model(&recipe, detail)
                    .map_err(RequestError::Construction)?,
            )?,
            Self::ValidateModel {
                recipe,
                controls,
                detail,
            } => {
                adventuresim_weapon_model::authoring::validate_controls(&recipe, &controls)
                    .map_err(RequestError::Construction)?;
                serde_json::to_value(
                    adventuresim_weapon_model::generate_model(&recipe, detail)
                        .map_err(RequestError::Construction)?,
                )?
            }
            Self::MuseumStudies {} => {
                adventuresim_weapon_model::authoring::museum_studies().clone()
            }
            Self::AuthoringCatalog {} => {
                adventuresim_weapon_model::authoring::authoring_catalog().clone()
            }
            Self::Compose { haft, head } => serde_json::to_value(
                adventuresim_weapon_model::authoring::compose_weapon(&haft, &head)
                    .map_err(RequestError::Construction)?,
            )?,
            Self::CompositionControls { recipe } => serde_json::to_value(
                adventuresim_weapon_model::authoring::composition_controls(&recipe)
                    .map_err(RequestError::Construction)?,
            )?,
            Self::Catalog {} => serde_json::to_value(Catalog {
                schema_version: SCHEMA_VERSION,
                generator_version: GENERATOR_VERSION,
                presets: PRESET_IDS,
                gameplay: MELEE_CATALOG_IDS,
            })?,
            Self::Preset { id } => {
                serde_json::to_value(preset_design(&id).ok_or(RequestError::UnknownPreset(id))?)?
            }
            Self::GameplayDesign { id } => serde_json::to_value(
                default_design(&id).ok_or(RequestError::UnknownGameplayDesign(id))?,
            )?,
            Self::Generate { design } => {
                // Encoding validates the exact recipe that the game will store.
                let recipe = encode(&design)?;
                let mesh = generate(&design)?;
                let materials = derive_material_masses(&design).map_err(RequestError::Materials)?;
                serde_json::json!({ "mesh": mesh, "materials": materials, "recipe": recipe })
            }
            Self::GenerateHolder { design } => serde_json::to_value(generate_holder(&design)?)?,
            Self::EditorFields { design } => {
                encode(&design)?;
                serde_json::to_value(numeric_editor_fields(&design))?
            }
        };
        Ok(serde_json::to_string(&value)?)
    }
}

/// Execute one JSON request against the same library used by native gameplay.
///
/// Returns an error for unknown operations, malformed recipes, unknown catalog
/// IDs, and any recipe rejected by the authoritative library. WASM converts an
/// error into a JavaScript exception; native callers receive the same message.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn weapon_model_request(json: &str) -> Result<String, String> {
    let mut deserializer = serde_json::Deserializer::from_str(json);
    let request: Request =
        serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
            request_error_context(
                json,
                format!("invalid request at {}: {}", error.path(), error.inner()),
            )
        })?;
    deserializer.end().map_err(|error| error.to_string())?;
    request.execute().map_err(|error| error.to_string())
}

fn request_error_context(json: &str, request_error: String) -> String {
    // Internally tagged request decoding buffers the payload and can erase a
    // nested field path. Re-read the recipe only to locate its decoding error;
    // no invalid request is executed or repaired through this diagnostic path.
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return request_error;
    };
    let Some(recipe) = value.get("recipe") else {
        return request_error;
    };
    match serde_path_to_error::deserialize::<_, adventuresim_weapon_model::recipe::Recipe>(
        recipe.clone(),
    ) {
        Err(error) => format!("invalid recipe at {}: {}", error.path(), error.inner()),
        Ok(_) => request_error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_weapon_model::{
        decode,
        recipe::{Metres, Shape},
    };

    fn request(value: serde_json::Value) -> Result<serde_json::Value, String> {
        let result = weapon_model_request(&value.to_string())?;
        serde_json::from_str(&result).map_err(|error| error.to_string())
    }

    #[test]
    fn edited_recipe_round_trips_to_gameplay_with_matching_mesh_and_mass() {
        let mut design = default_design("longsword").unwrap();
        let Shape::LoftedBlade(blade) = &mut design
            .recipe
            .components
            .iter_mut()
            .find(|part| matches!(part.shape, Shape::LoftedBlade(_)))
            .unwrap()
            .shape
        else {
            unreachable!()
        };
        blade.length = Metres::new(blade.length.get() + 0.15).unwrap();
        let result =
            request(serde_json::json!({ "operation": "generate", "design": design })).unwrap();
        let bytes: Vec<u8> = serde_json::from_value(result["recipe"].clone()).unwrap();
        let stored = decode(&bytes).unwrap();
        assert_eq!(stored, design);
        let expected = generate(&stored).unwrap();
        let received: adventuresim_weapon_model::GeneratedWeapon =
            serde_json::from_value(result["mesh"].clone()).unwrap();
        assert!(received == expected, "transport changed the generated mesh");
        assert!(result["mesh"]["derived"]["length_m"].as_f64().unwrap() > 1.3);
    }

    #[test]
    fn invalid_and_unknown_requests_fail_without_a_substitute_recipe() {
        for value in [
            serde_json::json!({ "operation": "unknown" }),
            serde_json::json!({ "operation": "preset", "id": "missing" }),
            serde_json::json!({ "operation": "generate", "design": { "catalog_id": "longsword", "components": [] } }),
            serde_json::json!({ "operation": "catalog", "extra": true }),
        ] {
            assert!(request(value.clone()).is_err(), "accepted {value}");
        }
    }

    #[test]
    fn unsupported_nested_recipe_fields_are_rejected() {
        let original = serde_json::to_value(default_design("longsword").unwrap()).unwrap();
        for path in [
            "",
            "/recipe/components/0",
            "/recipe/components/0/attach",
            "/recipe",
        ] {
            let mut design = original.clone();
            design
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unsupported_parameter".into(), serde_json::json!(25));
            let error = request(serde_json::json!({ "operation": "generate", "design": design }))
                .unwrap_err();
            assert!(error.contains("unsupported_parameter"), "{path}: {error}");
        }
    }
}
