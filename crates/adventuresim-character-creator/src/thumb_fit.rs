//! Thumb plate anchors and dorsal orientation, independent of the finger fan.
use super::{
    MINIMUM_RADIAL_EXTENT_M, add, cross, dot, joint, local, normalize, prefix, scale, subtract,
};
use crate::armor_frames::{Side, Wearer};
use adventuresim_armor_model::PartFrame;
use anyhow::{Context, Result, ensure};

pub(super) fn thumb_frame(wearer: &Wearer<'_>, side: Side) -> Result<PartFrame> {
    let prefix = prefix(side);
    let root = joint(wearer, &format!("{prefix}_thumb1"))?;
    let tip = joint(wearer, &format!("{prefix}_thumb_null"))?;
    let axial = normalize(subtract(root, tip));
    let index = wearer
        .joint_names
        .iter()
        .position(|n| n == &format!("{prefix}_thumb3"))
        .context("missing distal thumb orientation")?;
    let q = &wearer.joints[index];
    // The canonical body's modeled thumbnail identifies local -Y on the left
    // and +Y on the right. Project that dorsal direction off the thumb axis.
    let sign = if matches!(side, Side::Left) {
        -1.0
    } else {
        1.0
    };
    let dorsal_axis = [
        2.0 * (q[3] * q[4] - q[5] * q[6]),
        1.0 - 2.0 * (q[3] * q[3] + q[5] * q[5]),
        2.0 * (q[4] * q[5] + q[3] * q[6]),
    ]
    .map(|v| v * sign);
    let dorsal = normalize(subtract(dorsal_axis, scale(axial, dot(dorsal_axis, axial))));
    let across = cross(axial, dorsal);
    let mut frame = PartFrame {
        origin: scale(add(root, tip), 0.5),
        axes: [across, axial, dorsal],
        half_extents: [
            MINIMUM_RADIAL_EXTENT_M,
            dot(subtract(root, tip), axial) * 0.5,
            MINIMUM_RADIAL_EXTENT_M,
        ],
    };
    let owners = wearer
        .joint_names
        .iter()
        .map(|n| n.starts_with(&format!("{prefix}_thumb")))
        .collect::<Vec<_>>();
    let mut low = [f32::INFINITY; 2];
    let mut high = [f32::NEG_INFINITY; 2];
    for (index, point) in wearer.positions.iter().enumerate() {
        let weight: f32 = wearer.joint_indices[index]
            .iter()
            .zip(wearer.joint_weights[index])
            .filter(|(i, _)| owners[**i as usize])
            .map(|(_, w)| w)
            .sum();
        if weight < 0.3 {
            continue;
        }
        let local = local(&frame, *point);
        // Thumb-zero weights include the palm heel beyond this plate's root.
        // They must not inflate the entire distal thumb envelope.
        if local[1] > frame.half_extents[1] {
            continue;
        }
        for (i, axis) in [0, 2].into_iter().enumerate() {
            low[i] = low[i].min(local[axis]);
            high[i] = high[i].max(local[axis]);
        }
    }
    ensure!(
        low.iter().chain(&high).all(|v| v.is_finite()),
        "no anatomical thumb envelope"
    );
    frame.origin = frame.point([(low[0] + high[0]) * 0.5, 0.0, (low[1] + high[1]) * 0.5]);
    frame.half_extents[0] = ((high[0] - low[0]) * 0.5).max(MINIMUM_RADIAL_EXTENT_M);
    frame.half_extents[2] = ((high[1] - low[1]) * 0.5).max(MINIMUM_RADIAL_EXTENT_M);
    frame.validate()?;
    Ok(frame)
}
