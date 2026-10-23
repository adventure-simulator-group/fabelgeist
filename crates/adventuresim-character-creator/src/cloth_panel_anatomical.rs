//! A retained native torso/shoulder chart preserves body ancestry.
//! Canonical unposed metre geometry is kept separate from the scalar region
//! chart used to crop the body; pose ownership does not waive collision checks.
use crate::armor_frames::{FitRegion, SKIN_SUPPORT_THRESHOLD, Side, Wearer};
use crate::surface_cut::{PanelRegion, Plane, SurfaceCut, connected_patch, interpolate};
use anyhow::{Context, Result};
use bevy::math::Vec3;
use fabelgeist_armor::{FittedSleeves, Millimeters, Permille};
use fabelgeist_rig::{RigJointName, RigJointPart};

/// The source reaches the humeral midpoint so its open boundary is a simple
/// upper-arm section rather than cutting across the axillary saddle.
const CARRIER_ARM_REACH_SHARE: Permille = Permille(500);
/// Source overlap below the lowest allowed hem, beyond the nominal gap range.
const CARRIER_HEM_SUPPORT_MARGIN: Millimeters = Millimeters(60);

pub(super) struct AnatomicalCarrier {
    pub source: SurfaceCut,
}

impl AnatomicalCarrier {
    pub fn new(
        body: &Wearer<'_>,
        hem_rise: Millimeters,
        sleeves: Option<&FittedSleeves>,
    ) -> Result<Self> {
        let landmark = |name: RigJointName| landmark(body, name);
        let hem = landmark(RigJointName::ROOT)?.y
            + if sleeves.is_some() {
                hem_rise.metres()
            } else {
                -CARRIER_HEM_SUPPORT_MARGIN.metres()
            };
        let chart = retained_chart(body, hem, sleeves)?;
        // Native neck affinity supplies continuation above the shoulder.
        // A horizontal cap through the shoulder can survive the final trim as
        // a raised tab; the physical neckline alone owns that boundary.
        let region = vec![Plane {
            normal: [-1., 0., 0.],
            offset: 0.,
        }];
        let partition =
            SurfaceCut::partition_panels(&chart, body.positions, body.faces, body.faces, &[region]);
        let mut source = partition.surface;
        source.faces = source
            .faces
            .into_iter()
            .zip(partition.triangles)
            .filter_map(|(face, owner)| (owner.region == PanelRegion::Panel(0)).then_some(face))
            .collect();
        let positions = source
            .points
            .iter()
            .map(|point| {
                interpolate(
                    body.faces[point.triangle].map(|v| body.positions[v as usize]),
                    point.weights,
                )
            })
            .collect::<Vec<_>>();
        let source = connected_patch(
            source,
            &positions,
            landmark(RigJointName::C_SPINE3)?.to_array(),
        )?;
        Ok(Self { source })
    }
}

/// Dimensionless bone-affinity chart; geometry stays in canonical metres.
fn retained_chart(
    body: &Wearer<'_>,
    hem: f32,
    sleeves: Option<&FittedSleeves>,
) -> Result<Vec<[f32; 3]>> {
    let landmark = |name: RigJointName| landmark(body, name);
    let torso =
        body.support_weights_for(FitRegion::Torso, body.joint_indices, body.joint_weights)?;
    let neck = body.support_weights_for(FitRegion::Neck, body.joint_indices, body.joint_weights)?;
    let hips = body.support_weights_for(FitRegion::Hips, body.joint_indices, body.joint_weights)?;
    let torso_span = landmark(RigJointName::C_NECK)?.y - hem;
    anyhow::ensure!(torso_span > 0., "cloth carrier has no torso span");
    let mut retained = torso
        .iter()
        .zip(neck)
        .zip(hips)
        .zip(body.positions)
        .map(|(((&a, b), hip), position)| {
            (a.max(b).max(hip) - SKIN_SUPPORT_THRESHOLD).min((position[1] - hem) / torso_span)
        })
        .collect::<Vec<_>>();
    for side in [Side::Right, Side::Left] {
        let shoulder = landmark(side.joint(RigJointPart::Uparm))?;
        let elbow = landmark(side.joint(RigJointPart::Lowarm))?;
        let axis = (elbow - shoulder)
            .try_normalize()
            .context("carrier arm has no axial extent")?;
        let wrist = landmark(side.joint(RigJointPart::Wrist))?;
        let upper_length = shoulder.distance(elbow);
        let lower_length = elbow.distance(wrist);
        let arm_length = upper_length + lower_length;
        let reach = sleeves.map_or(upper_length * CARRIER_ARM_REACH_SHARE.unit(), |s| {
            arm_length * s.length.unit()
        });
        let (cuff, cuff_axis) = if reach <= upper_length {
            (shoulder + axis * reach, axis)
        } else {
            let lower_axis = (wrist - elbow)
                .try_normalize()
                .context("carrier forearm has no length")?;
            (elbow + lower_axis * (reach - upper_length), lower_axis)
        };
        let region = if sleeves.is_some() {
            FitRegion::SleevedArm(side)
        } else {
            FitRegion::UpperArm(side)
        };
        let affinity = body.support_weights_for(region, body.joint_indices, body.joint_weights)?;
        for ((keep, &position), weight) in retained.iter_mut().zip(body.positions).zip(affinity) {
            // Both operands are dimensionless scalar chart coordinates;
            // normalizing axial reach by humerus length preserves units.
            let proximal = (cuff - Vec3::from_array(position)).dot(cuff_axis) / arm_length;
            *keep = keep.max((weight - SKIN_SUPPORT_THRESHOLD).min(proximal));
        }
    }
    let chart = body
        .positions
        .iter()
        .zip(retained)
        .map(|(p, keep)| [keep, p[1], p[2]])
        .collect::<Vec<_>>();
    Ok(chart)
}

fn landmark(body: &Wearer<'_>, name: RigJointName) -> Result<Vec3> {
    let index = name.require_in(body.joint_names)?;
    Ok(Vec3::from_slice(&body.joints[index][..3]))
}
