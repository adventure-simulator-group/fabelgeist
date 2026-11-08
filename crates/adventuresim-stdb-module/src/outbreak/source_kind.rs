use adventuresim_core::quest_generation::OutbreakSource;

pub(super) fn project(source: &OutbreakSource) -> (&'static str, Option<&str>) {
    match source {
        OutbreakSource::Sanitation { .. } => ("sanitation", None),
        OutbreakSource::Behavior { .. } => ("behavior", None),
        OutbreakSource::ThreatVector { threat } => ("threat_vector", Some(threat.as_str())),
        OutbreakSource::Environmental { .. } => ("environmental", None),
    }
}
