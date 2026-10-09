use super::*;
use crate::spacetimedb::DestinationKnowledgeStage;
use adventuresim_world_schema::coordinates::Wgs84CoordinateE7;
use adventuresim_world_schema::source_package::SourcePackageDigest;
fn settlement(id: &str, name: &str, longitude: f64, latitude: f64) -> SettlementView {
    SettlementView {
        id: id.into(),
        name: name.into(),
        longitude,
        latitude,
        population_level: 4,
        population_estimate: 1_000,
        category: crate::spacetimedb::SettlementCategory::Town,
        languages: adventuresim_world_schema::SettlementLanguageProfile {
            east_central_bp: 10_000,
            west_central_bp: 0,
            low_bp: 0,
            yiddish_incidence_bp: 75,
        },
        industries: adventuresim_world_schema::InferredIndustryProfile::new(vec![
            adventuresim_world_schema::IndustryEvidence::Fallback(
                adventuresim_world_schema::FallbackIndustry::CroplandGrain,
            ),
        ])
        .unwrap(),
        economy: adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder(),
        religious_status: adventuresim_world_schema::SettlementReligiousStatus::Established {
            religion: adventuresim_world_schema::OfficialReligion::RomanCatholic,
        },
        scene_key: "hills".into(),
        religion_id: "western_church".into(),
        currency_id: "coin".into(),
        source_node_id: Some(1),
    }
}

fn case_site(id: &str, title: &str, longitude: f64, latitude: f64) -> BackendCaseSitePin {
    let coordinate =
        Wgs84CoordinateE7::from_longitude_latitude_degrees(longitude, latitude).unwrap();
    BackendCaseSitePin {
        raiding_allowed: false,
        owner_character_id: 7,
        case_id: "quest-1".into(),
        case_site_id: adventuresim_stdb_client::CaseSiteId { value: id.into() },
        origin_settlement_id: "origin".into(),
        name: title.into(),
        description: "A camp in the woods.".into(),
        scene_key: "forest".into(),
        longitude_e_7: coordinate.longitude().get(),
        latitude_e_7: coordinate.latitude().get(),
        coordinates_are_geographic: true,
        distance_m: 8_000,
        knowledge_stage: DestinationKnowledgeStage::ExactBelieved,
        tracked: true,
        display_title: title.into(),
        generated_case: false,
        case_resolved: false,
        combat_available: false,
        opposition_count: None,
        opposition_combat_power: None,
    }
}

fn mapped_html(
    settlements: &[SettlementView],
    sites: &[BackendCaseSitePin],
    selected: Option<&str>,
    route: Option<&RoutePlan>,
) -> String {
    let connected = BTreeSet::from(["near"]);
    let presentation = MapPresentation::from_native_source(
        SourcePackageDigest::from_hex(&"a".repeat(64)).unwrap(),
        [-10.0, 40.0, 30.0, 70.0],
        MapLocations {
            settlements,
            case_sites: sites,
            current: &settlements[0],
            connected: &connected,
            selected,
            route,
        },
    )
    .unwrap();
    let json = serde_json::to_string(&presentation.input).unwrap();
    render(
        &presentation,
        &json,
        &settlements[0].name,
        "/locations/settlement/origin",
    )
    .into_string()
}

#[test]
fn environment_map_has_canonical_pin_links_and_accessible_controls() {
    let mut source_less = settlement("demo", "Demo", 0.0, 0.0);
    source_less.source_node_id = None;
    let settlements = [
        settlement("origin", "Origin", 10.0, 53.0),
        settlement("near", "Nearby", 11.0, 53.2),
        settlement("far", "Far away", 20.0, 60.0),
        settlement("outside", "Outside package", 40.0, 80.0),
        source_less,
    ];
    let markup = mapped_html(&settlements, &[], Some("near"), None);
    assert!(markup.contains("data-regional-map-input"));
    assert!(markup.contains("data-map-window"));
    for action in [
        "zoom-in",
        "zoom-out",
        "rotate-left",
        "rotate-right",
        "reset",
        "frame-route",
        "retry",
    ] {
        assert!(markup.contains(&format!("data-map-action=\"{action}\"")));
    }
    assert!(markup.contains("data-strategic-tooltip=\"Zoom in\""));
    assert!(markup.contains("tabindex=\"0\""));
    assert!(markup.contains("?destination=near"));
    assert!(markup.contains("Nearby,"));
    assert!(markup.contains("Far away,"));
    assert!(markup.contains("no direct route"));
    assert!(!markup.contains("Outside package"));
    assert!(!markup.contains("?destination=demo"));
    assert!(markup.contains("aria-current=\"true\""));
    assert!(markup.contains("Selected"));
    assert!(markup.contains("data-map-place=\"place:v1:settlement:"));
    assert!(markup.contains("\"kind\":\"estimate\""));
    assert!(!markup.contains("map-tile"));
    assert!(!markup.contains("<canvas"));
    assert!(markup.len() < 50_000);
    assert!(markup.contains("href=\"/map/data-license\""));
    assert!(markup.contains("rel=\"license\""));
    assert!(DATA_LICENSE.contains("Creative Commons Attribution-ShareAlike 4.0"));
    assert!(DATA_LICENSE.contains("The organisations in charge of the Copernicus programme"));
}

#[test]
fn selected_case_site_has_a_pin_and_computed_terrain_route() {
    let mut site = case_site("site:opaque-hash", "The abandoned croft", 11.0, 53.2);
    site.name = "Undiscovered private identity".into();
    let route = RoutePlan {
        points: vec![
            adventuresim_terrain::RoutePoint {
                latitude: 53.0,
                longitude: 10.0,
            },
            adventuresim_terrain::RoutePoint {
                latitude: 53.1,
                longitude: 10.4,
            },
            adventuresim_terrain::RoutePoint {
                latitude: 53.2,
                longitude: 11.0,
            },
        ],
        spans: Vec::new(),
        distance_m: 74_500,
        minutes: 1_100,
    };
    let settlements = [settlement("origin", "Origin", 10.0, 53.0)];
    let markup = mapped_html(
        &settlements,
        std::slice::from_ref(&site),
        Some("site:opaque-hash"),
        Some(&route),
    );
    assert!(markup.contains("?destination=site%3Aopaque-hash"));
    assert!(markup.contains("Known case site: The abandoned croft"));
    assert!(!markup.contains("Undiscovered private identity"));
    assert!(markup.contains("\"kind\":\"computed\""));
    assert!(markup.contains("53100000"));
    assert!(markup.contains("aria-current=\"true\""));
    for stage in [
        DestinationKnowledgeStage::Unknown,
        DestinationKnowledgeStage::Textual,
        DestinationKnowledgeStage::Landmark,
        DestinationKnowledgeStage::ApproximateArea,
        DestinationKnowledgeStage::RouteSegment,
    ] {
        site.knowledge_stage = stage;
        let markup = mapped_html(
            &settlements,
            std::slice::from_ref(&site),
            Some("site:opaque-hash"),
            Some(&route),
        );
        assert!(!markup.contains("The abandoned croft"));
        assert!(!markup.contains("place:v1:case-site:"));
        assert!(markup.contains("\"route\":null"));
    }
    site.knowledge_stage = DestinationKnowledgeStage::Visited;
    let markup = mapped_html(
        &settlements,
        std::slice::from_ref(&site),
        Some("site:opaque-hash"),
        Some(&route),
    );
    assert!(markup.contains("Visited case site"));
    site.coordinates_are_geographic = false;
    assert!(
        !mapped_html(&settlements, &[site], Some("site:opaque-hash"), None)
            .contains("The abandoned croft")
    );
}

#[test]
fn source_less_origin_has_explicit_unavailable_state() {
    let mut origin = settlement("demo", "Demo settlement", 0.0, 0.0);
    origin.source_node_id = None;
    assert!(!has_geographic_source(&origin));
    let markup = strategic_map_unavailable(&origin.name).into_string();
    assert!(markup.contains("Map unavailable"));
    assert!(markup.contains("Demo settlement"));
    assert!(!markup.contains("data-regional-map"));
}

#[test]
fn missing_runtime_source_keeps_the_destination_workflow_accessible() {
    let markup = strategic_map_bundle_unavailable().into_string();
    assert!(markup.contains("Choose a destination"));
    assert!(!markup.contains("data-regional-map"));
}
