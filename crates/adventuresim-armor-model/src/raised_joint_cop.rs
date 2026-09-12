//! A closed-outline joint dish with an integral, laterally returned fan.
use crate::{GenerateError, JointCupDesign, PartFrame, PartMesh};
use std::f32::consts::PI;

const DISH_ROWS: usize = 24;
const DISH_RISE_SCALE: f32 = 0.8;
const FAN_START: f32 = 0.2;

pub(crate) fn generate(d: &JointCupDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    crate::plate_patch::fluted_patch(
        DISH_ROWS,
        false,
        d.gauge.thickness.metres(),
        d.fluting.as_ref(),
        [0.0, 1.0],
        |_, _| 0.0,
        |u, v| {
            let [u, v] = d.flute_coordinates(u, v);
            point(d, fit, u, v)
        },
    )
}

fn point(d: &JointCupDesign, fit: &PartFrame, u: f32, v: f32) -> [f32; 3] {
    let [width, length, depth] = fit.half_extents;
    let clearance = d.surface_clearance();
    let square_x = 2.0 * u - 1.0;
    let square_y = 2.0 * v - 1.0;
    // A square-to-disc chart retains an ordinary closed perimeter and fixed
    // connectivity without a collapsed polar apex or an axial opening.
    let rounding = d.wing_roundness.unit() * 0.5;
    let x = square_x * (1.0 - rounding * square_y.powi(2)).sqrt();
    let y = square_y * (1.0 - rounding * square_x.powi(2)).sqrt();
    let radius_squared = x * x + y * y;
    let fan = smooth(((x - FAN_START) / (1.0 - FAN_START)).clamp(0.0, 1.0));
    let notch = 1.0 - d.wing_notch.unit() * (1.0 - y * y).max(0.0).powi(2);
    // The lateral fan ends in a median point, with its upper and lower edges
    // returning to the enclosing cup. A constant-height extension makes a
    // bucket; the roundness control softens the fan's two sloping boundaries.
    let fan_height = 1.0 - square_y.abs().powf(1.0 + d.wing_roundness.unit());
    let wing_reach = width * d.wing.unit() * 4.0 * fan * (1.0 - fan) * notch * fan_height;
    let distal = 1.0 + (d.distal_wing_scale.unit() - 1.0) * smooth((-y).max(0.0));
    let dish = (1.0 - radius_squared).max(0.0);
    let medial = JointCupDesign::DEFAULT_MEDIAL_WRAP.unit();
    let lateral = JointCupDesign::DEFAULT_LATERAL_WRAP.unit();
    let angle = -medial * PI + (lateral * PI + medial * PI) * (x + 1.0) * 0.5;
    let theta = angle
        * if angle < 0.0 {
            d.medial_wrap.unit() / medial
        } else {
            d.lateral_wrap.unit() / lateral
        };
    let radius = ((theta.sin() / (width + clearance)).powi(2)
        + (theta.cos() / (depth + clearance)).powi(2))
    .sqrt()
    .recip();
    let ridge = d.center_ridge.metres() * (1.0 - y.abs()).max(0.0) * fan;
    let radial = radius
        + depth * DISH_RISE_SCALE * d.dome.unit() * dish * (1.0 - fan)
        + d.proximal_flare.metres() * y.max(0.0).powi(4);
    [
        // The fan reaches its lateral extremity before returning to its
        // frontal edge. Keeping that edge on the enclosing radius gives the
        // broad face its anterior-facing normal without extra front standoff.
        (radial + ridge) * theta.sin() + wing_reach,
        (length + clearance)
            * d.length.unit()
            * y
            * (1.0 + (d.wing_height.unit() - 1.0) * fan)
            * distal,
        (radial + ridge) * theta.cos(),
    ]
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
