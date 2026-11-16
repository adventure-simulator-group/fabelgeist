//! Structured scene values use one self-describing binary document inside the
//! ordinary Postcard length frame. Tagged building/support recipes cannot be
//! decoded by Postcard itself; CBOR also retains the packed geometry buffers.

use bevy::prelude::{Component, Result};
use bevy_replicon::{
    bytes::Bytes, postcard_utils, shared::replication::registry::rule_fns::RuleFns,
};
use serde::{Serialize, de::DeserializeOwned};

pub(crate) fn component_rules<Value: Component + Serialize + DeserializeOwned>() -> RuleFns<Value> {
    RuleFns::new(
        |_, value, message| serialize(value, message),
        |context, message| {
            let mut value = deserialize(message)?;
            Value::map_entities(&mut value, context);
            Ok(value)
        },
    )
}

pub(crate) fn serialize<Value: Serialize>(value: &Value, message: &mut Vec<u8>) -> Result {
    let mut document = Vec::new();
    ciborium::into_writer(value, &mut document)?;
    postcard_utils::to_extend_mut(&document, message)?;
    Ok(())
}

pub(crate) fn deserialize<Value: DeserializeOwned>(message: &mut Bytes) -> Result<Value> {
    let document: Vec<u8> = postcard_utils::from_buf(message)?;
    let mut remaining = document.as_slice();
    let value = ciborium::from_reader(&mut remaining)?;
    if !remaining.is_empty() {
        return Err("Scene document contains trailing data".into());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::SceneVistaBundle;
    use adventuresim_tactical_core::prelude::*;

    #[test]
    fn town_initialization_preserves_owned_support_and_vista_through_replication() {
        for name in ["shop-sign-review", "compound-review", "garden-review"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../assets/tactical-scenes/{name}.json"));
            let input = TacticalSceneInput::load(&path).unwrap();
            let generated = input.generate().unwrap();
            assert!(generated.terrain.property_surface().is_some());
            let mut bytes = Vec::new();
            serialize(&generated.terrain, &mut bytes).unwrap();
            let restored: SceneTerrain = deserialize(&mut Bytes::from(bytes)).unwrap();
            assert!(
                restored == generated.terrain,
                "{name}: terrain changed in transit"
            );

            let mut bytes = Vec::new();
            serialize(&generated.ground, &mut bytes).unwrap();
            let restored: SceneGround = deserialize(&mut Bytes::from(bytes)).unwrap();
            assert!(
                restored == generated.ground,
                "{name}: ground changed in transit"
            );
            if !input.streets.is_empty() {
                let mut bytes = Vec::new();
                postcard_utils::to_extend_mut(&generated.ground, &mut bytes).unwrap();
                assert!(bevy_replicon::postcard::from_bytes::<SceneGround>(&bytes).is_err());
            }

            assert_postcard_roundtrip(&input.environment_snapshot(generated.digest));
            for boundary in &generated.boundaries {
                assert_postcard_roundtrip(boundary.scene());
            }
            for garden in &generated.gardens {
                assert_postcard_roundtrip(garden);
            }
            for establishment in &input.establishments {
                assert_postcard_roundtrip(establishment);
            }

            let vista = SceneVistaBundle::try_from(&input).unwrap();
            let mut bytes = Vec::new();
            serialize(&vista, &mut bytes).unwrap();
            let restored: SceneVistaBundle = deserialize(&mut Bytes::from(bytes)).unwrap();
            assert!(
                serde_json::to_value(restored).unwrap() == serde_json::to_value(vista).unwrap(),
                "{name}: vista changed in transit"
            );
        }
    }

    #[test]
    fn geological_recipes_keep_their_surface_structure_through_initial_replication() {
        for name in ["fault-scarp-cliff", "sandstone-alcove"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../assets/tactical-scenes/{name}.json"));
            let input = TacticalSceneInput::load(&path).unwrap();
            let recipe = input.landform.expect("geological fixture has a landform");
            let mut plain = Vec::new();
            postcard_utils::to_extend_mut(&recipe, &mut plain).unwrap();
            assert!(bevy_replicon::postcard::from_bytes::<TerrainLandformRecipe>(&plain).is_err());
            let mut bytes = Vec::new();
            serialize(&recipe, &mut bytes).unwrap();
            let restored: TerrainLandformRecipe = deserialize(&mut Bytes::from(bytes)).unwrap();
            assert_eq!(
                recipe, restored,
                "{name}: surface structure changed in transit"
            );
        }
    }

    #[test]
    fn scene_frames_reject_truncation_and_trailing_documents() {
        let terrain = SceneTerrain::default();
        let mut bytes = Vec::new();
        serialize(&terrain, &mut bytes).unwrap();
        bytes.pop();
        assert!(deserialize::<SceneTerrain>(&mut Bytes::from(bytes)).is_err());

        let mut document = Vec::new();
        ciborium::into_writer(&terrain, &mut document).unwrap();
        document.push(0);
        let mut bytes = Vec::new();
        postcard_utils::to_extend_mut(&document, &mut bytes).unwrap();
        assert!(deserialize::<SceneTerrain>(&mut Bytes::from(bytes)).is_err());
    }

    fn assert_postcard_roundtrip<Value: Serialize + DeserializeOwned + PartialEq>(value: &Value) {
        let mut bytes = Vec::new();
        postcard_utils::to_extend_mut(value, &mut bytes).unwrap();
        let decoded =
            bevy_replicon::postcard::from_bytes::<Value>(&bytes).unwrap_or_else(|error| {
                panic!(
                    "{} cannot use ordinary component replication: {error}",
                    std::any::type_name::<Value>()
                );
            });
        assert!(
            *value == decoded,
            "{} changed in transit",
            std::any::type_name::<Value>()
        );
    }
}
