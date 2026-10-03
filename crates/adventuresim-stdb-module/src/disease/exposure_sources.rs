//! Stable settlement disease-source ordering, bounds, and storage validation.
use super::*;

pub(super) struct SettlementExposureSource {
    pub id: String,
    pub disease_id: DiseaseId,
    pub start: StrategicMinute,
    pub end: StrategicMinute,
    pub intensity: f32,
    pub scoped: bool,
}

impl SettlementExposureSource {
    pub(super) fn for_settlement(
        ctx: &ReducerContext,
        settlement_id: &String,
    ) -> Result<Vec<Self>, String> {
        let mut outbreaks = ctx
            .db
            .settlement_outbreak()
            .settlement_id()
            .filter(settlement_id)
            .collect::<Vec<_>>();
        outbreaks.sort_by(|left, right| {
            (left.start_minute, left.id.as_str()).cmp(&(right.start_minute, right.id.as_str()))
        });
        let scope_key = format!("settlement:{settlement_id}");
        let mut problems = ctx
            .db
            .local_problem_authority()
            .scope_key()
            .filter(&scope_key)
            .filter(|row| {
                !row.disease_id.is_empty()
                    && row.disease_intensity > 0
                    && row.mitigation_bps < adventuresim_world_schema::BASIS_POINTS_PER_WHOLE
            })
            .collect::<Vec<_>>();
        problems.sort_by(|left, right| left.id.cmp(&right.id));
        problems.truncate(adventuresim_core::local_problem::MAX_ACTIVE_PER_SCOPE);
        outbreaks
            .into_iter()
            .map(|row| {
                Ok(Self {
                    id: row.id,
                    disease_id: row
                        .disease_id
                        .parse::<DiseaseId>()
                        .map_err(|error| error.to_string())?,
                    start: row.start_minute,
                    end: row.end_minute,
                    intensity: row.intensity,
                    scoped: false,
                })
            })
            .chain(problems.into_iter().map(|row| {
                Ok(Self {
                    id: row.id,
                    disease_id: row
                        .disease_id
                        .parse::<DiseaseId>()
                        .map_err(|error| error.to_string())?,
                    start: row.starts_at,
                    end: row
                        .ends_at
                        .min(row.resolved_at.unwrap_or(StrategicMinute::MAX)),
                    intensity: adventuresim_core::local_problem::mitigated_disease_exposure(
                        row.disease_intensity,
                        adventuresim_world_schema::UnitBasisPoints::new(row.mitigation_bps)
                            .expect("filtered mitigation is a unit basis-point value"),
                    ),
                    scoped: true,
                })
            }))
            .collect()
    }
}
