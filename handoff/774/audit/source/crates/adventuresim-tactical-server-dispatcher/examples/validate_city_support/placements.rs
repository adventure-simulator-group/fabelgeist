//! Diagnostic typed export retains the scene's exact numeric representations.
use super::*;
use adventuresim_world_schema::settlement_buildings::BusinessKey;
use std::path::Path;

#[derive(serde::Serialize)]
struct BusinessBinding {
    building_id: adventuresim_tactical_core::scene_input::SceneBuildingId,
    key: BusinessKey,
}

#[derive(serde::Serialize)]
struct GroundedPlacements<'a> {
    scope: &'static str,
    buildings: &'a [TacticalBuildingPlacement],
    distant_buildings: &'a [DistantBuildingPlacement],
    compounds: &'a [CityCompound],
    single_properties: &'a [CitySingleProperty],
    gardens: &'a [CityGarden],
    businesses: Vec<BusinessBinding>,
    streets: &'a [CityStreetPatch],
    yards: &'a [CityYardPatch],
    parishes: &'a [CityParish],
}

impl<'a> GroundedPlacements<'a> {
    fn from_layout(layout: &'a CitySceneLayout) -> Self {
        Self {
            scope: "Accepted named floor projection; not a TacticalSceneInput or runtime acceptance",
            buildings: &layout.playable,
            distant_buildings: &layout.distant,
            compounds: &layout.compounds,
            single_properties: &layout.single_properties,
            gardens: &layout.gardens,
            businesses: layout
                .businesses
                .iter()
                .map(|b| BusinessBinding {
                    building_id: b.building_id,
                    key: b.key,
                })
                .collect(),
            streets: &layout.streets,
            yards: &layout.yards,
            parishes: &layout.parishes,
        }
    }
}

pub(super) fn write(
    path: &Path,
    layout: &CitySceneLayout,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&GroundedPlacements::from_layout(layout))?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_retains_direct_typed_numeric_formatting() {
        let value: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/tactical-grounding/goslar-1238.json"
        )))
        .unwrap();
        let front: DistantBuildingPlacement =
            serde_json::from_value(value["front_distant_placement"].clone()).unwrap();
        let rear = serde_json::from_value(value["rear_distant_placement"].clone()).unwrap();
        let layout = CitySceneLayout {
            playable: vec![front.into()],
            distant: vec![rear],
            ..Default::default()
        };
        let encoded = serde_json::to_vec(&GroundedPlacements::from_layout(&layout)).unwrap();
        let projection: Value = serde_json::from_slice(&encoded).unwrap();
        let playable: Value =
            serde_json::from_slice(&serde_json::to_vec(&layout.playable).unwrap()).unwrap();
        let distant: Value =
            serde_json::from_slice(&serde_json::to_vec(&layout.distant).unwrap()).unwrap();
        assert_eq!(projection["buildings"], playable);
        assert_eq!(projection["distant_buildings"], distant);
    }
}
