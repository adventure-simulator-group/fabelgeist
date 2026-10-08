//! Signed displacement along a property frontage, distinct from its scene translation.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct FrontageDisplacement(f64);
impl FrontageDisplacement {
    pub fn from_metres(metres: f64) -> Option<Self> {
        metres.is_finite().then_some(Self(metres))
    }
    pub fn metres(self) -> f64 {
        self.0
    }
}

impl<'de> serde::Deserialize<'de> for FrontageDisplacement {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_metres(<f64 as serde::Deserialize>::deserialize(deserializer)?).ok_or_else(
            || serde::de::Error::custom("selected frontage displacement must be finite"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_displacement_decoding_cannot_admit_solver_infinity() {
        for coordinate in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let bytes = postcard::to_allocvec(&coordinate).unwrap();
            assert!(postcard::from_bytes::<FrontageDisplacement>(&bytes).is_err());
        }
        let position = FrontageDisplacement::from_metres(-2.5).unwrap();
        assert_eq!(serde_json::to_string(&position).unwrap(), "-2.5");
        assert_eq!(
            serde_json::from_str::<FrontageDisplacement>("-2.5").unwrap(),
            position
        );
    }
}
