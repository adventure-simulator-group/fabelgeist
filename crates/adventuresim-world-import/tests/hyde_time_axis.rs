//! Public importer behavior against generated HYDE-shaped NetCDF inputs.

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use adventuresim_world_import::{Error, hyde_crop_cells};
use adventuresim_world_schema::calendar::CalendarYear;
use zip::ZipArchive;

const BOUNDS: [f64; 4] = [0.02, -0.06, 0.06, -0.02];
const COMPONENTS: [&str; 3] = ["cropland.nc", "grazing_land.nc", "urban_area.nc"];

struct Fixtures {
    root: PathBuf,
}

enum Expected {
    Crop(f64),
    Uncovered,
    Invalid {
        filename: &'static str,
        message: &'static str,
    },
    InvalidComponentValue {
        filename: &'static str,
        field: &'static str,
        value: &'static str,
    },
}

struct Case {
    scenario: &'static str,
    year: i32,
    expected: Expected,
}

impl Fixtures {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("hyde-time-{}-{unique}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let archive =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hyde-time/fixtures.zip");
        ZipArchive::new(File::open(archive).unwrap())
            .unwrap()
            .extract(root.join("inputs"))
            .unwrap();
        Self { root }
    }

    fn scenario(&self, name: &str) -> PathBuf {
        let inputs = self.root.join("inputs");
        let destination = self.root.join("cases").join(name);
        fs::create_dir_all(&destination).unwrap();
        let variants = inputs.join(name);
        assert!(variants.is_dir(), "missing fixture scenario {name}");
        let expected: &[&str] = match name {
            "ascending"
            | "descending"
            | "negative"
            | "valid_range"
            | "identity_transform"
            | "unused_markers"
            | "float_valid_range"
            | "wide_offsets"
            | "small_negative_days"
            | "valid_min_max"
            | "grazing_descending_order"
            | "urban_descending_order"
            | "grazing_descending_later"
            | "urban_descending_later"
            | "rounding_inside" => &COMPONENTS,
            "component_mismatch"
            | "later_missing_marker"
            | "near_equal_component"
            | "opposite_component" => &["grazing_land.nc"],
            "later_invalid_minimum" => &["urban_area.nc"],
            _ => &["cropland.nc"],
        };
        for filename in COMPONENTS {
            let variant = variants.join(filename);
            assert_eq!(
                variant.is_file(),
                expected.contains(&filename),
                "unexpected fixture components for {name}: {filename}",
            );
            let source = if variant.is_file() {
                variant
            } else {
                inputs.join("negative").join(filename)
            };
            fs::copy(source, destination.join(filename)).unwrap();
        }
        fs::copy(
            inputs.join("area/general_files.zip"),
            destination.join("general_files.zip"),
        )
        .unwrap();
        destination
    }
}

impl Drop for Fixtures {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Case {
    fn crop(scenario: &'static str, year: i32, value: f64) -> Self {
        Self {
            scenario,
            year,
            expected: Expected::Crop(value),
        }
    }

    fn uncovered(scenario: &'static str, year: i32) -> Self {
        Self {
            scenario,
            year,
            expected: Expected::Uncovered,
        }
    }

    fn invalid(scenario: &'static str, filename: &'static str, message: &'static str) -> Self {
        Self {
            scenario,
            year: 1,
            expected: Expected::Invalid { filename, message },
        }
    }

    fn invalid_component(
        scenario: &'static str,
        year: i32,
        filename: &'static str,
        field: &'static str,
        value: &'static str,
    ) -> Self {
        Self {
            scenario,
            year,
            expected: Expected::InvalidComponentValue {
                filename,
                field,
                value,
            },
        }
    }
}

#[test]
fn hyde_crop_cells_selects_checked_time_axes_and_rejects_invalid_metadata() {
    let fixtures = Fixtures::new();
    let cases = [
        Case::crop("ascending", 1544, 18.8),
        Case::crop("ascending", 1500, 10.0),
        Case::crop("ascending", 1600, 30.0),
        Case::uncovered("ascending", 1499),
        Case::uncovered("ascending", 1601),
        Case::crop("descending", 1544, 18.8),
        Case::crop("descending", 1500, 10.0),
        Case::crop("descending", 1600, 30.0),
        Case::uncovered("descending", 1499),
        Case::uncovered("descending", 1601),
        Case::crop("negative", 1, 20.0),
        Case::crop("negative", 2, 30.0),
        Case::uncovered("negative", 3),
        Case::crop("valid_range", 1, 20.0),
        Case::crop("identity_transform", 1, 20.0),
        Case::crop("unused_markers", 1, 20.0),
        Case::crop("float_valid_range", 1, 20.0),
        Case::crop(
            "wide_offsets",
            1,
            10.0 + 20.0 * (2_147_483_648_f64 / 4_294_967_295_f64),
        ),
        Case::crop(
            "wide_offsets",
            i32::MAX,
            10.0 + 20.0 * (4_294_967_294_f64 / 4_294_967_295_f64),
        ),
        Case::crop("small_negative_days", 1, 10.0),
        Case::crop("rounding_inside", 1, 10.0),
        Case::invalid(
            "rounding_outside",
            "cropland.nc",
            "expected a whole HYDE 365-day year",
        ),
        Case::invalid(
            "offset_above_i32",
            "cropland.nc",
            "year offset is out of range",
        ),
        Case::invalid(
            "offset_below_i32",
            "cropland.nc",
            "year offset is out of range",
        ),
        Case::invalid_component(
            "grazing_descending_order",
            1500,
            "grazing_land.nc",
            "grazing_land",
            "-8",
        ),
        Case::invalid_component(
            "grazing_descending_order",
            1544,
            "grazing_land.nc",
            "grazing_land",
            "-8",
        ),
        Case::invalid_component(
            "grazing_descending_order",
            1600,
            "grazing_land.nc",
            "grazing_land",
            "-7",
        ),
        Case::invalid_component(
            "urban_descending_order",
            1500,
            "urban_area.nc",
            "urban_area",
            "-8",
        ),
        Case::invalid_component(
            "urban_descending_order",
            1544,
            "urban_area.nc",
            "urban_area",
            "-8",
        ),
        Case::invalid_component(
            "urban_descending_order",
            1600,
            "urban_area.nc",
            "urban_area",
            "-7",
        ),
        Case::invalid_component(
            "grazing_descending_later",
            1544,
            "grazing_land.nc",
            "grazing_land",
            "-7",
        ),
        Case::invalid_component(
            "urban_descending_later",
            1544,
            "urban_area.nc",
            "urban_area",
            "-7",
        ),
        Case::invalid("duplicate", "cropland.nc", "duplicate whole-year offsets"),
        Case::invalid(
            "positive_extreme",
            "cropland.nc",
            "year offset is out of range",
        ),
        Case::invalid(
            "negative_extreme",
            "cropland.nc",
            "year offset is out of range",
        ),
        Case::invalid(
            "nonfinite",
            "cropland.nc",
            "finite, strictly monotonic coordinates",
        ),
        Case::invalid(
            "fill_marker",
            "cropland.nc",
            "contains a declared missing value",
        ),
        Case::invalid(
            "missing_second_marker",
            "cropland.nc",
            "contains a declared missing value",
        ),
        Case::invalid(
            "invalid_minimum",
            "cropland.nc",
            "outside the declared valid bounds",
        ),
        Case::invalid(
            "reversed_range",
            "cropland.nc",
            "finite, ordered HYDE time-coordinate bounds",
        ),
        Case::invalid(
            "nonidentity_transform",
            "cropland.nc",
            "numeric identity time-coordinate transform",
        ),
        Case::invalid(
            "component_mismatch",
            "grazing_land.nc",
            "coordinate axis differs",
        ),
        Case::invalid(
            "later_missing_marker",
            "grazing_land.nc",
            "contains a declared missing value",
        ),
        Case::invalid(
            "later_invalid_minimum",
            "urban_area.nc",
            "outside the declared valid bounds",
        ),
        Case::invalid(
            "missing_wrong_type",
            "cropland.nc",
            "coordinate storage type",
        ),
        Case::invalid("range_wrong_type", "cropland.nc", "coordinate storage type"),
        Case::invalid(
            "range_three_values",
            "cropland.nc",
            "expected two HYDE time-coordinate bounds",
        ),
        Case::invalid(
            "range_conflict",
            "cropland.nc",
            "conflicting HYDE time-coordinate bounds",
        ),
        Case::crop("valid_min_max", 1, 20.0),
        Case::invalid(
            "range_nonfinite",
            "cropland.nc",
            "finite, ordered HYDE time-coordinate bounds",
        ),
        Case::crop("unused_nan_marker", 1, 20.0),
        Case::crop("identity_float_scale", 1, 20.0),
        Case::invalid(
            "scale_string",
            "cropland.nc",
            "numeric identity time-coordinate transform",
        ),
        Case::invalid(
            "scale_vector",
            "cropland.nc",
            "numeric identity time-coordinate transform",
        ),
        Case::invalid(
            "scale_nonfinite",
            "cropland.nc",
            "numeric identity time-coordinate transform",
        ),
        Case::invalid(
            "offset_nonidentity",
            "cropland.nc",
            "numeric identity time-coordinate transform",
        ),
        Case::crop("near_equal_component", 1, 20.0),
        Case::invalid(
            "opposite_component",
            "grazing_land.nc",
            "coordinate axis differs",
        ),
    ];

    for case in cases {
        let directory = fixtures.scenario(case.scenario);
        let result = hyde_crop_cells(&directory, CalendarYear::new(case.year).unwrap(), BOUNDS);
        match (case.expected, result) {
            (Expected::Crop(expected), Ok((cells, _))) => {
                assert_eq!(cells.len(), 1, "{} year {}", case.scenario, case.year);
                let cell = &cells[0];
                assert_eq!((cell.row, cell.column), (1080, 2160), "{}", case.scenario);
                assert!(
                    (cell.crop_km2 - expected).abs() <= 1e-12,
                    "{} year {}: expected {expected}, got {}",
                    case.scenario,
                    case.year,
                    cell.crop_km2,
                );
            }
            (Expected::Uncovered, Err(Error::Validation(message))) => {
                assert!(message.contains("does not cover HYDE year"), "{message}");
            }
            (
                Expected::Invalid { filename, message },
                Err(Error::InvalidField {
                    path,
                    field,
                    message: actual,
                    ..
                }),
            ) => {
                assert_eq!(field, "time", "{}", case.scenario);
                assert_eq!(
                    path.file_name(),
                    Some(std::ffi::OsStr::new(filename)),
                    "{}",
                    case.scenario
                );
                assert!(actual.contains(message), "{}: {actual}", case.scenario);
            }
            (
                Expected::InvalidComponentValue {
                    filename,
                    field,
                    value,
                },
                Err(Error::InvalidField {
                    path,
                    field: actual_field,
                    value: actual_value,
                    ..
                }),
            ) => {
                assert_eq!(
                    path.file_name(),
                    Some(std::ffi::OsStr::new(filename)),
                    "{}",
                    case.scenario
                );
                assert_eq!(actual_field, field, "{}", case.scenario);
                assert_eq!(actual_value, value, "{} year {}", case.scenario, case.year);
            }
            (_, actual) => panic!(
                "unexpected result for {} year {}: {actual:?}",
                case.scenario, case.year
            ),
        }
    }
}
