use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use adventuresim_world_import::{Error, WorldBuilder, builder::ViabundusEnrichment};
use adventuresim_world_schema::calendar::CalendarYear;

const INSIDE: &str = "1,null,Lübeck,10.6864,53.8655,y,y,n,y,n,null,null,n,null,null,1200,null";
const OUTSIDE: &str = "2,null,Road junction,10.7,53.9,n,n,n,n,n,null,null,n,null,null,null,null";
const BOUNDS: [f64; 4] = [10.68, 53.86, 10.69, 53.87];

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

struct FixtureDirectory(PathBuf);

impl FixtureDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "fabelgeist-viabundus-years-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let original = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/viabundus");
        for name in [
            "nodes.csv",
            "edges.csv",
            "population.csv",
            "alternativenames.csv",
            "descriptions.csv",
        ] {
            fs::copy(original.join(name), path.join(name)).unwrap();
        }
        Self(path)
    }

    fn replace(&self, name: &str, before: &str, after: &str) {
        let path = self.0.join(name);
        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents.matches(before).count(), 1);
        fs::write(path, contents.replacen(before, after, 1)).unwrap();
    }

    fn build(&self, year: i32, bounds: Option<[f64; 4]>) -> Result<ViabundusEnrichment, Error> {
        let mut builder = WorldBuilder::new(CalendarYear::new(year).unwrap());
        if let Some(bounds) = bounds {
            builder = builder.with_bounds(bounds);
        }
        builder.build_from_viabundus(&self.0)
    }
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn negative_interval_labels_are_admitted_before_and_after_spatial_filtering() {
    let fixture = FixtureDirectory::new();
    fixture.replace(
        "nodes.csv",
        INSIDE,
        "1,null,Lübeck,10.6864,53.8655,y,y,n,y,n,null,null,n,null,null,-16,null",
    );
    fixture.replace(
        "nodes.csv",
        OUTSIDE,
        "2,null,Road junction,10.7,53.9,n,n,n,n,n,null,null,n,null,null,-16,null",
    );
    fixture.replace(
        "alternativenames.csv",
        "1,Lubeke,null,1200,null,deu,1",
        "1,Lubeke,null,-16,null,deu,1",
    );
    let result = fixture.build(1544, Some(BOUNDS)).unwrap();
    assert_eq!(result.report.settlements, 1);
    assert_eq!(result.report.nodes, 1);
    assert_eq!(result.report.edges, 0);
    assert_eq!(result.report.excluded_edges["outside-playable-bounds"], 1);
    assert_eq!(result.settlement_aliases.len(), 1);
    assert_eq!(result.settlement_aliases[0].name, "Lubeke");
}

#[test]
fn negative_feature_and_edge_starts_preserve_exclusive_ends() {
    let fixture = FixtureDirectory::new();
    fixture.replace(
        "nodes.csv",
        INSIDE,
        "1,null,Lübeck,10.6864,53.8655,y,y,n,y,y,-16,1545,y,-16,1545,-16,null",
    );
    fixture.replace(
        "edges.csv",
        "10,land,1,2,1000,1.0,1,A,1200,null",
        "10,land,1,2,1000,1.0,1,A,-16,1546",
    );
    let active = fixture.build(1544, None).unwrap();
    assert_eq!(active.report.edges, 1);
    assert_eq!(active.report.route_crossings, 1);
    assert_eq!(active.report.toll_edges, 1);
    let ended = fixture.build(1545, None).unwrap();
    assert_eq!(ended.report.edges, 1);
    assert_eq!(ended.report.route_crossings, 0);
    assert_eq!(ended.report.toll_edges, 0);
    let route_ended = fixture.build(1546, None).unwrap();
    assert_eq!(route_ended.report.edges, 0);
    assert_eq!(route_ended.report.excluded_edges["inactive"], 1);
}

#[test]
fn negative_equal_dates_keep_feature_contradiction_distinct_from_empty_interval() {
    let fixture = FixtureDirectory::new();
    fixture.replace(
        "nodes.csv",
        INSIDE,
        "1,null,Lübeck,10.6864,53.8655,y,y,n,y,y,-16,-16,n,null,null,-16,-16",
    );
    let result = fixture.build(1544, None).unwrap();
    assert_eq!(result.report.contradictory_feature_dates, 1);
    assert_eq!(result.report.route_crossings, 0);
    assert_eq!(result.report.settlements, 0);
}

#[test]
fn interval_boundaries_include_starts_and_exclude_ends() {
    for (from, to, year, expected) in [
        ("-2147483648", "2147483647", 1544, 1),
        ("1544", "1545", 1544, 1),
        ("1544", "1545", 1545, 0),
        ("null", "-16", 1544, 0),
        ("2147483647", "null", i32::MAX, 1),
    ] {
        let fixture = FixtureDirectory::new();
        fixture.replace(
            "nodes.csv",
            INSIDE,
            &format!("1,null,Lübeck,10.6864,53.8655,y,y,n,y,n,null,null,n,null,null,{from},{to}"),
        );
        assert_eq!(
            fixture.build(year, None).unwrap().report.settlements,
            expected
        );
    }
}

#[test]
fn invalid_interval_values_still_fail_even_outside_playable_bounds() {
    for value in ["0", "bad", "2147483648"] {
        let fixture = FixtureDirectory::new();
        fixture.replace(
            "nodes.csv",
            OUTSIDE,
            &format!("2,null,Road junction,10.7,53.9,n,n,n,n,n,null,null,n,null,null,{value},null"),
        );
        let error = fixture.build(1544, Some(BOUNDS)).unwrap_err();
        let Error::InvalidField {
            path,
            field,
            value: actual,
            message,
        } = error
        else {
            panic!("expected an invalid source field");
        };
        assert!(path.ends_with("nodes.csv"));
        assert_eq!(field, "Settlement_From");
        assert_eq!(actual, value);
        if value == "0" {
            assert_eq!(message, "Viabundus source year zero is not supported");
        }
    }
}

#[test]
fn reversed_intervals_and_unset_dated_flags_remain_errors() {
    for (from, to, expected_value) in [
        (" 1545 ", "1544", "\" 1545 \"..\"1544\""),
        ("-1", "-16", "\"-1\"..\"-16\""),
    ] {
        let reversed = FixtureDirectory::new();
        reversed.replace(
            "nodes.csv",
            INSIDE,
            &format!("1,null,Lübeck,10.6864,53.8655,y,y,n,y,n,null,null,n,null,null,{from},{to}"),
        );
        let Error::InvalidField {
            path,
            field,
            value,
            message,
        } = reversed.build(1544, None).unwrap_err()
        else {
            panic!("expected an invalid source field");
        };
        assert!(path.ends_with("nodes.csv"));
        assert_eq!(field, "Settlement_From");
        assert_eq!(value, expected_value);
        assert_eq!(
            message,
            "Settlement_From must not be later than Settlement_To"
        );
    }

    let unset = FixtureDirectory::new();
    unset.replace(
        "nodes.csv",
        INSIDE,
        "1,null,Lübeck,10.6864,53.8655,y,y,n,y,n,-16,null,n,null,null,1200,null",
    );
    assert!(matches!(
        unset.build(1544, None).unwrap_err(),
        Error::InvalidField { field: "Bridge", message, .. }
            if message == "feature dates are present while its source flag is unset"
    ));
}

#[test]
fn population_observation_year_keeps_its_existing_positive_contract() {
    let fixture = FixtureDirectory::new();
    fixture.replace("population.csv", "1,1500,12", "1,-16,12");
    assert!(matches!(
        fixture.build(1544, None).unwrap_err(),
        Error::InvalidField { field: "year", .. }
    ));
}
