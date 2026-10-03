//! The gorget's angular chart: its rear trim swept back while every carrier
//! meridian stays fixed from neck to hem.
use std::f32::consts::TAU;

/// Fraction of the formed bib chart occupied by its integral collar band.
pub const GORGET_FORMED_COLLAR_FRACTION: f32 = 1.0 / 3.0;

/// The authored collar height, relative to the neck-to-head landmark span.
pub const GORGET_COLLAR_HEIGHT_NECK_RATIO: f32 = 0.20;

/// The chart angle of a physical angle around the neck, used to sample
/// anatomical sections at uniform angular resolution.
pub fn gorget_control_angle(physical: f32, sweep: crate::Permille) -> f32 {
    rear_angle(physical, 1.0 / (1.0 - 0.9 * sweep.unit()))
}

fn rear_angle(angle: f32, scale: f32) -> f32 {
    if angle.cos() >= 0.0 {
        return angle;
    }
    let rear = angle.rem_euclid(TAU) - std::f32::consts::PI;
    std::f32::consts::PI + (scale * rear.sin()).atan2(rear.cos().max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The physical angle of a chart angle: the rear trim swept back.
    fn gorget_surface_angle(control: f32, sweep: crate::Permille) -> f32 {
        rear_angle(control, 1.0 - 0.9 * sweep.unit())
    }

    #[test]
    fn rear_chart_and_inverse_preserve_order_and_side_boundaries() {
        use std::f32::consts::PI;
        for sweep in [0, 500, 850, 1000].map(crate::Permille) {
            let mut previous = 0.0;
            for index in 0..=256 {
                let control = PI * 0.5 + PI * index as f32 / 256.0;
                let physical = gorget_surface_angle(control, sweep);
                let restored = gorget_control_angle(physical, sweep);
                assert!((restored - control).abs() < 1e-5);
                assert!(physical >= previous - 1e-6);
                previous = physical;
                if index == 0 || index == 256 {
                    assert!((physical - control).abs() < 1e-5);
                }
            }
            for front in [-1.0, 0.0, 1.0] {
                assert_eq!(gorget_surface_angle(front, sweep), front);
            }
        }
    }
}
