//! Tensioned leather between plates, with a straight seat for rigid hardware.
use bevy::math::Vec3;

/// Lift a sampled profile to its upper concave envelope. Unlike a bounded
/// smoothing pass, a taut span bridges a plate step at any sampling density.
pub(super) fn tension(profile: &mut [[f32; 2]]) {
    let mut hull: Vec<usize> = Vec::new();
    for i in 0..profile.len() {
        while hull.len() >= 2 {
            let a = hull[hull.len() - 2];
            let b = hull[hull.len() - 1];
            let t = (profile[b][0] - profile[a][0]) / (profile[i][0] - profile[a][0]);
            if profile[b][1] > profile[a][1] + t * (profile[i][1] - profile[a][1]) {
                break;
            }
            hull.pop();
        }
        hull.push(i);
    }
    for span in hull.windows(2) {
        let [a, b] = [profile[span[0]], profile[span[1]]];
        for p in &mut profile[span[0]..=span[1]] {
            p[1] = a[1] + (b[1] - a[1]) * (p[0] - a[0]) / (b[0] - a[0]);
        }
    }
}

pub(super) struct BuckleSeat {
    pub center: Vec3,
    pub slope: f32,
}

impl BuckleSeat {
    pub fn transform(&self, p: [f32; 3]) -> [f32; 3] {
        let down = Vec3::new(0.0, -1.0, -self.slope).normalize();
        let outward = Vec3::new(0.0, -self.slope, 1.0).normalize();
        (self.center + down * p[0] + Vec3::X * p[1] + outward * p[2]).to_array()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tension_bridges_steps_and_preserves_anchors_at_multiple_resolutions() {
        for rows in [8, 32, 64, 128] {
            for step in [0.005, 0.04, 0.1] {
                let mut profile = (0..=rows)
                    .map(|i| {
                        let t = i as f32 / rows as f32;
                        [1.0 - t, if t < 0.5 { step } else { 0.0 }]
                    })
                    .collect::<Vec<_>>();
                let original = profile.clone();
                tension(&mut profile);
                assert_eq!(profile[0], original[0]);
                assert_eq!(profile[rows], original[rows]);
                for (p, before) in profile.iter().zip(&original) {
                    assert!(p[1] + 1e-6 >= before[1]);
                }
                assert!(profile[rows / 2][1] > step * 0.5);
            }
        }
    }

    #[test]
    fn steep_seats_rotate_hardware_without_shearing_or_stretching_it() {
        for slope in [-3.0, -0.4, 0.0, 0.6, 3.0] {
            for width in [0.012, 0.018, 0.024] {
                let seat = BuckleSeat {
                    center: Vec3::new(0.2, 0.8, 0.18),
                    slope,
                };
                let hardware = super::super::mesh::buckle_shape(width);
                for pair in hardware.positions.windows(2) {
                    let before = Vec3::from_array(pair[0]).distance(Vec3::from_array(pair[1]));
                    let after = Vec3::from_array(seat.transform(pair[0]))
                        .distance(Vec3::from_array(seat.transform(pair[1])));
                    assert!((before - after).abs() < 1e-6);
                }
            }
        }
    }
}
