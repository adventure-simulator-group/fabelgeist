//! Body-surface skin transfer for sewn and procedural textile meshes.
//!
//! Positions are metres in the canonical unposed fitting frame, with Y up.
//! Joint rows retain the native eight-influence mesh representation for upload;
//! rig names remain typed through anatomical lookup.

use anyhow::{Context, Result, ensure};
use fabelgeist_bvh::TriangleBvh;
use fabelgeist_math::Vec3;
use fabelgeist_rig::{PackedRigSkin, RigJointName, RigJointOrdinal, RigJointTable};
use std::collections::BTreeMap;

/// Prevent a nearly coincident hip pair from making the blend singular.
const MINIMUM_HIP_HALF_SPAN_M: f32 = 0.025;
/// A squared triangle-area measure in metres to the fourth power.
const MINIMUM_BARYCENTRIC_DENOMINATOR_M4: f32 = 1e-15;

/// Skin rows ready for the mesh/upload boundary, in garment vertex order.
#[derive(Clone)]
pub struct SkinWeights {
    pub indices: PackedRigSkin<8>,
    pub weights: Vec<[f32; 8]>,
}

/// The canonical body surface and its rig, borrowed by either clothing path.
pub struct SkinSource<'a> {
    pub positions: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
    pub indices: &'a PackedRigSkin<8>,
    pub weights: &'a [[f32; 8]],
    pub names: &'a RigJointTable<RigJointName>,
    pub joints: &'a RigJointTable<[f32; 8]>,
}

struct SkinRow {
    indices: [RigJointOrdinal; 8],
    weights: [f32; 8],
}

impl SkinSource<'_> {
    /// Interpolate the nearest body triangle's skin, rather than copying a
    /// single vertex's rows. UVs belong to the garment's own construction.
    pub fn transfer(&self, positions: &[[f32; 3]]) -> Result<SkinWeights> {
        self.validate()?;
        let tree = TriangleBvh::new(
            self.positions.iter().copied().map(vector).collect(),
            self.faces.to_vec(),
        );
        let mut output = SkinWeights {
            indices: PackedRigSkin::empty(self.indices.joint_count()),
            weights: Vec::with_capacity(positions.len()),
        };
        for &position in positions {
            ensure!(
                position.iter().all(|v| v.is_finite()),
                "non-finite skin query"
            );
            let (triangle, point, _) = tree
                .closest_point(vector(position), f32::MAX)
                .context("no body surface for garment skin transfer")?;
            let face = self.faces[triangle as usize];
            let [a, b, c] = face.map(|i| vector(self.positions[i as usize]));
            let bary = barycentric(a, b, c, point);
            let mut influences = BTreeMap::new();
            for (vertex, factor) in face.into_iter().zip(bary) {
                accumulate(
                    &mut influences,
                    self.indices.vertex(vertex as usize),
                    self.weights[vertex as usize],
                    factor.max(0.0),
                );
            }
            let row = normalized(influences)?;
            output.indices.push(row.indices)?;
            output.weights.push(row.weights);
        }
        Ok(output)
    }

    /// Blend the two upper-leg joints below the waist. The lateral axis
    /// follows the hip pair, so turning the canonical body around Y does not
    /// change which leg owns a side. Knees, feet and surface twist helpers do not
    /// drive a continuous skirt: copying their local body ownership creates
    /// abrupt creases on fabric far from the body. This is skinning, not
    /// collision response.
    pub fn skirt(&self, positions: &[[f32; 3]]) -> Result<SkinWeights> {
        let left = self.landmark(&RigJointName::L_UPLEG)?;
        let right = self.landmark(&RigJointName::R_UPLEG)?;
        let center = (left + right) * 0.5;
        let span = Vec3::new(right.x - left.x, 0.0, right.z - left.z);
        let half_span = span.length() * 0.5;
        ensure!(half_span > 0.0, "skirt hip landmarks coincide");
        let axis = span / span.length();
        let half_span = half_span.max(MINIMUM_HIP_HALF_SPAN_M);
        let mut skin = self.transfer(positions)?;
        let knee_center = (self.landmark(&RigJointName::L_LOWLEG)?
            + self.landmark(&RigJointName::R_LOWLEG)?)
            * 0.5;
        let thigh_height = center.y - knee_center.y;
        ensure!(thigh_height > 0.0, "skirt knees must be below its hips");
        // Spread ownership over the cloth's own width. A hip-width transition
        // on a wide hem creates a narrow strip of extreme animation shear.
        let cloth_half_span = positions
            .iter()
            .filter(|p| p[1] <= center.y)
            .map(|&p| (vector(p) - center).dot(axis).abs())
            .fold(half_span, f32::max);
        let pelvis = RigJointName::ROOT.require_in(self.names)?;
        let left_joint = RigJointName::L_UPLEG.require_in(self.names)?;
        let right_joint = RigJointName::R_UPLEG.require_in(self.names)?;
        for (vertex, &position) in positions.iter().enumerate() {
            let position = vector(position);
            let lateral = (position - center).dot(axis);
            if position.y >= center.y + half_span {
                continue;
            }
            let horizontal = smooth((lateral + cloth_half_span) / (2.0 * cloth_half_span));
            // The waist is suspended from the pelvis. Introduce leg motion
            // gradually over the thigh rather than folding it into a narrow
            // band above the hip. No nearest-leg skin remains below the hips.
            let surface = smooth((position.y - center.y) / half_span);
            let legs = smooth((center.y - position.y) / thigh_height);
            let mut influences = BTreeMap::new();
            accumulate(
                &mut influences,
                skin.indices.vertex(vertex),
                skin.weights[vertex],
                surface,
            );
            *influences.entry(pelvis).or_default() += (1.0 - surface) * (1.0 - legs);
            *influences.entry(left_joint).or_default() += legs * (1.0 - horizontal);
            *influences.entry(right_joint).or_default() += legs * horizontal;
            let row = normalized(influences)?;
            skin.indices.set_vertex(vertex, row.indices)?;
            skin.weights[vertex] = row.weights;
        }
        Ok(skin)
    }

    fn landmark(&self, name: &RigJointName) -> Result<Vec3> {
        let slot = name.require_in(self.names)?;
        let state = self.joints.get(slot).context("missing hip transform")?;
        ensure!(
            state[..3].iter().all(|v| v.is_finite()),
            "non-finite hip landmark"
        );
        Ok(Vec3::new(state[0], state[1], state[2]))
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            self.names.len() == self.joints.len()
                && self.indices.joint_count() == self.names.joint_count()
                && self
                    .indices
                    .iter()
                    .zip(self.weights)
                    .all(|(indices, weights)| {
                        indices
                            .iter()
                            .zip(weights)
                            .all(|(&joint, _)| usize::from(joint) < self.names.len())
                    }),
            "body skin references a missing rig joint"
        );
        ensure!(
            self.indices.len() == self.positions.len()
                && self.weights.len() == self.positions.len(),
            "body skin arrays do not match its vertices"
        );
        ensure!(
            self.faces
                .iter()
                .flatten()
                .all(|&i| (i as usize) < self.positions.len())
                && self.positions.iter().flatten().all(|v| v.is_finite())
                && self
                    .weights
                    .iter()
                    .flatten()
                    .all(|v| v.is_finite() && *v >= 0.0),
            "invalid body surface for garment skin transfer"
        );
        Ok(())
    }
}

fn normalized(influences: BTreeMap<RigJointOrdinal, f32>) -> Result<SkinRow> {
    let mut influences: Vec<_> = influences.into_iter().filter(|(_, w)| *w > 0.0).collect();
    influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    influences.truncate(8);
    let sum: f32 = influences.iter().map(|(_, w)| w).sum();
    ensure!(
        sum.is_finite() && sum > 0.0,
        "garment vertex has no skin weights"
    );
    let mut row = SkinRow {
        indices: [RigJointOrdinal::from(0usize); 8],
        weights: [0.0; 8],
    };
    for (slot, (joint, weight)) in influences.into_iter().enumerate() {
        row.indices[slot] = joint;
        row.weights[slot] = weight / sum;
    }
    Ok(row)
}

fn accumulate(
    output: &mut BTreeMap<RigJointOrdinal, f32>,
    indices: [RigJointOrdinal; 8],
    weights: [f32; 8],
    factor: f32,
) {
    for (joint, weight) in indices.into_iter().zip(weights) {
        *output.entry(joint).or_default() += weight * factor;
    }
}

fn barycentric(a: Vec3, b: Vec3, c: Vec3, point: Vec3) -> [f32; 3] {
    let (v0, v1, v2) = (b - a, c - a, point - a);
    let (d00, d01, d11, d20, d21) = (v0.dot(v0), v0.dot(v1), v1.dot(v1), v2.dot(v0), v2.dot(v1));
    let denominator = d00 * d11 - d01 * d01;
    if denominator.abs() <= MINIMUM_BARYCENTRIC_DENOMINATOR_M4 {
        return [1.0, 0.0, 0.0];
    }
    let v = (d11 * d20 - d01 * d21) / denominator;
    let w = (d00 * d21 - d01 * d20) / denominator;
    [1.0 - v - w, v, w]
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn vector(p: [f32; 3]) -> Vec3 {
    Vec3::new(p[0], p[1], p[2])
}
