//! Native Burn index tensors for the admitted rig's forward-kinematics scan.

use crate::skel_state::PrefixMultiplicationLevel;
use anyhow::Result;
use burn::tensor::{Device, Int, Tensor, TensorData};

/// Project rig ordinals into Burn's signed gather-index words at tensor upload.
/// Each level keeps the source/target order produced by the skeleton scan.
pub(super) fn fk_levels(
    levels: Vec<PrefixMultiplicationLevel>,
    device: &Device,
) -> Result<Vec<(Tensor<1, Int>, Tensor<1, Int>)>> {
    levels
        .into_iter()
        .map(|level| -> Result<_> {
            let len = level.source.len();
            let source: Vec<i32> = level
                .source
                .into_iter()
                .map(|joint| i32::try_from(usize::from(joint)))
                .collect::<Result<_, _>>()?;
            let target: Vec<i32> = level
                .target
                .into_iter()
                .map(|joint| i32::try_from(usize::from(joint)))
                .collect::<Result<_, _>>()?;
            Ok((
                Tensor::from_data(TensorData::new(source, [len]), device),
                Tensor::from_data(TensorData::new(target, [len]), device),
            ))
        })
        .collect()
}
