//! Behavior of the HYDE source stage with distinct synthetic role inputs.

use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use adventuresim_world_schema::{ElevationMeters, SpatialGridSpec, calendar::CalendarYear};
use zip::ZipArchive;

use crate::draft::{ElevatedSettlementDraft, LandUseEvidence, SettlementDraft, WorldDraft};

struct FixtureDirectory {
    path: PathBuf,
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn hyde_enrichment_assigns_distinct_areas_to_the_settlement_profile() {
    let directory = role_fixture();
    let draft = WorldDraft {
        // The expected fractions use the fixture's 1544 interpolation.
        year: CalendarYear::new(1544).unwrap(),
        spatial_grid: SpatialGridSpec::default(),
        sources: Vec::new(),
        road_types: Vec::new(),
        nodes: Vec::new(),
        edges: Vec::new(),
        settlement_aliases: Vec::new(),
        settlement_descriptions: Vec::new(),
        settlements: vec![ElevatedSettlementDraft {
            settlement: SettlementDraft {
                id: "hyde-fixture".into(),
                source_node_id: 1,
                name: "HYDE fixture".into(),
                longitude: 0.04,
                latitude: -0.04,
                population_level: 1,
                population_estimate: 0,
                scene_key: "fixture".into(),
                sources: String::new(),
            },
            elevation: ElevationMeters::new(0).unwrap(),
        }],
        report: Default::default(),
    };

    let result = super::enrich(draft, &directory.path).unwrap();
    assert_eq!(result.settlements.len(), 1);
    let settlement = &result.settlements[0];
    assert_eq!(settlement.land_use.cropland().basis_points(), 1_880);
    assert_eq!(settlement.land_use.grazing().basis_points(), 2_880);
    assert_eq!(settlement.land_use.built_up().basis_points(), 276);
    assert_eq!(settlement.land_use.natural().basis_points(), 4_964);
    assert_eq!(
        settlement.evidence,
        LandUseEvidence::Hyde35Sampled { normalized: false }
    );
    assert_eq!(result.report.land_use_rasters_read, 4);
    assert_eq!(result.report.land_use_samples, 1);
    assert_eq!(result.report.land_use_fallback_samples, 0);
    assert_eq!(result.report.land_use_normalized_samples, 0);
    assert!(
        settlement
            .elevated
            .settlement
            .sources
            .contains("regional reconstruction")
    );
}

fn role_fixture() -> FixtureDirectory {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "hyde-settlement-stage-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&directory).unwrap();
    let directory = FixtureDirectory { path: directory };
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hyde-time/fixtures.zip");
    let mut archive = ZipArchive::new(File::open(path).unwrap()).unwrap();
    for (member, filename) in [
        ("role_ascending/cropland.nc", "cropland.nc"),
        ("role_ascending/grazing_land.nc", "grazing_land.nc"),
        ("role_ascending/urban_area.nc", "urban_area.nc"),
        ("area/general_files.zip", "general_files.zip"),
    ] {
        let mut input = archive.by_name(member).unwrap();
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.path.join(filename))
            .unwrap();
        std::io::copy(&mut input, &mut output).unwrap();
    }
    directory
}
