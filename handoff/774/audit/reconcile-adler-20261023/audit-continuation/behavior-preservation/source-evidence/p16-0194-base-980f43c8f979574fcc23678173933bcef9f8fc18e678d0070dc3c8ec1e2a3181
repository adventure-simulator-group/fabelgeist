//! Validate persisted treatment rows before using strategic physiology.

use super::PhysiologyAdministration;
use adventuresim_core::physiology::{self, Administration, DoseMilliunits};

pub(super) fn administration(row: &PhysiologyAdministration) -> Result<Administration, String> {
    if row.ruleset_version != physiology::PHYSIOLOGY_RULESET_VERSION {
        return Err(format!(
            "Unsupported intervention ruleset version {}",
            row.ruleset_version
        ));
    }
    Ok(Administration {
        id: row.id,
        patient_id: row.patient_id,
        preparation_id: row.preparation_id.clone(),
        profile_version: row.profile_version,
        route: row.route,
        dose: DoseMilliunits::try_new(row.dose_milliunits)
            .map_err(|_| "Persisted intervention dose exceeds the supported maximum")?,
        region: row.region,
        administered_at: row.administered_at,
        stopped_at: row.stopped_at,
        sensitivity_bps: row.sensitivity_bps,
        adverse_bps: row.adverse_bps,
    })
}
