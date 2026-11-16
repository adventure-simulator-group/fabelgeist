//! Compiled settlement support shared by physics tests.
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use adventuresim_tactical_core::city_layout::grounding::*;
use adventuresim_tactical_core::city_layout::{
    CitySceneLayout, CompoundGradingPolicy, StreetApronDimensions,
};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

/// Compiles the compound review scene's property support over a sampled source.
pub(crate) fn compound_review_terrain(
    heightmap: Vec<f32>,
) -> (SceneTerrain, Vec<CompoundSupportPlan>) {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../assets/tactical-scenes/compound-review.json"
    ))
    .unwrap();
    let sampled = SceneTerrain::from_heightmap(101, 101, 1.0, heightmap).unwrap();
    let source = sampled.sampled_geographic_surface().unwrap();
    let layout = CitySceneLayout {
        playable: input.buildings.clone(),
        distant: input.distant_buildings.clone(),
        compounds: input.compounds.clone(),
        ..default()
    };
    let policy = CompoundGradingPolicy {
        limits: SupportLimits::new(
            SupportGrade::from_ratio(0.65).unwrap(),
            PositiveLength::from_metres(6.0).unwrap(),
            PositiveLength::from_metres(0.001).unwrap(),
        ),
        stairs: CourtStairLimits::new(
            PositiveLength::from_metres(0.19).unwrap(),
            PositiveLength::from_metres(0.25).unwrap(),
            PositiveLength::from_metres(1.0).unwrap(),
            PositiveLength::from_metres(1.05).unwrap(),
            PositiveLength::from_metres(0.5).unwrap(),
        ),
        embedment: FoundationEmbedment::from_metres(0.2).unwrap(),
        street_apron: StreetApronDimensions::from_metres(Vec2::new(1.0, 4.0)).unwrap(),
    };
    let plans = layout.plan_compound_support(&source, policy).unwrap();
    let terrain = sampled.with_property_surface(
        BoundedSettlementTerrain::compile(
            &plans
                .iter()
                .map(|plan| plan.support_surface().unwrap())
                .collect::<Vec<_>>(),
            &source,
            policy.embedment,
        )
        .unwrap(),
    );
    (terrain, plans)
}
