use super::*;

pub(super) const WELL_MARGIN_METRES: f32 = 0.06;
pub(super) const DECK_THICKNESS_METRES: f32 = 0.16;
const MAXIMUM_RISER_METRES: f32 = 0.19;
const TREAD_THICKNESS_METRES: f32 = 0.14;
const NEWEL_EMBED_METRES: f32 = 0.08;
const LANDING_WIDTH_METRES: f32 = 0.9;
const PORTAL_OUTSIDE_WELL_METRES: f32 = 0.4;
const LANDING_END_MARGIN_METRES: f32 = 0.5;
const TREAD_OVERLAP_METRES: f32 = 0.025;
const LANDING_TREAD_BEARING_METRES: f32 = 0.12;

#[derive(Clone, Copy, Debug)]
pub struct SpiralMember {
    pub centre: Vec3,
    pub size: Vec3,
    pub yaw_radians: f32,
    pub role: SolidRole,
}

#[derive(Clone, Copy, Debug)]
pub struct SpiralLanding {
    pub storey: u16,
    pub position_metres: Vec2,
    pub elevation_metres: f32,
}

#[derive(Clone, Debug)]
pub struct SpiralFlight {
    pub centre: Vec2,
    pub outer_radius_metres: f32,
    pub base_height_metres: f32,
    pub top_height_metres: f32,
    pub members: Vec<SpiralMember>,
    pub landings: Vec<SpiralLanding>,
}

/// Pure geometry is also consumed by the editor viewer; no second spiral recipe.
pub fn compile_flight(stair: Stair, storey_height: f32) -> Option<SpiralFlight> {
    let Stair::Spiral {
        centre,
        base_height_metres,
        rise_metres,
        inner_radius_metres,
        outer_radius_metres,
        turns,
        clockwise,
        tread_count,
    } = stair
    else {
        return None;
    };
    let top = base_height_metres + rise_metres;
    let handedness = if clockwise { -1.0 } else { 1.0 };
    let angle = |height: f32| {
        ((height - base_height_metres) / rise_metres).clamp(0.0, 1.0)
            * turns
            * std::f32::consts::TAU
            * handedness
    };
    let mut members = vec![SpiralMember {
        centre: Vec3::new(centre.x, top * 0.5, centre.y),
        size: Vec3::new(inner_radius_metres * 2.0, top, inner_radius_metres * 2.0),
        yaw_radians: 0.0,
        role: SolidRole::StairNewel,
    }];
    let levels = level_breaks(base_height_metres, rise_metres, storey_height);
    let required = required_treads(base_height_metres, rise_metres, storey_height);
    let extras = tread_count.checked_sub(required)?;
    let radial_inner = (inner_radius_metres - NEWEL_EMBED_METRES).max(0.05);
    let radius = (radial_inner + outer_radius_metres) * 0.5;
    if base_height_metres > 0.0 {
        members.push(SpiralMember {
            centre: Vec3::new(centre.x + radius, base_height_metres * 0.5, centre.y),
            size: Vec3::new(
                outer_radius_metres - radial_inner,
                base_height_metres,
                LANDING_WIDTH_METRES,
            ),
            yaw_radians: 0.0,
            role: SolidRole::StairTread,
        });
    }
    let interval_count = levels.len() - 1;
    for (interval, elevations) in levels.windows(2).enumerate() {
        let extra = usize::from(extras) / interval_count
            + usize::from(interval < usize::from(extras) % interval_count);
        let count =
            ((elevations[1] - elevations[0]) / MAXIMUM_RISER_METRES).ceil() as u16 + extra as u16;
        for tread in 1..=count {
            let previous = elevations[0]
                + (elevations[1] - elevations[0]) * f32::from(tread - 1) / f32::from(count);
            let height = elevations[0]
                + (elevations[1] - elevations[0]) * f32::from(tread) / f32::from(count);
            let theta = angle(height);
            let depth = 2.0 * outer_radius_metres * ((theta - angle(previous)).abs() * 0.5).tan()
                + TREAD_OVERLAP_METRES;
            let point = centre + Vec2::new(theta.cos(), theta.sin()) * radius;
            members.push(SpiralMember {
                centre: Vec3::new(point.x, height - TREAD_THICKNESS_METRES * 0.5, point.y),
                size: Vec3::new(
                    outer_radius_metres - radial_inner,
                    TREAD_THICKNESS_METRES,
                    depth.max(0.12),
                ),
                yaw_radians: -theta,
                role: SolidRole::StairTread,
            });
        }
    }
    let (landing_members, landings) =
        landing_members(centre, outer_radius_metres, top, storey_height, angle);
    members.extend(landing_members);
    Some(SpiralFlight {
        centre,
        outer_radius_metres,
        base_height_metres,
        top_height_metres: top,
        members,
        landings,
    })
}

/// Count aligned to each occupied storey, including a grounded entrance riser.
pub fn required_treads(base: f32, rise: f32, storey_height: f32) -> u16 {
    u16::from(base > 0.0)
        + level_breaks(base, rise, storey_height)
            .windows(2)
            .map(|pair| ((pair[1] - pair[0]) / MAXIMUM_RISER_METRES).ceil() as u16)
            .sum::<u16>()
}

fn level_breaks(base: f32, rise: f32, storey_height: f32) -> Vec<f32> {
    let top = base + rise;
    let mut levels = vec![base];
    levels.extend(
        (1..=(top / storey_height).floor() as u16)
            .map(|level| f32::from(level) * storey_height)
            .filter(|height| *height > base && *height < top - 0.001),
    );
    levels.push(top);
    levels
}

/// Crown guard openings align with the actual final tread at the flight top.
pub fn arrival_angle(stair: Stair) -> Option<f32> {
    let Stair::Spiral {
        turns, clockwise, ..
    } = stair
    else {
        return None;
    };
    Some(if clockwise { -1.0 } else { 1.0 } * turns * std::f32::consts::TAU)
}

fn landing_members(
    centre: Vec2,
    outer_radius_metres: f32,
    top: f32,
    storey_height: f32,
    angle: impl Fn(f32) -> f32,
) -> (Vec<SpiralMember>, Vec<SpiralLanding>) {
    let mut members = Vec::new();
    let well = outer_radius_metres + WELL_MARGIN_METRES;
    let mut landings = Vec::new();
    for level in 0..=(top / storey_height).floor() as u16 {
        let height = f32::from(level) * storey_height;
        let theta = angle(height);
        let axis = Vec2::new(theta.cos(), theta.sin());
        let portal_radius = (well + PORTAL_OUTSIDE_WELL_METRES) / axis.abs().max_element();
        let end = portal_radius + LANDING_END_MARGIN_METRES;
        let point = centre + axis * portal_radius;
        let landing_start = outer_radius_metres - LANDING_TREAD_BEARING_METRES;
        let slab_centre = centre + axis * (landing_start + end) * 0.5;
        members.push(SpiralMember {
            centre: Vec3::new(
                slab_centre.x,
                height - DECK_THICKNESS_METRES * 0.5,
                slab_centre.y,
            ),
            size: Vec3::new(
                end - landing_start,
                DECK_THICKNESS_METRES,
                LANDING_WIDTH_METRES,
            ),
            yaw_radians: -theta,
            role: SolidRole::Landing,
        });
        landings.push(SpiralLanding {
            storey: level,
            position_metres: point,
            elevation_metres: height,
        });
    }
    (members, landings)
}
