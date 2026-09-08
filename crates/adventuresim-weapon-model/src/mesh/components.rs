//! Canonical component solids shared by rendering and physical integration.
use super::*;

impl RawMesh {
    pub(crate) fn from_shape(shape: &ComponentShape) -> Self {
        match shape {
            ComponentShape::Cylinder(value) => frustum(
                value.length.meters(),
                value.radius.meters() * value.bottom_scale.unit(),
                value.radius.meters() * value.top_scale.unit(),
                value.segments.0,
            ),
            ComponentShape::OvalGrip(value) => elliptical_frustum(
                value.length.meters(),
                value.width.meters() * value.bottom_scale.unit() * 0.5,
                value.width.meters() * value.top_scale.unit() * 0.5,
                value.thickness.meters() / value.width.meters(),
                value.segments.0,
            ),
            ComponentShape::Blade(value) => blade(value),
            ComponentShape::Guard(value) => guard(value),
            ComponentShape::Mace(value) => mace(value),
            ComponentShape::Socket(value) => socket(value),
            ComponentShape::Langet(value) => box_mesh(
                [0.0, value.length.meters() / 2.0, 0.0],
                [
                    value.width.meters(),
                    value.length.meters(),
                    value.thickness.meters(),
                ],
                0.0,
            ),
            ComponentShape::Axe(value) => axe(value),
            ComponentShape::HammerPoll(value) => hammer(value),
            ComponentShape::CurvedBeak(value) => curved_beak(value),
            ComponentShape::FacetedBeak(value) => faceted_beak(value),
            ComponentShape::Glaive(value) => glaive(value),
            ComponentShape::Bill(value) => bill(value),
            ComponentShape::Fork(value) => fork(value),
            ComponentShape::Partisan(value) => partisan(value),
            ComponentShape::TubePath(value) => tube_path(value),
            ComponentShape::RingGuard(value) => ring(value),
            ComponentShape::FigureEight(value) => figure_eight(value),
            ComponentShape::FanPommel(value) => fan_pommel(value),
            ComponentShape::Rondel(value) => cylinder(
                value.thickness.meters(),
                value.radius.meters(),
                value.segments.0,
            ),
            ComponentShape::GothicMace(value) => gothic_mace(value),
            ComponentShape::SlabGrip(value) => box_mesh(
                [0.0, value.length.meters() / 2.0, 0.0],
                [
                    value.width.meters(),
                    value.length.meters(),
                    value.thickness.meters() + value.scale_thickness.meters() * 2.0,
                ],
                0.0,
            ),
            ComponentShape::KnuckleBow(value) => knuckle_bow(value),
            ComponentShape::Collar(value) => cylinder(
                value.width.meters(),
                value.radius.meters(),
                value.segments.0,
            ),
            ComponentShape::Sleeve(value) => socket(&SocketSpec {
                length: value.length,
                outer_radius: value.radius,
                top_radius: value.top_radius,
                wall: value.wall,
                segments: value.segments,
            }),
            ComponentShape::Boss(value) => boss(value),
            ComponentShape::Spear(value) => spear(value),
            ComponentShape::ProfiledPommel(value) => profiled_pommel(value),
        }
    }
}

fn knuckle_bow(value: &crate::KnuckleBowSpec) -> RawMesh {
    let count = value.samples.0 as usize;
    let centers: Vec<_> = (0..=count)
        .map(|index| {
            let t = index as f32 / count as f32;
            let arch = (std::f32::consts::PI * t).sin();
            [
                value.side as f32
                    * value.width.meters()
                    * arch
                    * (1.0 + value.bulge.unit() * 0.25 * arch),
                value.length.meters() * t,
                0.0,
            ]
        })
        .collect();
    tube_centers(
        &centers,
        value.bar.meters(),
        value.radial_segments.0 as usize,
        false,
    )
}

fn boss(value: &crate::BossSpec) -> RawMesh {
    let mut mesh = cylinder(
        value.thickness.meters(),
        value.radius.meters(),
        value.segments.0,
    );
    for point in &mut mesh.positions {
        [point[1], point[2]] = [point[2], point[1]];
    }
    mesh.orient_positive();
    mesh
}
