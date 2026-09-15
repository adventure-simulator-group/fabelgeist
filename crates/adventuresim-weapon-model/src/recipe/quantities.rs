//! Finite dimensional values preserve authored submillimetre precision.
use serde::{Deserialize, Deserializer, Serialize};
use std::hash::{Hash, Hasher};

macro_rules! quantity {
    ($name:ident,$documentation:literal) => {
        #[doc=$documentation]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
        #[serde(transparent)]
        pub struct $name(f64);
        impl $name {
            pub fn new(value: f64) -> Result<Self, &'static str> {
                if value.is_finite() {
                    Ok(Self(if value == 0.0 { 0.0 } else { value }))
                } else {
                    Err("quantity must be finite")
                }
            }
            pub fn get(self) -> f64 {
                self.0
            }
        }
        impl Eq for $name {}
        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.0.to_bits().hash(state);
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                Self::new(f64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }
    };
}
quantity!(
    Metres,
    "Signed geometric distance in metres; shape validation applies local bounds."
);
quantity!(Ratio, "Finite dimensionless construction ratio.");
quantity!(
    Degrees,
    "Finite rotation about an authored local axis, in degrees."
);
quantity!(Radians, "Finite angle in radians.");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Count(pub u16);
impl<'de> Deserialize<'de> for Count {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CountVisitor;
        impl serde::de::Visitor<'_> for CountVisitor {
            type Value = Count;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an integer sample count between 0 and 65535")
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Count, E> {
                u16::try_from(value)
                    .map(Count)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Unsigned(value), &self))
            }
        }
        deserializer.deserialize_u16(CountVisitor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(into = "i8")]
pub enum Direction {
    Negative,
    Positive,
}
impl Direction {
    pub fn sign(self) -> f64 {
        match self {
            Self::Negative => -1.0,
            Self::Positive => 1.0,
        }
    }
}
impl From<Direction> for i8 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Negative => -1,
            Direction::Positive => 1,
        }
    }
}
impl<'de> Deserialize<'de> for Direction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match i8::deserialize(deserializer)? {
            -1 => Ok(Self::Negative),
            1 => Ok(Self::Positive),
            _ => Err(serde::de::Error::custom("direction must be -1 or 1")),
        }
    }
}
