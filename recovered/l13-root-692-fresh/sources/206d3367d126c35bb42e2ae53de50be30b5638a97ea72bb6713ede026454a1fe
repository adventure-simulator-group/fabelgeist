//! Projection axes, screen placement and occupied pixel bounds.
use super::*;
pub(super) fn raw_coordinates(point: [f32; 3], layout: WeaponIconLayout) -> [f32; 2] {
    // A slight deterministic quarter view keeps transverse furniture legible.
    let yaw = render::CAMERA_YAW;
    let lateral = point[0] * yaw.cos() + point[2] * yaw.sin();
    let axial = match layout {
        WeaponIconLayout::HiltFocus => point[1],
        WeaponIconLayout::HeadFocus => -point[1],
    };
    [lateral, axial]
}

pub(super) fn relative_screen(
    point: [f32; 2],
    center: [f32; 2],
    layout: WeaponIconLayout,
    mirror_x: bool,
    flip_lateral: bool,
) -> [f32; 2] {
    let mut lateral = point[0] - center[0];
    if flip_lateral {
        lateral = -lateral;
    }
    let axial = point[1] - center[1];
    let diagonal = std::f32::consts::FRAC_1_SQRT_2;
    let mut screen = match layout {
        WeaponIconLayout::HiltFocus => {
            [(-axial + lateral) * diagonal, (axial + lateral) * diagonal]
        }
        WeaponIconLayout::HeadFocus => [(axial + lateral) * diagonal, (axial - lateral) * diagonal],
    };
    if mirror_x {
        screen[0] = -screen[0];
    }
    screen
}

pub(super) fn occupied_bounds(alpha: &[u8], size: usize) -> Option<IconBounds> {
    let mut bounds = IconBounds::empty();
    for (index, value) in alpha.iter().enumerate() {
        if *value == 0 {
            continue;
        }
        let x = index % size;
        let y = index / size;
        bounds.include([x as f32 / size as f32, y as f32 / size as f32]);
    }
    bounds.is_finite().then_some(bounds)
}
