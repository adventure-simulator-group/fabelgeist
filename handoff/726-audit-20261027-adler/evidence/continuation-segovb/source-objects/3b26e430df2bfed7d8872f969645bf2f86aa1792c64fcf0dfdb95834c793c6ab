//! Convert durable corpse timestamps into the shared autopsy time context.

use adventuresim_core::autopsy::{
    AutopsyEvidenceContext, CorpseLocation, corpse_location, decomposition_band,
};
use adventuresim_world_schema::calendar::StrategicMinute;

use super::StrategicCorpse;

pub(super) fn autopsy_evidence_context(
    corpse: &StrategicCorpse,
    minute: StrategicMinute,
) -> AutopsyEvidenceContext {
    let location = corpse_location(
        corpse.discovered_minute,
        minute,
        corpse.buried,
        corpse.exhumed,
    );
    AutopsyEvidenceContext {
        decomposition: decomposition_band(corpse.death_minute, minute, corpse.handling_damage_bps),
        at_scene: location == CorpseLocation::Scene,
        opening_obscuration_bps: corpse.opening_obscuration_bps,
    }
}
