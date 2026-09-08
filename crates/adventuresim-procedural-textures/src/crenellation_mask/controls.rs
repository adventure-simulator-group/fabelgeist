//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        crenellation_merlon_duty_cycle: f32 = CRENELLATION_MERLON_DUTY_CYCLE;
        crenellation_breastwork_height_ratio: f32 = CRENELLATION_BREASTWORK_HEIGHT_RATIO;
        masonry_rgb: [u8; 3] = MASONRY_RGB;
    }
}
