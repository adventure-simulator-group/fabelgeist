//! Typed solver selections, separate from LP coefficients and table ordinals.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PackingDomainIndex(usize);
impl PackingDomainIndex {
    pub(super) fn new(index: usize) -> Self {
        Self(index)
    }
    pub(super) fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FrontagePosition(f64);
impl FrontagePosition {
    pub(super) fn from_metres(metres: f64) -> Option<Self> {
        metres.is_finite().then_some(Self(metres))
    }
    pub(super) fn metres(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PlanDisplacement(Vec2);
impl PlanDisplacement {
    pub(super) fn from_metres(metres: Vec2) -> Option<Self> {
        metres.is_finite().then_some(Self(metres))
    }
    pub(super) fn metres(self) -> Vec2 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub(super) struct FrontageSelection {
    pub(super) domain: PackingDomainIndex,
    pub(super) position: FrontagePosition,
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

#[derive(Clone, Copy)]
pub(super) struct ExploredSearchNodes(usize);
impl ExploredSearchNodes {
    pub(super) fn new(count: usize) -> Self {
        Self(count)
    }
    pub(super) fn count(self) -> usize {
        self.0
    }
}

pub(super) struct FrontageCoordinates(Vec<FrontagePosition>);
impl FrontageCoordinates {
    pub(super) fn from_solver(coordinates: Vec<f64>) -> Result<Self, CoupledPackingIssue> {
        coordinates
            .into_iter()
            .map(FrontagePosition::from_metres)
            .collect::<Option<Vec<_>>>()
            .map(Self)
            .ok_or(CoupledPackingIssue::NumericalFailure)
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = FrontagePosition> + '_ {
        self.0.iter().copied()
    }
    pub(super) fn into_selections(self) -> Vec<FrontageSelection> {
        self.0
            .into_iter()
            .enumerate()
            .map(|(index, position)| FrontageSelection {
                domain: PackingDomainIndex::new(index),
                position,
            })
            .collect()
    }
}
pub(super) struct CountedSearchOutcome {
    pub(super) outcome: Result<FrontageCoordinates, CoupledPackingIssue>,
    pub(super) explored_nodes: ExploredSearchNodes,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn solver_coordinates_reject_nonfinite_values_without_losing_signed_positions() {
        assert!(matches!(
            FrontageCoordinates::from_solver(vec![f64::NAN]),
            Err(CoupledPackingIssue::NumericalFailure)
        ));
        assert!(FrontageCoordinates::from_solver(vec![f64::INFINITY]).is_err());
        assert!(PlanDisplacement::from_metres(Vec2::splat(f32::NAN)).is_none());
        let selected = FrontageCoordinates::from_solver(vec![-2.0, 3.0])
            .unwrap()
            .into_selections();
        assert_eq!(selected[1].domain.index(), 1);
        assert_eq!(selected[0].position.metres(), -2.0);
    }
}
