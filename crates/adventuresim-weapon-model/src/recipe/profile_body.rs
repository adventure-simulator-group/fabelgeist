//! Visible grip envelopes and a separately authored material cover.
use super::*;

pub(crate) const MIN_PROFILE_RADIAL_SEGMENTS: u16 = 12;

pub(crate) const MAX_BODY_PROFILE_STATIONS: usize = 12;
pub(crate) const MAX_PROFILE_RIBS: u16 = 64;

/// Rounded transverse courses cut inward from the authored crest envelope.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileRibs {
    pub count: Count,
    pub depth: Metres,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileStation {
    pub at: Ratio,
    pub width: Metres,
    pub depth: Metres,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileCover {
    pub material: Material,
    /// Inward distance normal to the transverse polygon's sides.
    pub thickness: Metres,
    /// Axial thickness closing the distal end; omission leaves both ends open.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub end_cap: Option<Metres>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileBodyParameters {
    pub length: Metres,
    pub profile: Vec<ProfileStation>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub ribs: Option<ProfileRibs>,
    /// Shared circumferential sampling for components with matching end rings.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub radial_segments: Option<Count>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub cover: Option<ProfileCover>,
}

impl ProfileBodyParameters {
    pub(crate) fn rib_inset(&self, t: f64) -> f64 {
        self.ribs.as_ref().map_or(0.0, |r| {
            if t == 0.0 || t == 1.0 {
                0.0
            } else {
                r.depth.get() * (std::f64::consts::PI * r.count.0 as f64 * t).sin().powi(2)
            }
        })
    }
    pub(crate) fn maximum_width(&self) -> f64 {
        self.profile
            .iter()
            .map(|p| p.width.get())
            .fold(0.0, f64::max)
    }
}
