//! Checked HYDE time coordinates and physical slice selection.
//!
//! Attribute checks use the element counts exposed by the NetCDF reader. It
//! does not expose the original HDF5 attribute dataspace rank.

use std::path::Path;

use adventuresim_world_schema::calendar::CalendarYear;
use netcdf_reader::{NcAttrValue, NcFile, NcType, NcVariable};

use super::{invalid, nc, read_axis, require_valid_axis_values};
use crate::{Error, Result};

const HYDE_DAYS_PER_YEAR: f64 = 365.0;
// Rounding tolerance is measured in 365-day years; component matching uses days.
const WHOLE_YEAR_TOLERANCE: f64 = 1e-6;
const COMPONENT_TIME_TOLERANCE_DAYS: f64 = 1e-6;

#[derive(Clone)]
pub(super) struct TimeAxis {
    coordinates: Vec<TimeCoordinate>,
}

#[derive(Clone, Copy)]
struct TimeCoordinate {
    days: f64,
    offset: YearOffset,
}

/// Whole 365-day years relative to the required HYDE reference date.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct YearOffset(i32);

#[derive(Clone, Copy)]
struct TimeIndex(usize);

#[derive(Clone, Copy)]
struct InterpolationPosition(f64);

#[derive(Clone, Copy)]
pub(super) struct TimeSelection(Selection);

#[derive(Clone, Copy)]
enum Selection {
    Exact {
        index: TimeIndex,
    },
    Interpolated {
        earlier: TimeIndex,
        later: TimeIndex,
        position: InterpolationPosition,
    },
}

struct TimeBounds {
    minimum: Option<f64>,
    maximum: Option<f64>,
}

impl TimeAxis {
    pub(super) fn read(file: &NcFile, path: &Path) -> Result<Self> {
        let values = read_axis(file, path, "time")?;
        let time = nc(path, file.variable("time"))?;
        require_time_metadata(path, time, &values)?;
        Self::from_days(path, values)
    }

    fn from_days(path: &Path, values: Vec<f64>) -> Result<Self> {
        require_valid_axis_values(path, "time", &values)?;
        let coordinates = values
            .into_iter()
            .map(|days| {
                Ok(TimeCoordinate {
                    days,
                    offset: YearOffset::from_days(path, days)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        if coordinates
            .windows(2)
            .any(|pair| pair[0].offset == pair[1].offset)
        {
            return Err(invalid(
                path,
                "time",
                format!("{} values", coordinates.len()),
                "HYDE time coordinates contain duplicate whole-year offsets",
            ));
        }
        Ok(Self { coordinates })
    }

    pub(super) fn len(&self) -> usize {
        self.coordinates.len()
    }

    pub(super) fn require_matches(&self, path: &Path, actual: &Self) -> Result<()> {
        if self.coordinates.len() != actual.coordinates.len()
            || self
                .coordinates
                .iter()
                .zip(&actual.coordinates)
                .any(|(expected, actual)| {
                    (expected.days - actual.days).abs() > COMPONENT_TIME_TOLERANCE_DAYS
                        || expected.offset != actual.offset
                })
        {
            return Err(invalid(
                path,
                "time",
                format!("{} values", actual.coordinates.len()),
                "coordinate axis differs from the other HYDE component files",
            ));
        }
        Ok(())
    }

    pub(super) fn select(&self, path: &Path, year: CalendarYear) -> Result<TimeSelection> {
        let target = YearOffset(year.years_since_first());
        if let Some(index) = self
            .coordinates
            .iter()
            .position(|coordinate| coordinate.offset == target)
        {
            return Ok(TimeSelection(Selection::Exact {
                index: TimeIndex(index),
            }));
        }
        for (index, pair) in self.coordinates.windows(2).enumerate() {
            let (earlier, later) = if pair[0].offset < pair[1].offset {
                (index, index + 1)
            } else {
                (index + 1, index)
            };
            let first = self.coordinates[earlier].offset;
            let last = self.coordinates[later].offset;
            if first < target && target < last {
                let position =
                    InterpolationPosition::between(first, target, last).ok_or_else(|| {
                        invalid(
                            path,
                            "time",
                            year.to_string(),
                            "could not represent the HYDE interpolation position",
                        )
                    })?;
                return Ok(TimeSelection(Selection::Interpolated {
                    earlier: TimeIndex(earlier),
                    later: TimeIndex(later),
                    position,
                }));
            }
        }
        Err(Error::Validation(format!(
            "{} does not cover HYDE year {year}",
            path.display()
        )))
    }
}

impl YearOffset {
    fn from_days(path: &Path, days: f64) -> Result<Self> {
        let years = days / HYDE_DAYS_PER_YEAR;
        let rounded = years.round();
        if !days.is_finite() || (years - rounded).abs() > WHOLE_YEAR_TOLERANCE {
            return Err(invalid(
                path,
                "time",
                days.to_string(),
                "expected a whole HYDE 365-day year",
            ));
        }
        if rounded < f64::from(i32::MIN) || rounded > f64::from(i32::MAX) {
            return Err(invalid(
                path,
                "time",
                days.to_string(),
                "HYDE year offset is out of range",
            ));
        }
        Ok(Self(rounded as i32))
    }
}

impl InterpolationPosition {
    fn between(first: YearOffset, target: YearOffset, last: YearOffset) -> Option<Self> {
        if !(first < target && target < last) {
            return None;
        }
        // Differences of i32 offsets fit in i64 and are exact in binary64.
        let numerator = i64::from(target.0) - i64::from(first.0);
        let denominator = i64::from(last.0) - i64::from(first.0);
        let amount = numerator as f64 / denominator as f64;
        (amount > 0.0 && amount < 1.0).then_some(Self(amount))
    }
}

impl TimeSelection {
    pub(super) fn start_index(self) -> usize {
        match self.0 {
            Selection::Exact { index } => index.0,
            Selection::Interpolated { earlier, .. } => earlier.0,
        }
    }

    pub(super) fn end_index(self) -> Option<usize> {
        match self.0 {
            Selection::Exact { .. } => None,
            Selection::Interpolated { later, .. } => Some(later.0),
        }
    }

    pub(super) fn amount(self) -> f64 {
        match self.0 {
            Selection::Exact { .. } => 0.0,
            Selection::Interpolated { position, .. } => position.0,
        }
    }
}

fn require_time_metadata(path: &Path, time: &NcVariable, values: &[f64]) -> Result<()> {
    let units = time
        .attribute("units")
        .and_then(|attribute| attribute.value.as_string());
    let calendar = time
        .attribute("calendar")
        .and_then(|attribute| attribute.value.as_string());
    if units.as_deref() != Some("days since 1-5-1 00:00:00")
        || calendar.as_deref() != Some("365_day")
    {
        return Err(invalid(
            path,
            "time",
            format!("units={units:?}, calendar={calendar:?}"),
            "expected HYDE 3.5's 365-day time axis",
        ));
    }
    require_identity_attribute(path, time, "scale_factor", 1.0)?;
    require_identity_attribute(path, time, "add_offset", 0.0)?;
    require_unused_markers(path, time, values)?;
    require_valid_bounds(path, time, values)
}

fn require_identity_attribute(
    path: &Path,
    time: &NcVariable,
    name: &'static str,
    identity: f64,
) -> Result<()> {
    let Some(attribute) = time.attribute(name) else {
        return Ok(());
    };
    let numeric = attribute.value.as_f64_vec();
    if !matches!(numeric.as_deref(), Some([value]) if value.is_finite() && *value == identity) {
        return Err(invalid(
            path,
            "time",
            format!("{name}={:?}", attribute.value),
            "expected one numeric identity time-coordinate transform value",
        ));
    }
    Ok(())
}

fn require_unused_markers(path: &Path, time: &NcVariable, values: &[f64]) -> Result<()> {
    for name in ["_FillValue", "missing_value"] {
        let Some(markers) = stored_attribute(path, time, name)? else {
            continue;
        };
        if markers.is_empty() || (name == "_FillValue" && markers.len() != 1) {
            return Err(invalid(
                path,
                "time",
                format!("{name}={markers:?}"),
                "expected one fill marker or a nonempty missing-value list",
            ));
        }
        if values.iter().any(|value| markers.contains(value)) {
            return Err(invalid(
                path,
                "time",
                format!("{name}={markers:?}"),
                "HYDE time axis contains a declared missing value",
            ));
        }
    }
    Ok(())
}

fn require_valid_bounds(path: &Path, time: &NcVariable, values: &[f64]) -> Result<()> {
    let range = stored_attribute(path, time, "valid_range")?;
    let minimum = scalar_bound(path, time, "valid_min")?;
    let maximum = scalar_bound(path, time, "valid_max")?;
    if range.is_some() && (minimum.is_some() || maximum.is_some()) {
        return Err(invalid(
            path,
            "time",
            "valid_range with valid_min or valid_max".into(),
            "conflicting HYDE time-coordinate bounds",
        ));
    }
    let bounds = if let Some(range) = range {
        let [minimum, maximum] = range.as_slice() else {
            return Err(invalid(
                path,
                "time",
                format!("valid_range={range:?}"),
                "expected two HYDE time-coordinate bounds",
            ));
        };
        TimeBounds {
            minimum: Some(*minimum),
            maximum: Some(*maximum),
        }
    } else {
        TimeBounds { minimum, maximum }
    };
    if bounds.minimum.is_some_and(|bound| !bound.is_finite())
        || bounds.maximum.is_some_and(|bound| !bound.is_finite())
        || bounds
            .minimum
            .zip(bounds.maximum)
            .is_some_and(|(minimum, maximum)| minimum > maximum)
    {
        return Err(invalid(
            path,
            "time",
            format!("minimum={:?}, maximum={:?}", bounds.minimum, bounds.maximum),
            "expected finite, ordered HYDE time-coordinate bounds",
        ));
    }
    if let Some(value) = values.iter().find(|value| {
        bounds.minimum.is_some_and(|minimum| **value < minimum)
            || bounds.maximum.is_some_and(|maximum| **value > maximum)
    }) {
        return Err(invalid(
            path,
            "time",
            value.to_string(),
            "HYDE time coordinate lies outside the declared valid bounds",
        ));
    }
    Ok(())
}

fn scalar_bound(path: &Path, time: &NcVariable, name: &'static str) -> Result<Option<f64>> {
    let Some(values) = stored_attribute(path, time, name)? else {
        return Ok(None);
    };
    let [value] = values.as_slice() else {
        return Err(invalid(
            path,
            "time",
            format!("{name}={values:?}"),
            "expected one HYDE time-coordinate bound",
        ));
    };
    Ok(Some(*value))
}

fn stored_attribute(
    path: &Path,
    time: &NcVariable,
    name: &'static str,
) -> Result<Option<Vec<f64>>> {
    let Some(attribute) = time.attribute(name) else {
        return Ok(None);
    };
    let values = match (&time.dtype, &attribute.value) {
        (NcType::Float, NcAttrValue::Floats(values)) => {
            values.iter().copied().map(f64::from).collect()
        }
        (NcType::Double, NcAttrValue::Doubles(values)) => values.clone(),
        _ => {
            return Err(invalid(
                path,
                "time",
                format!("{name}={:?}", attribute.value),
                "expected a time attribute with the coordinate storage type",
            ));
        }
    };
    Ok(Some(values))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use adventuresim_world_schema::calendar::CalendarYear;

    use super::super::LandUseSourceValues;
    use super::{TimeAxis, TimeSelection, YearOffset};

    fn selection_for_1544() -> TimeSelection {
        let path = Path::new("hyde.nc");
        let axis = TimeAxis::from_days(path, vec![547_135.0, 583_635.0]).unwrap();
        axis.select(path, CalendarYear::new(1544).unwrap()).unwrap()
    }

    #[test]
    fn hyde_time_axis_interpolates_1544_between_1500_and_1600() {
        let path = Path::new("hyde.nc");
        let selection = selection_for_1544();
        assert_eq!(selection.start_index(), 0);
        assert_eq!(selection.end_index(), Some(1));
        assert_eq!(selection.amount(), 0.44);
        assert!(YearOffset::from_days(path, 547_135.0).unwrap() == YearOffset(1499));
    }

    #[test]
    fn source_areas_interpolate_into_an_exhaustive_profile() {
        let values = LandUseSourceValues {
            cell_area: Some(100.0),
            crop_start: Some(10.0),
            crop_end: Some(30.0),
            grazing_start: Some(20.0),
            grazing_end: Some(20.0),
            urban_start: Some(1.0),
            urban_end: Some(1.0),
        };
        let (profile, normalized) = values.profile(selection_for_1544()).unwrap().unwrap();
        assert!(!normalized);
        assert_eq!(profile.cropland().basis_points(), 1_880);
        assert_eq!(profile.grazing().basis_points(), 2_000);
        assert_eq!(profile.built_up().basis_points(), 100);
        assert_eq!(profile.natural().basis_points(), 6_020);
    }

    #[test]
    fn partial_nodata_fails_closed() {
        let values = LandUseSourceValues {
            cell_area: Some(100.0),
            crop_start: Some(1.0),
            ..Default::default()
        };
        assert!(values.profile(selection_for_1544()).is_err());
    }
}
