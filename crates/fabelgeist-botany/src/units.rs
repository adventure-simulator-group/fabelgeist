//! Units carried by the ecosystem and its growth kernels.
//! Native extraction is reserved for authored catalogs, numerical formulas and UI adapters.

/// Mean annual temperature in degrees Celsius.
use fabelgeist_math::units::Metres;
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Celsius(f32);

/// Temperature tolerance in degrees Celsius.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct CelsiusDelta(f32);

/// Annual precipitation in millimetres.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Millimetres(f32);

/// Temperature lapse per metre of elevation.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct CelsiusPerMetre(f32);

/// Elapsed simulation time in years, distinct from physiological development.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Years(f32);

/// Development along a prototype, distinct from elapsed years.
///
/// ```compile_fail
/// use fabelgeist_botany::units::{PhysiologicalAge, Years};
/// let age: PhysiologicalAge = Years::from_years(2.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct PhysiologicalAge(f32);

/// Physiological development per simulation year.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct AgeRate(f32);

/// A location on terrain, in the ecosystem's world-space metre frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundPoint {
    pub x: Metres,
    pub z: Metres,
}

/// A finite, positive regulation interval. A zero step cannot advance a forest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepDuration(Years);

impl Celsius {
    pub const fn from_degrees(value: f32) -> Self {
        Self(value)
    }
    pub const fn degrees(self) -> f32 {
        self.0
    }
}

impl CelsiusDelta {
    pub const fn from_degrees(value: f32) -> Self {
        Self(value)
    }
    pub const fn degrees(self) -> f32 {
        self.0
    }
}

impl Millimetres {
    pub const fn from_millimetres(value: f32) -> Self {
        Self(value)
    }
    pub const fn millimetres(self) -> f32 {
        self.0
    }
}

impl CelsiusPerMetre {
    pub const fn from_degrees_per_metre(value: f32) -> Self {
        Self(value)
    }
    pub const fn degrees_per_metre(self) -> f32 {
        self.0
    }
}

impl Years {
    pub const fn from_years(value: f32) -> Self {
        Self(value)
    }
    pub const fn years(self) -> f32 {
        self.0
    }
}

impl PhysiologicalAge {
    pub const fn from_stage(value: f32) -> Self {
        Self(value)
    }
    pub const fn stage(self) -> f32 {
        self.0
    }
}

impl AgeRate {
    pub const fn from_stages_per_year(value: f32) -> Self {
        Self(value)
    }
    pub const fn stages_per_year(self) -> f32 {
        self.0
    }
}

impl GroundPoint {
    pub const fn new(x: Metres, z: Metres) -> Self {
        Self { x, z }
    }
}

impl StepDuration {
    pub const HALF_YEAR: Self = Self(Years::from_years(0.5));
    pub fn try_new(duration: Years) -> crate::error::Result<Self> {
        let years = duration.years();
        if !years.is_finite() || years <= 0.0 {
            return Err(crate::error::BotanyError::InvalidStep { duration });
        }
        Ok(Self(duration))
    }
    pub const fn duration(self) -> Years {
        self.0
    }
}
