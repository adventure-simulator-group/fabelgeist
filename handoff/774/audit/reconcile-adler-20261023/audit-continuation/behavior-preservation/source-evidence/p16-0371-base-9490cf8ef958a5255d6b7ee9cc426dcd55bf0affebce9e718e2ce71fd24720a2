//! Native report quantities retain f64 precision through sums and reductions.
//!
//! Roles distinguish clocks and units even when they share numeric storage.
//! Native f32 observations are promoted once; sums and means never narrow back
//! into gameplay quantities. Scalar projection belongs to the report serializer
//! or to the named arithmetic operations below.

use std::{marker::PhantomData, time::Duration};

use adventuresim_core::{
    elapsed::AnimationElapsed, linear_speed::NonnegativeSpeed, unit_interval::UnitInterval,
};
use adventuresim_tactical_server::iteration::TacticalElapsed;
use serde::Serialize;

pub(super) type MeasurementResult<T> = Result<T, MeasurementError>;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub(super) struct Measurement<Role> {
    value: f64,
    #[serde(skip)]
    role: PhantomData<Role>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum MeasurementError {
    #[error("native report {quantity:?} must be finite")]
    NonFinite { quantity: ReportQuantity },
    #[error("native report {quantity:?} arithmetic overflowed")]
    Overflow { quantity: ReportQuantity },
}

pub(super) trait ReportUnit: Copy + Default + PartialOrd {
    const QUANTITY: ReportQuantity;
}

macro_rules! report_quantities {
    ($($role:ident),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(crate) enum ReportQuantity {
            $($role),+
        }

        $(
            #[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
            pub(super) struct $role;

            impl ReportUnit for $role {
                const QUANTITY: ReportQuantity = ReportQuantity::$role;
            }
        )+
    };
}

report_quantities! {
    TacticalSeconds,
    AutoresolveSeconds,
    ContactMetres,
    CenterSeparationMetres,
    WeaponReachMetres,
    MovementMetres,
    MovementSpeed,
    ContactJoules,
    HealthDamage,
    BloodLossFraction,
    WoundFlowFractionPerSecond,
    Fatigue,
    AcuteTrauma,
    Imbalance,
    PainIncapacitation,
    IncapacitationPerformance,
    AttackPowerMultiplier,
    WinRate,
    WinRateDifference,
    WallSeconds,
    WallMilliseconds,
    DuelsPerWallSecond,
    TacticalSecondsPerWallSecond,
}

impl<Role: ReportUnit> Measurement<Role> {
    pub(super) const ZERO: Self = Self {
        value: 0.0,
        role: PhantomData,
    };

    /// Admission at a native observation or authored report-limit boundary.
    pub(super) fn from_native(value: f32) -> MeasurementResult<Self> {
        Self::from_f64(f64::from(value))
    }

    pub(super) fn checked_add(&mut self, sample: Self) -> MeasurementResult<()> {
        *self = Self::from_arithmetic(self.value + sample.value)?;
        Ok(())
    }

    /// Divide after the original ordered sum. An unobserved mean keeps its
    /// existing zero representation; contact extrema use Option separately.
    pub(super) fn mean(self, samples: u64) -> MeasurementResult<Self> {
        if samples == 0 {
            return Ok(self);
        }
        Self::from_arithmetic(self.value / samples as f64)
    }

    pub(super) fn min(self, other: Self) -> Self {
        Self {
            value: self.value.min(other.value),
            role: PhantomData,
        }
    }

    pub(super) fn max(self, other: Self) -> Self {
        Self {
            value: self.value.max(other.value),
            role: PhantomData,
        }
    }

    fn from_f64(value: f64) -> MeasurementResult<Self> {
        if !value.is_finite() {
            return Err(MeasurementError::NonFinite {
                quantity: Role::QUANTITY,
            });
        }
        Ok(Self {
            value,
            role: PhantomData,
        })
    }

    fn from_arithmetic(value: f64) -> MeasurementResult<Self> {
        Self::from_f64(value).map_err(|_| MeasurementError::Overflow {
            quantity: Role::QUANTITY,
        })
    }
}

impl Measurement<TacticalSeconds> {
    pub(super) fn from_elapsed(elapsed: TacticalElapsed) -> MeasurementResult<Self> {
        Self::from_native(elapsed.seconds())
    }

    pub(super) fn from_interval(interval: AnimationElapsed) -> MeasurementResult<Self> {
        Self::from_native(interval.seconds())
    }

    pub(super) fn per_wall_second(
        self,
        elapsed: Measurement<WallSeconds>,
    ) -> MeasurementResult<Measurement<TacticalSecondsPerWallSecond>> {
        let value = if elapsed.value > 0.0 {
            self.value / elapsed.value
        } else {
            0.0
        };
        Measurement::from_arithmetic(value)
    }
}

impl Measurement<AutoresolveSeconds> {
    pub(super) fn from_elapsed(elapsed: AnimationElapsed) -> MeasurementResult<Self> {
        Self::from_native(elapsed.seconds())
    }
}

impl Measurement<IncapacitationPerformance> {
    pub(super) fn from_performance(performance: UnitInterval) -> MeasurementResult<Self> {
        Self::from_native(performance.fraction())
    }
}

impl Measurement<MovementSpeed> {
    pub(super) fn from_speed(speed: NonnegativeSpeed) -> MeasurementResult<Self> {
        Self::from_native(speed.metres_per_second())
    }
}

impl Measurement<WallSeconds> {
    pub(super) fn from_duration(elapsed: Duration) -> MeasurementResult<Self> {
        Self::from_f64(elapsed.as_secs_f64())
    }

    pub(super) fn milliseconds(self) -> MeasurementResult<Measurement<WallMilliseconds>> {
        Measurement::from_arithmetic(self.value * 1_000.0)
    }

    pub(super) fn duels_per_second(
        self,
        duels: u64,
    ) -> MeasurementResult<Measurement<DuelsPerWallSecond>> {
        let value = if self.value > 0.0 {
            duels as f64 / self.value
        } else {
            0.0
        };
        Measurement::from_arithmetic(value)
    }
}

impl Measurement<WinRate> {
    pub(super) fn from_counts(wins: u64, runs: u64) -> MeasurementResult<Self> {
        Self::from_arithmetic(wins as f64 / runs as f64)
    }

    pub(super) fn difference(self, other: Self) -> MeasurementResult<Measurement<WinRateDifference>> {
        Measurement::from_arithmetic(self.value - other.value)
    }
}
