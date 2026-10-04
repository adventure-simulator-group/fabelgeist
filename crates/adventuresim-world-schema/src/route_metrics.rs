//! Bounded terrain metrics in compiled route records.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouteMetricOutOfRange {
    pub metric: &'static str,
    pub value: i64,
    pub min: i64,
    pub max: i64,
}
impl std::fmt::Display for RouteMetricOutOfRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} is outside {}..={}",
            self.metric, self.value, self.min, self.max
        )
    }
}
impl std::error::Error for RouteMetricOutOfRange {}

macro_rules! bounded_route_metric {
    ($name:ident, $field:ident, $inner:ty, $min:expr, $max:expr) => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
        pub struct $name {
            $field: $inner,
        }
        impl $name {
            pub const MIN: $inner = $min;
            pub const MAX: $inner = $max;
            pub fn new(value: $inner) -> Result<Self, RouteMetricOutOfRange> {
                if (Self::MIN..=Self::MAX).contains(&value) {
                    Ok(Self { $field: value })
                } else {
                    Err(RouteMetricOutOfRange {
                        metric: stringify!($name),
                        value: i64::from(value),
                        min: i64::from(Self::MIN),
                        max: i64::from(Self::MAX),
                    })
                }
            }
            pub const fn get(self) -> $inner {
                self.$field
            }
        }
        impl TryFrom<$inner> for $name {
            type Error = RouteMetricOutOfRange;
            fn try_from(value: $inner) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
        #[cfg(feature = "spacetimedb")]
        crate::checked_sats::checked_numeric_product!($name, $field: $inner);
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                #[derive(Deserialize)]
                struct Wire {
                    $field: $inner,
                }
                Self::new(Wire::deserialize(deserializer)?.$field).map_err(serde::de::Error::custom)
            }
        }
    };
}

bounded_route_metric!(RouteVerticalMeters, meters, u32, 0, 100_000);
bounded_route_metric!(RouteSignedGradePermille, permille, i16, -10_000, 10_000);
bounded_route_metric!(RouteSlopePermille, permille, u16, 0, 10_000);
bounded_route_metric!(RouteRoughnessMeters, meters, u16, 0, 9_500);
bounded_route_metric!(RouteReliefMeters, meters, u16, 0, 9_500);

pub fn route_grade_permille(
    dz: i32,
    length_m: u32,
    progress_delta: u32,
) -> Result<RouteSignedGradePermille, String> {
    if length_m == 0 || progress_delta == 0 {
        return Err("route grade requires positive length and progress delta".into());
    }
    const PERMILLE_SQUARED_SCALE: i64 = 1_000_000;
    let numerator = i64::from(dz) * PERMILLE_SQUARED_SCALE;
    let denominator = i64::from(length_m)
        .checked_mul(i64::from(progress_delta))
        .ok_or("route grade denominator overflow")?;
    let magnitude = (numerator.unsigned_abs() + denominator as u64 / 2) / denominator as u64;
    let signed = if numerator < 0 {
        -(magnitude as i64)
    } else {
        magnitude as i64
    };
    RouteSignedGradePermille::new(signed.clamp(-10_000, 10_000) as i16)
        .map_err(|error| error.to_string())
}

#[cfg(all(test, feature = "spacetimedb"))]
mod tests {
    use super::*;
    use spacetimedb_lib::bsatn;

    #[test]
    fn database_route_metrics_check_signed_and_unsigned_bounds() {
        for raw in [-10_001_i16, 10_001] {
            let wire = bsatn::to_vec(&raw).unwrap();
            assert!(bsatn::from_slice::<RouteSignedGradePermille>(&wire).is_err());
        }
        let oversized = bsatn::to_vec(&100_001_u32).unwrap();
        assert!(bsatn::from_slice::<RouteVerticalMeters>(&oversized).is_err());
        let wire = bsatn::to_vec(&-10_000_i16).unwrap();
        let grade = bsatn::from_slice::<RouteSignedGradePermille>(&wire).unwrap();
        assert_eq!(bsatn::to_vec(&grade).unwrap(), wire);
        assert_eq!(
            serde_json::to_string(&grade).unwrap(),
            r#"{"permille":-10000}"#
        );
    }
}
