//! Typed solver selections, separate from LP coefficients and table ordinals.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(super) struct PackingDomainIndex(usize);

#[derive(Clone, Copy)]
pub(super) struct FrontageSelection {
    pub(super) domain: PackingDomainIndex,
    pub(super) frontage_displacement: FrontageDisplacement,
}
#[derive(Clone, Copy)]
pub(super) struct PropertyTranslation {
    pub(super) domain: PackingDomainIndex,
    pub(super) displacement: PlanDisplacement,
}
#[derive(Clone)]
pub(super) struct DomainChoices {
    pub(super) domain: PackingDomainIndex,
    pub(super) intervals: Vec<FrontageInterval>,
}

pub(super) struct FrontageDisplacements(Vec<FrontageDisplacement>);
pub(super) struct CountedSearchOutcome {
    pub(super) outcome: Result<FrontageDisplacements, CoupledPackingIssue>,
    pub(super) explored_nodes: ExploredSearchNodes,
}
impl PackingDomainIndex {
    pub(super) fn new(index: usize) -> Self {
        Self(index)
    }
    pub(super) fn index(self) -> usize {
        self.0
    }
}
impl FrontageDisplacements {
    pub(super) fn from_solver(coordinates: Vec<f64>) -> Result<Self, CoupledPackingIssue> {
        coordinates
            .into_iter()
            .map(FrontageDisplacement::from_metres)
            .collect::<Option<Vec<_>>>()
            .map(Self)
            .ok_or(CoupledPackingIssue::NumericalFailure)
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = FrontageDisplacement> + '_ {
        self.0.iter().copied()
    }
    pub(super) fn into_selections(self) -> Vec<FrontageSelection> {
        self.0
            .into_iter()
            .enumerate()
            .map(|(index, frontage_displacement)| FrontageSelection {
                domain: PackingDomainIndex::new(index),
                frontage_displacement,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn solver_coordinates_reject_nonfinite_values_without_losing_signed_positions() {
        assert!(matches!(
            FrontageDisplacements::from_solver(vec![f64::NAN]),
            Err(CoupledPackingIssue::NumericalFailure)
        ));
        assert!(FrontageDisplacements::from_solver(vec![f64::INFINITY]).is_err());
        assert!(PlanDisplacement::from_metres(Vec2::splat(f32::NAN)).is_none());
        let selected = FrontageDisplacements::from_solver(vec![-2.0, 3.0])
            .unwrap()
            .into_selections();
        assert_eq!(selected[1].domain.index(), 1);
        assert_eq!(selected[0].frontage_displacement.metres(), -2.0);
    }
}
