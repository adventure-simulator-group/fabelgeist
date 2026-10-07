//! The drape: how gravity and wind change a groom.
//!
//! DiffLocks' last step runs Blender's cloth simulation on the strands, with
//! the wind's direction and gravity's strength drawn at random, so that
//! samples of one style hang and blow differently. A dynamic simulation run
//! for a fixed number of steps is neither settled nor faithful to the
//! style; here each strand is solved to static equilibrium as an elastic
//! rod, and the drape is what *changes* between conditions.
//!
//! **The rod.** A strand is a chain of segments whose joints resist
//! bending away from the groomed shape. At each joint, the load of all the
//! hair beyond it -- gravity plus wind, as an acceleration -- makes a
//! moment `M`; the joint bends by `M l / (g L_g^3)` towards the load, where
//! `L_g` is the strand's gravity length (the length at which hair bends
//! under its own weight: about 5 cm for a single strand, more for a lock).
//! No joint bends past the load's direction. Moments come from suffix sums,
//! so a sweep is linear in the points; sweeps are repeated, relaxed, with
//! the body pushed out after each, until the strand is still.
//!
//! **The difference.** A groom is the shape hair has under ordinary
//! gravity. Solving it under the reference conditions and under the asked
//! ones, and adding the difference to the groom, leaves it unchanged at 9.81
//! m/s^2 and no wind, lifts it in weaker gravity, and blows it in wind --
//! without ever sagging a style into something it was not.

use fabelgeist_math::Vec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::collider::{Collider, push_out};
use crate::ops::{Context, restore_lengths};
use crate::strands::{Strands, rotate, safe_normalize, transport};

/// Standard gravity: the conditions a groom is shaped under.
pub const STANDARD_GRAVITY: f32 = 9.81;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settle {
    /// Metres per second squared, down.
    pub gravity: f32,
    /// Acceleration, metres per second squared.
    pub wind: [f32; 3],
    /// Gravity length `L_g`, metres: how long a strand bears its own
    /// weight.
    pub stiffness: f32,
    /// Sweeps to equilibrium.
    pub iterations: usize,
    /// Metres kept off the body.
    pub collision_offset: f32,
    /// Apply the change from standard gravity and no wind (true), or solve
    /// the groom as a rest shape under the conditions outright (false).
    pub relative: bool,
}

impl Default for Settle {
    fn default() -> Self {
        Self {
            gravity: STANDARD_GRAVITY,
            wind: [0.0; 3],
            stiffness: 0.06,
            iterations: 60,
            collision_offset: 0.003,
            relative: true,
        }
    }
}

impl Settle {
    pub fn apply(&self, strands: &mut Strands, context: &Context) {
        let points = strands.points;
        let collider = context.collider;
        let load = Vec3::new(0.0, -self.gravity, 0.0) + Vec3::from_array(self.wind);
        let reference = Vec3::new(0.0, -STANDARD_GRAVITY, 0.0);
        if self.relative && (load - reference).length() < 1e-4 {
            return;
        }
        strands.positions.par_chunks_mut(points).for_each(|strand| {
            let groom = strand.to_vec();
            let under = solve(&groom, load, self, collider);
            if self.relative {
                let base = solve(&groom, reference, self, collider);
                for ((p, a), b) in strand.iter_mut().zip(&under).zip(&base) {
                    *p += *a - *b;
                }
                restore_lengths(strand, &groom, 1.0);
                for p in strand.iter_mut().skip(1) {
                    *p = push_out(collider, *p, self.collision_offset * 0.5);
                }
            } else {
                strand.copy_from_slice(&under);
            }
        });
    }
}

/// The equilibrium of `rest` as an elastic rod under `load` (an
/// acceleration), rooted at its first point.
fn solve(rest: &[Vec3], load: Vec3, settings: &Settle, collider: &dyn Collider) -> Vec<Vec3> {
    let n = rest.len();
    let segments = n - 1;
    let lengths: Vec<f32> = rest.windows(2).map(|w| (w[1] - w[0]).length()).collect();
    let rest_dirs: Vec<Vec3> = rest
        .windows(2)
        .map(|w| safe_normalize(w[1] - w[0], Vec3::down()))
        .collect();
    let magnitude = load.length();
    if magnitude < 1e-6 {
        return rest.to_vec();
    }
    let force = load * (1.0 / magnitude);
    // Joint compliance: bend per unit moment per unit length.
    let compliance = 1.0 / (STANDARD_GRAVITY * settings.stiffness.max(1e-3).powi(3));

    let mut dirs = rest_dirs.clone();
    let mut points = rest.to_vec();
    let mut moment_arm_sum = vec![Vec3::default(); segments + 1];
    let mut length_sum = vec![0.0f32; segments + 1];
    for _ in 0..settings.iterations {
        // Suffix sums of each segment's centre times its length, and of the
        // lengths: the moment about point i of the load on everything past
        // it is (sum c_j l_j - x_i sum l_j) x load.
        moment_arm_sum[segments] = Vec3::default();
        length_sum[segments] = 0.0;
        for j in (0..segments).rev() {
            let centre = (points[j] + points[j + 1]) * 0.5;
            moment_arm_sum[j] = moment_arm_sum[j + 1] + centre * lengths[j];
            length_sum[j] = length_sum[j + 1] + lengths[j];
        }
        // Root to tip: each joint bends from its groomed angle -- carried
        // along as its parent turned -- towards the load.
        let mut position = points[0];
        for i in 0..segments {
            let groomed = if i == 0 {
                rest_dirs[0]
            } else {
                transport(rest_dirs[i], rest_dirs[i - 1], dirs[i - 1])
            };
            let moment = (moment_arm_sum[i] - points[i] * length_sum[i]).cross(load);
            let torque = moment.length();
            let mut target = groomed;
            if torque > 1e-12 {
                let axis = groomed.cross(force);
                let sin = axis.length();
                if sin > 1e-6 {
                    let limit = sin.atan2(groomed.dot(force));
                    let bend = (torque * compliance * lengths[i]).min(limit);
                    target = rotate(groomed, axis * (1.0 / sin), bend);
                }
            }
            let d = safe_normalize(dirs[i].lerp(target, 0.5), target);
            let mut next = position + d * lengths[i];
            // The body holds the hair up.
            if collider.distance(next) < settings.collision_offset {
                next = push_out(collider, next, settings.collision_offset);
                next = position + safe_normalize(next - position, d) * lengths[i];
            }
            dirs[i] = safe_normalize(next - position, d);
            points[i + 1] = next;
            position = next;
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collider::NoCollider;
    use crate::strands::Attachment;
    use crate::surface::Surface;

    fn horizontal(length: f32) -> Strands {
        let mut strands = Strands::new(40);
        let positions: Vec<Vec3> = (0..40)
            .map(|j| Vec3::new(j as f32 * length / 39.0, 0.0, 0.0))
            .collect();
        strands.push(&positions, Attachment::default());
        strands
    }

    fn scalp() -> Surface {
        Surface::new(
            vec![Vec3::default(), Vec3::right(), Vec3::forward()],
            None,
            None,
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn standard_gravity_leaves_the_groom_alone() {
        let mut strands = horizontal(0.3);
        let before = strands.clone();
        let scalp = scalp();
        let context = Context {
            scalp: &scalp,
            collider: &NoCollider,
        };
        Settle::default().apply(&mut strands, &context);
        assert_eq!(strands.positions, before.positions);
    }

    #[test]
    fn long_hair_sags_short_hair_holds() {
        let scalp = scalp();
        let context = Context {
            scalp: &scalp,
            collider: &NoCollider,
        };
        let absolute = Settle {
            relative: false,
            ..Default::default()
        };
        let mut long = horizontal(0.4);
        absolute.apply(&mut long, &context);
        let mut short = horizontal(0.02);
        absolute.apply(&mut short, &context);
        let long_tip = long.strand(0)[39];
        let short_tip = short.strand(0)[39];
        assert!(long_tip.y < -0.3, "{long_tip}");
        assert!(short_tip.y > -0.002, "{short_tip}");
        for w in long.strand(0).windows(2) {
            assert!(((w[1] - w[0]).length() - 0.4 / 39.0).abs() < 1e-4);
        }
    }

    #[test]
    fn wind_blows_hair_along_it() {
        let scalp = scalp();
        let context = Context {
            scalp: &scalp,
            collider: &NoCollider,
        };
        let mut strands = Strands::new(30);
        let hanging: Vec<Vec3> = (0..30)
            .map(|j| Vec3::new(0.0, -(j as f32) * 0.01, 0.0))
            .collect();
        strands.push(&hanging, Attachment::default());
        Settle {
            wind: [0.0, 0.0, 5.0],
            ..Default::default()
        }
        .apply(&mut strands, &context);
        assert!(strands.strand(0)[29].z > 0.05, "{}", strands.strand(0)[29]);
    }
}
