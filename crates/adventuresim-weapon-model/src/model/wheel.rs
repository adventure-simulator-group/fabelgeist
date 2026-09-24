//! One beveled wheel solid with a cut chord receiving its grip.
use super::*;

const WHEEL_RADIAL_SEGMENTS: usize = 48;

pub(super) fn construct(p: &WheelPommelParameters, detail: Detail) -> Result<Solid, String> {
    let radius = p.diameter.get() / 2.0;
    let face = p.face_diameter.get() / 2.0;
    let half = p.thickness.get() / 2.0;
    let rim = p.rim_thickness.get() / 2.0;
    Solid::lathe(
        &[[-half, face], [-rim, radius], [rim, radius], [half, face]],
        WHEEL_RADIAL_SEGMENTS,
        1.0,
        false,
        detail,
    )?
    .transform([90.0, 0.0, 0.0], [0.0; 3])
    .truncate_y(p.seat_height.get())
}
