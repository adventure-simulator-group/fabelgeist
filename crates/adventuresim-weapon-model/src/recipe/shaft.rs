//! A shaft's body taper and optional local receiving tenon share one profile.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShaftTenon {
    /// Portion of the total shaft length turned down at its upper end.
    pub length: Metres,
    pub tip_radius: Metres,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WrappingPattern {
    RightHanded,
    LeftHanded,
    Crossed,
}

/// A flat strip winding around the actual tapered shaft surface. Width is its
/// axial footprint; start and length include the strip's square ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShaftWrapping {
    /// Earlier wrapping whose outer crest supports this layer across its gaps.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub on_wrapping: Option<Count>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub section: Option<WrappingSection>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub underlay: Option<WrappingUnderlay>,
    pub start: Metres,
    pub length: Metres,
    pub pitch: Metres,
    pub width: Metres,
    pub thickness: Metres,
    pub phase: Degrees,
    pub pattern: WrappingPattern,
    pub material: Material,
}

/// Compressed cord has a flat receiving underside and an authored crest width.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum WrappingSection {
    Rounded {
        #[serde(rename = "crestFraction")]
        crest_fraction: Ratio,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WrappingUnderlay {
    pub thickness: Metres,
    pub material: Material,
}

impl Shaft {
    pub(crate) fn profile(&self) -> Vec<[f64; 2]> {
        let mut result = vec![
            [
                0.0,
                self.radius.get() * self.bottom_scale.map_or(1.0, Ratio::get),
            ],
            [
                self.length.get() - self.tenon.as_ref().map_or(0.0, |t| t.length.get()),
                self.radius.get() * self.top_scale.map_or(0.92, Ratio::get),
            ],
        ];
        if let Some(tenon) = &self.tenon {
            result.push([self.length.get(), tenon.tip_radius.get()]);
        }
        result
    }

    pub(crate) fn radius_at(&self, height: f64) -> f64 {
        let profile = self.profile();
        let span = profile
            .windows(2)
            .find(|p| height <= p[1][0])
            .unwrap_or(&profile[profile.len() - 2..]);
        let progress = ((height - span[0][0]) / (span[1][0] - span[0][0])).clamp(0.0, 1.0);
        span[0][1] + (span[1][1] - span[0][1]) * progress
    }
}
