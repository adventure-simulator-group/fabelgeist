//! The existing f32 sampling grid shared by primary and split-eave drainage.
use super::*;

pub(super) struct RoofPlanSampling {
    polygon: Vec<Vec2>,
    cutouts: Vec<Vec<Vec2>>,
    min: Vec2,
    max: Vec2,
}

impl RoofPlanSampling {
    pub(super) fn new(face: &RoofFace) -> Self {
        // These native projections are scratch data for the authored 5x5
        // arithmetic kernel. They never replace admitted architectural bounds.
        let polygon = face
            .polygon
            .iter()
            .map(|point| Vec2::new(point.x, point.z))
            .collect::<Vec<_>>();
        let min = polygon
            .iter()
            .copied()
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let max = polygon
            .iter()
            .copied()
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
        let cutouts = face
            .cutouts
            .iter()
            .map(|cutout| {
                cutout
                    .iter()
                    .map(|point| Vec2::new(point.x, point.z))
                    .collect()
            })
            .collect();
        Self {
            polygon,
            cutouts,
            min,
            max,
        }
    }

    pub(super) fn origins(&self) -> impl Iterator<Item = Vec2> + '_ {
        (0..5)
            .flat_map(|x| (0..5).map(move |z| (x, z)))
            .filter_map(|(x, z)| {
                let fraction = Vec2::new((x as f32 + 0.5) / 5.0, (z as f32 + 0.5) / 5.0);
                let point = self.min + (self.max - self.min) * fraction;
                (plan_point_in_convex_polygon(point, &self.polygon)
                    && !self
                        .cutouts
                        .iter()
                        .any(|cutout| plan_point_in_convex_polygon(point, cutout)))
                .then_some(point)
            })
    }
}
