//! Rig topology and checked network dimensions, before device allocation.
use super::{FEATURES_PER_JOINT, HIDDEN_PER_JOINT, SKIPPED_JOINTS};
use crate::Character;
use fabelgeist_numpy_storage::{NpyArray, NpyDimension, NpyShape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CorrectiveRigTopology {
    pub(super) joints: usize,
    pub(super) vertices: usize,
}
impl From<&Character> for CorrectiveRigTopology {
    fn from(character: &Character) -> Self {
        Self {
            joints: character.skeleton.len(),
            vertices: character.mesh.vertices.len(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CorrectiveNetworkDimensions {
    pub(super) posed_joints: usize,
    pub(super) vertices: usize,
    pub(super) hidden: usize,
    pub(super) inputs: usize,
    pub(super) output_coordinates: usize,
    pub(super) dense_entries: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorrectiveDimension {
    HiddenUnits,
    InputFeatures,
    OutputCoordinates,
    ActivationEntries,
}
impl TryFrom<CorrectiveRigTopology> for CorrectiveNetworkDimensions {
    type Error = super::CorrectiveDecodeError;
    fn try_from(rig: CorrectiveRigTopology) -> Result<Self, super::CorrectiveDecodeError> {
        let posed_joints = rig
            .joints
            .checked_sub(SKIPPED_JOINTS)
            .ok_or(super::CorrectiveDecodeError::SkeletonWithoutPose(rig))?;
        let hidden = posed_joints.checked_mul(HIDDEN_PER_JOINT).ok_or(
            super::CorrectiveDecodeError::DimensionOverflow(CorrectiveDimension::HiddenUnits),
        )?;
        let inputs = posed_joints.checked_mul(FEATURES_PER_JOINT).ok_or(
            super::CorrectiveDecodeError::DimensionOverflow(CorrectiveDimension::InputFeatures),
        )?;
        let output_coordinates =
            rig.vertices
                .checked_mul(3)
                .ok_or(super::CorrectiveDecodeError::DimensionOverflow(
                    CorrectiveDimension::OutputCoordinates,
                ))?;
        let dense_entries =
            inputs
                .checked_mul(hidden)
                .ok_or(super::CorrectiveDecodeError::DimensionOverflow(
                    CorrectiveDimension::ActivationEntries,
                ))?;
        Ok(Self {
            posed_joints,
            vertices: rig.vertices,
            hidden,
            inputs,
            output_coordinates,
            dense_entries,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedCorrectiveShape(NpyShape);
impl From<&NpyArray> for RejectedCorrectiveShape {
    fn from(array: &NpyArray) -> Self {
        Self(array.shape().clone())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CorrectiveBasisDimensions {
    pub(super) components: usize,
    vertices: usize,
}
impl TryFrom<&NpyArray> for CorrectiveBasisDimensions {
    type Error = super::CorrectiveDecodeError;
    fn try_from(array: &NpyArray) -> Result<Self, super::CorrectiveDecodeError> {
        match array.shape().dimensions() {
            [components, vertices, coordinates] if *coordinates == NpyDimension::from(3) => {
                Ok(Self {
                    components: usize::from(*components),
                    vertices: usize::from(*vertices),
                })
            }
            _ => Err(super::CorrectiveDecodeError::BasisShape(
                RejectedCorrectiveShape::from(array),
            )),
        }
    }
}
impl CorrectiveBasisDimensions {
    pub(super) fn admit_vertices(
        self,
        rig: CorrectiveRigTopology,
    ) -> Result<(), super::CorrectiveDecodeError> {
        if self.vertices != rig.vertices {
            Err(super::CorrectiveDecodeError::BasisVertices { basis: self, rig })
        } else {
            Ok(())
        }
    }
    pub(super) fn admit_components(
        self,
        network: CorrectiveNetworkDimensions,
    ) -> Result<(), super::CorrectiveDecodeError> {
        if self.components != network.hidden {
            Err(super::CorrectiveDecodeError::BasisComponents {
                basis: self,
                network,
            })
        } else {
            Ok(())
        }
    }
}
impl std::fmt::Display for CorrectiveRigTopology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} joints, {} mesh vertices", self.joints, self.vertices)
    }
}
impl std::fmt::Display for CorrectiveNetworkDimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} features, {} hidden units, {} vertices",
            self.inputs, self.hidden, self.vertices
        )
    }
}
impl std::fmt::Display for CorrectiveBasisDimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} components, {} basis vertices",
            self.components, self.vertices
        )
    }
}
impl std::fmt::Display for RejectedCorrectiveShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn short_skeletons_and_overflowing_dimensions_fail_before_allocation() {
        for joints in [0, 1] {
            assert!(matches!(
                CorrectiveNetworkDimensions::try_from(CorrectiveRigTopology {
                    joints,
                    vertices: 1,
                }),
                Err(super::super::CorrectiveDecodeError::SkeletonWithoutPose(_))
            ));
        }
        for (rig, expected) in [
            (
                CorrectiveRigTopology {
                    joints: usize::MAX,
                    vertices: 1,
                },
                CorrectiveDimension::HiddenUnits,
            ),
            (
                CorrectiveRigTopology {
                    joints: 3,
                    vertices: usize::MAX,
                },
                CorrectiveDimension::OutputCoordinates,
            ),
            (
                CorrectiveRigTopology {
                    joints: usize::MAX / 24,
                    vertices: 1,
                },
                CorrectiveDimension::ActivationEntries,
            ),
        ] {
            assert!(matches!(CorrectiveNetworkDimensions::try_from(rig),
                Err(super::super::CorrectiveDecodeError::DimensionOverflow(actual)) if actual==expected));
        }
        let empty = CorrectiveNetworkDimensions::try_from(CorrectiveRigTopology {
            joints: 2,
            vertices: 0,
        })
        .unwrap();
        assert_eq!(
            (
                empty.posed_joints,
                empty.hidden,
                empty.inputs,
                empty.dense_entries
            ),
            (0, 0, 0, 0)
        );
    }
}
