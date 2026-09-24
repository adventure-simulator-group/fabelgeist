//! Physical polygon sockets retain their bore and closing seat at every LOD.
use super::longsword_tests::closed;
use super::*;
use serde_json::json;
use std::collections::BTreeSet;

fn fixture(sides: usize) -> serde_json::Value {
    json!({"components":[
        {"id":"tube","kind":"socket","profile":[[0,0.015],[0.12,0.012]],
         "wall":0.003,"facets":sides,"attach":{"to":"weapon.root","at":"base"}},
        {"id":"roof","kind":"socket","profile":[[0,0.012],[0.006,0.012]],
         "facets":sides,"attach":{"to":"tube.top","at":"base"}}
    ]})
}

fn section(solid: &Solid, height: f64, radius: f64) -> BTreeSet<[u64; 2]> {
    solid
        .positions
        .iter()
        .filter(|p| p[1] == height && (p[0].hypot(p[2]) - radius).abs() < 1e-12)
        .map(|p| [p[0].to_bits(), p[2].to_bits()])
        .collect()
}

#[test]
fn faceted_socket_bore_and_roof_share_physical_sections() {
    for sides in [3, 8, 13, 32] {
        let recipe: Recipe = serde_json::from_value(fixture(sides)).unwrap();
        let resolved = placement::resolve(&recipe).unwrap();
        let mut previous = None;
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let tube = shapes::construct(&resolved.components[0], detail).unwrap();
            let roof = shapes::construct(&resolved.components[1], detail).unwrap();
            let outer = section(&tube[0].solid, 0.12, 0.012);
            let inner = section(&tube[0].solid, 0.12, 0.009);
            assert_eq!(outer.len(), sides);
            assert_eq!(inner.len(), sides);
            assert_eq!(outer, section(&roof[0].solid, 0.0, 0.012));
            if let Some((prior_outer, prior_inner)) = previous {
                assert_eq!(outer, prior_outer);
                assert_eq!(inner, prior_inner);
            }
            previous = Some((outer, inner));
            for part in [&tube[0], &roof[0]] {
                closed(&part.solid);
                closed(
                    &part
                        .solid
                        .clone()
                        .transform([23.0, 47.0, 61.0], [19.0, -18.0, 17.0]),
                );
            }
        }
    }
}

#[test]
fn faceted_socket_rejects_conflicting_or_empty_sections() {
    for (key, value) in [
        ("facets", json!(2)),
        ("facets", json!(33)),
        ("segments", json!(8)),
        ("fitShaft", json!(true)),
        ("wall", json!(0.02)),
        (
            "crenellations",
            json!({"count":4,"depth":0.01,"toothFraction":0.5}),
        ),
    ] {
        let mut value_recipe = fixture(8);
        value_recipe["components"][0][key] = value;
        let recipe: Recipe = serde_json::from_value(value_recipe).unwrap();
        assert!(generate_model(&recipe, Detail::High).is_err(), "{key}");
    }
    let mut value = fixture(8);
    value["shaft"] = json!({"length":0.8,"radius":0.009});
    value["components"][0]
        .as_object_mut()
        .unwrap()
        .remove("attach");
    value["components"][0]["mount"] = json!("shaft-top");
    let recipe: Recipe = serde_json::from_value(value).unwrap();
    assert!(generate_model(&recipe, Detail::High).is_err());
}
