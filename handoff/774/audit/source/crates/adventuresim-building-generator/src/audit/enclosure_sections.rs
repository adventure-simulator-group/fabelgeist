//! Sections of non-box opening surrounds retain their real splayed and arched shape.
use super::Result;
use crate::spatial_geometry::{Architectural, Elevation, PlanDirection, Position, PositiveLength};
use crate::{BuildingPlan, ResolvedSolid, ResolvedSolidShape};
use bevy::math::{Vec2, Vec3};

#[derive(Clone, Copy)]
pub(super) struct ElevationInterval {
    pub low: Elevation<Architectural>,
    pub high: Elevation<Architectural>,
}
impl ElevationInterval {
    pub fn new(low: Elevation<Architectural>, high: Elevation<Architectural>) -> Result<Self> {
        crate::SpatialBounds::<Architectural>::from_metres(
            Vec3::new(0.0, low.metres(), 0.0),
            Vec3::new(0.0, high.metres(), 0.0),
        )?;
        Ok(Self { low, high })
    }
}

const PLANE_TOLERANCE_METRES: f32 = 0.00001;

pub(super) fn intervals(
    plan: &BuildingPlan,
    solid: &ResolvedSolid,
    origin: Position<Architectural>,
    wall_normals: [PlanDirection<Architectural>; 2],
    depth: PositiveLength,
    base: Elevation<Architectural>,
    top: Elevation<Architectural>,
) -> Result<Vec<ElevationInterval>> {
    // Section direction retains the magnitude of summed admitted wall normals;
    // normalization here would change the established sampling contract.
    let outward = wall_normals[0].vector() + wall_normals[1].vector();
    let direction = Vec3::new(outward.x, 0.0, outward.y);
    let (origin, depth, base, top) = (origin.metres(), depth.metres(), base.metres(), top.metres());
    if matches!(
        solid.shape,
        ResolvedSolidShape::Cuboid | ResolvedSolidShape::TimberPanelPrism { .. }
    ) {
        return super::enclosure_geometry::vertical_interval(
            solid, origin, direction, depth, base, top,
        )
        .into_iter()
        .map(|(low, high)| {
            ElevationInterval::new(Elevation::from_metres(low)?, Elevation::from_metres(high)?)
        })
        .collect::<Result<Vec<_>>>();
    }
    if !matches!(
        solid.shape,
        ResolvedSolidShape::SplayedReveal { .. }
            | ResolvedSolidShape::SplayedHead { .. }
            | ResolvedSolidShape::SegmentalArchRing { .. }
            | ResolvedSolidShape::PointedArchRing { .. }
    ) {
        return Ok(Vec::new());
    }
    let detail = crate::detail::compile_solid_detail(plan, solid)?;
    detail
        .meshes
        .iter()
        .flat_map(|mesh| {
            mesh.indices
                .as_chunks::<3>()
                .0
                .iter()
                .filter_map(|indices| {
                    let triangle =
                        [0, 1, 2].map(|index| mesh.vertices[indices[index] as usize].position);
                    section_interval(triangle, origin, direction, depth, base, top)
                })
        })
        .map(|(low, high)| {
            ElevationInterval::new(Elevation::from_metres(low)?, Elevation::from_metres(high)?)
        })
        .collect()
}

fn section_interval(
    triangle: [Vec3; 3],
    origin: Vec3,
    direction: Vec3,
    depth: f32,
    base: f32,
    top: f32,
) -> Option<(f32, f32)> {
    let normal = Vec3::new(-direction.z, 0.0, direction.x).normalize();
    let mut points = Vec::new();
    let project = |p: Vec3| {
        Vec2::new(
            (p - origin).dot(direction) / direction.length_squared(),
            p.y,
        )
    };
    for index in 0..3 {
        let a = triangle[index];
        let b = triangle[(index + 1) % 3];
        let da = normal.dot(a - origin);
        let db = normal.dot(b - origin);
        if da.abs() <= PLANE_TOLERANCE_METRES {
            points.push(project(a));
        }
        if (da < 0.0 && db > 0.0) || (da > 0.0 && db < 0.0) {
            points.push(project(a.lerp(b, da / (da - db))));
        }
    }
    for sign in [-1.0, 1.0] {
        let mut next = Vec::new();
        for (&a, &b) in points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
        {
            let da = a.x * sign - depth;
            let db = b.x * sign - depth;
            if da <= 0.0 {
                next.push(a);
            }
            if (da < 0.0 && db > 0.0) || (da > 0.0 && db < 0.0) {
                next.push(a.lerp(b, da / (da - db)));
            }
        }
        points = next;
    }
    let low = points
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min)
        .max(base);
    let high = points
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max)
        .min(top);
    (low <= high).then_some((low, high))
}
