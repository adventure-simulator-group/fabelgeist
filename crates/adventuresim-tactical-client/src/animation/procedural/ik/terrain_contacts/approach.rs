//! Constant-speed acquisition of an upcoming run contact.

use adventuresim_tactical_core::animation::CycleDelta;

pub(in crate::animation::procedural) fn run_contact_approach_progress(
    phase_to_contact: CycleDelta,
    approach_window: f32,
    contact_ready_phase: f32,
) -> f32 {
    // The retained cycle displacement meets configured native window values
    // only at this dimensionless acquisition-ratio kernel.
    let phase_to_contact = phase_to_contact.cycles();
    if approach_window <= contact_ready_phase {
        return f32::from(phase_to_contact <= contact_ready_phase);
    }
    let linear = ((approach_window - phase_to_contact) / (approach_window - contact_ready_phase))
        .clamp(0.0, 1.0);
    // Constant horizontal velocity avoids both the old smoothstep mid-swing
    // spike and a one-sample ease/catch-up seam. Clearance remains a separate
    // sine arc, so the endpoint arrives early enough for bounded acquisition.
    linear
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collapsed_contact_window_is_a_step_and_small_windows_interpolate() {
        assert_eq!(
            run_contact_approach_progress(CycleDelta::from_cycles(0.3).unwrap(), 0.2, 0.2),
            0.0
        );
        assert_eq!(
            run_contact_approach_progress(CycleDelta::from_cycles(0.2).unwrap(), 0.2, 0.2),
            1.0
        );
        assert_eq!(
            run_contact_approach_progress(CycleDelta::from_cycles(0.1).unwrap(), 0.2, 0.2),
            1.0
        );
        assert_eq!(
            run_contact_approach_progress(CycleDelta::from_cycles(0.5e-10).unwrap(), 1.0e-10, 0.0),
            0.5
        );
    }
}
