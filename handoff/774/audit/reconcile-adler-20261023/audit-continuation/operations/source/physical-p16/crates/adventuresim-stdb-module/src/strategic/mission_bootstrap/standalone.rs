//! Standalone mission identity and the location of its diagnostic case site.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StandaloneMissionFamily {
    Animation,
    Diagnostic,
    General,
}

const ANIMATION_MISSION_COORDINATE_PREFIX: &str = "animation-";
const DIAGNOSTIC_MISSION_COORDINATE_PREFIX: &str = "diagnostic-";
const STANDALONE_DIAGNOSTIC_SITE_DISTANCE_M: u64 = 2_000;
const METERS_PER_GEOGRAPHIC_LATITUDE_DEGREE: f64 = 111_000.0;
const METERS_PER_UNBOUNDED_COORDINATE_UNIT: f64 = 1_000.0;

impl StandaloneMissionFamily {
    pub(super) fn from_mission_id(mission_id: &str) -> Result<Self, String> {
        let mission_id = adventuresim_core::mission::MissionId::new(mission_id)
            .map_err(|_| "Standalone mission ID has an invalid domain coordinate".to_owned())?;
        let (_, coordinate) = mission_id
            .as_str()
            .split_once(':')
            .expect("validated mission IDs contain a domain separator");
        if let Some(animation_coordinate) =
            coordinate.strip_prefix(ANIMATION_MISSION_COORDINATE_PREFIX)
        {
            if animation_coordinate.is_empty() {
                return Err("Animation mission ID has no coordinate".into());
            }
            Ok(Self::Animation)
        } else if let Some(diagnostic_coordinate) =
            coordinate.strip_prefix(DIAGNOSTIC_MISSION_COORDINATE_PREFIX)
        {
            if diagnostic_coordinate.is_empty() {
                return Err("Diagnostic mission ID has no coordinate".into());
            }
            Ok(Self::Diagnostic)
        } else {
            Ok(Self::General)
        }
    }

    pub(super) fn case_site_distance_m(self) -> u64 {
        match self {
            Self::Diagnostic => STANDALONE_DIAGNOSTIC_SITE_DISTANCE_M,
            Self::Animation | Self::General => 0,
        }
    }
}

pub(super) fn standalone_case_site_northward_offset(
    distance_m: u64,
    coordinates_are_geographic: bool,
) -> f64 {
    let coordinate_unit_m = if coordinates_are_geographic {
        METERS_PER_GEOGRAPHIC_LATITUDE_DEGREE
    } else {
        METERS_PER_UNBOUNDED_COORDINATE_UNIT
    };
    distance_m as f64 / coordinate_unit_m
}

pub(super) fn standalone_case_id(mission_id: &str) -> String {
    format!("case:standalone:{mission_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standalone_mission_family_uses_an_exact_mission_tag() {
        assert_eq!(
            StandaloneMissionFamily::from_mission_id("mission:animation-demo").unwrap(),
            StandaloneMissionFamily::Animation
        );
        assert_eq!(
            StandaloneMissionFamily::from_mission_id("mission:ordinary").unwrap(),
            StandaloneMissionFamily::General
        );
        assert_eq!(
            StandaloneMissionFamily::from_mission_id("mission:diagnostic-demo").unwrap(),
            StandaloneMissionFamily::Diagnostic
        );
        for invalid in [
            "mission:",
            "mission:animation-",
            "mission:diagnostic-",
            "case:animation-demo",
            "animation-demo",
        ] {
            assert!(StandaloneMissionFamily::from_mission_id(invalid).is_err());
        }
    }

    #[test]
    fn diagnostic_standalone_missions_create_wilderness_case_sites() {
        assert_eq!(
            StandaloneMissionFamily::Diagnostic.case_site_distance_m(),
            STANDALONE_DIAGNOSTIC_SITE_DISTANCE_M
        );
        assert_eq!(StandaloneMissionFamily::Animation.case_site_distance_m(), 0);
        assert_eq!(
            standalone_case_site_northward_offset(
                STANDALONE_DIAGNOSTIC_SITE_DISTANCE_M,
                true,
            ),
            STANDALONE_DIAGNOSTIC_SITE_DISTANCE_M as f64
                / METERS_PER_GEOGRAPHIC_LATITUDE_DEGREE
        );
    }
}
