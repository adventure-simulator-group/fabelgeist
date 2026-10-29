//! Corrective joint-tensor admission and its native rotation-feature kernel.

use std::{error::Error, fmt};

use burn::tensor::Tensor;
use fabelgeist_numpy_storage::{NpyDimension, NpyLayoutError, NpyTensorLayout};
use fabelgeist_rig::RigJointCount;

use super::{SKIPPED_JOINTS, import::CorrectiveLayout};
use crate::{character::PARAMETERS_PER_JOINT, model_def::JointChannel};

type Result<T> = std::result::Result<T, CorrectiveForwardError>;

/// Joint parameters admitted against the same layout used by the network.
/// Private construction retains the exact joint/channel match through feature
/// extraction. Values must use the model's channel order; admission checks
/// extents, not rig identity or value provenance.
pub(super) struct CorrectiveInput<'layout> {
    parameters: Tensor<3>,
    layout: &'layout CorrectiveLayout,
    batch: NpyDimension,
}

struct AxisSinCos {
    sine: Tensor<3>,
    cosine: Tensor<3>,
}

enum EulerAxis {
    X,
    Y,
    Z,
}

/// A rejected input shape or derived tensor layout in corrective evaluation.
#[derive(Debug)]
pub enum CorrectiveForwardError {
    JointCount {
        expected: RigJointCount,
        actual: RigJointCount,
    },
    Channels {
        expected: NpyDimension,
        actual: NpyDimension,
    },
    Layout {
        source: NpyLayoutError,
    },
}

impl<'layout> CorrectiveInput<'layout> {
    pub(super) fn new(parameters: Tensor<3>, layout: &'layout CorrectiveLayout) -> Result<Self> {
        let [batch, joints, channels] = parameters.dims();
        let actual = RigJointCount::from_native_joint_count(joints);
        let expected = layout.joints();
        if actual != expected {
            return Err(CorrectiveForwardError::JointCount { expected, actual });
        }
        let actual = NpyDimension::from(channels);
        let expected = NpyDimension::from(PARAMETERS_PER_JOINT);
        if actual != expected {
            return Err(CorrectiveForwardError::Channels { expected, actual });
        }
        Ok(Self {
            parameters,
            layout,
            batch: NpyDimension::from(batch),
        })
    }

    pub(super) fn output_dimensions(&self) -> Result<NpyTensorLayout<3>> {
        Ok(self.layout.output_dimensions(self.batch)?)
    }

    /// The first two rotation-matrix columns, with the rest-pose diagonals
    /// removed, in `[batch, posed * 6]` order. The native tensor was admitted
    /// against this layout before any joint or channel slice is evaluated.
    pub(super) fn into_features(self) -> Result<Tensor<2>> {
        let feature_dimensions = self.layout.feature_dimensions(self.batch)?;
        let euler = self
            .parameters
            .narrow(
                1,
                usize::from(SKIPPED_JOINTS),
                usize::from(self.layout.posed_joints()),
            )
            .narrow(
                2,
                JointChannel::ROTATIONS[0].native_index(),
                JointChannel::ROTATIONS.len(),
            );

        let AxisSinCos {
            sine: sx,
            cosine: cx,
        } = EulerAxis::X.sin_cos(&euler);
        let AxisSinCos {
            sine: sy,
            cosine: cy,
        } = EulerAxis::Y.sin_cos(&euler);
        let AxisSinCos {
            sine: sz,
            cosine: cz,
        } = EulerAxis::Z.sin_cos(&euler);

        let feature = Tensor::cat(
            vec![
                (cy.clone() * cz.clone()).sub_scalar(1.0),
                cy.clone() * sz.clone(),
                sy.clone().neg(),
                cx.clone().neg() * sz.clone() + sx.clone() * sy.clone() * cz.clone(),
                (cx * cz + sx.clone() * sy * sz).sub_scalar(1.0),
                sx * cy,
            ],
            2,
        );
        Ok(feature.reshape(feature_dimensions))
    }
}

impl EulerAxis {
    fn sin_cos(self, euler: &Tensor<3>) -> AxisSinCos {
        let channel = match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        };
        let angle = euler.clone().narrow(2, channel, 1);
        AxisSinCos {
            sine: angle.clone().sin(),
            cosine: angle.cos(),
        }
    }
}

impl fmt::Display for CorrectiveForwardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JointCount { expected, actual } => write!(
                formatter,
                "corrective input has {actual} joints, expected {expected}"
            ),
            Self::Channels { expected, actual } => write!(
                formatter,
                "corrective input has {actual} channels per joint, expected {expected}"
            ),
            Self::Layout { .. } => formatter.write_str("invalid corrective tensor layout"),
        }
    }
}

impl Error for CorrectiveForwardError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Layout { source } => Some(source),
            _ => None,
        }
    }
}

impl From<NpyLayoutError> for CorrectiveForwardError {
    fn from(source: NpyLayoutError) -> Self {
        Self::Layout { source }
    }
}
