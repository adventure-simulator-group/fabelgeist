//! Validate every declared vista ring against observed renderer mesh bounds.
//! Fixture extent is an input contract, not a universal fifty-kilometre rule.
use super::{TacticalGameplayCamera, VistaTerrain};
use crate::presentation::VistaTerrainMesh;
use adventuresim_tactical_core::scene_coordinates::ScenePlanPoint;
use adventuresim_tactical_core::scene_input::{VistaLevelIndex, VistaSample};
use bevy::{camera::primitives::Aabb, prelude::*};
use std::collections::BTreeMap;

#[derive(Debug)]
struct RingBounds {
    minimum: ScenePlanPoint,
    maximum: ScenePlanPoint,
}

impl RingBounds {
    fn from_points(points: impl IntoIterator<Item = Vec3>) -> Option<Self> {
        let mut points = points
            .into_iter()
            .map(|p| ScenePlanPoint::from_metres(p.xz()));
        let first = points.next()??;
        points.try_fold(
            Self {
                minimum: first,
                maximum: first,
            },
            |mut bounds, point| {
                let point = point?;
                bounds.include(Self {
                    minimum: point,
                    maximum: point,
                });
                Some(bounds)
            },
        )
    }
    fn include(&mut self, other: Self) {
        self.minimum =
            ScenePlanPoint::from_metres(self.minimum.metres().min(other.minimum.metres()))
                .expect("union of finite ring bounds");
        self.maximum =
            ScenePlanPoint::from_metres(self.maximum.metres().max(other.maximum.metres()))
                .expect("union of finite ring bounds");
    }
    fn matches(&self, other: &Self) -> bool {
        let tolerance =
            adventuresim_tactical_core::city_layout::CityPlotBounds::COORDINATE_TOLERANCE_METRES
                as f32;
        (self.minimum.metres() - other.minimum.metres())
            .abs()
            .cmple(Vec2::splat(tolerance))
            .all()
            && (self.maximum.metres() - other.maximum.metres())
                .abs()
                .cmple(Vec2::splat(tolerance))
                .all()
    }
}

pub(super) type VistaQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static VistaTerrain,
        Option<&'static Aabb>,
        &'static GlobalTransform,
        Has<adventuresim_tactical_core::prelude::Collider>,
        Has<VistaTerrainMesh>,
    ),
    (Without<Camera3d>, Without<TacticalGameplayCamera>),
>;

struct ExpectedVistaRing {
    level: VistaLevelIndex,
    bounds: RingBounds,
}

pub(super) struct VistaCaptureContract(Vec<ExpectedVistaRing>);

pub(super) struct VistaObservation {
    pub presented_lods: Vec<VistaLevelIndex>,
    pub chunks: usize,
    pub colliders: usize,
    pub matches_source: bool,
}

impl VistaCaptureContract {
    pub(super) fn from_source(vista: &VistaSample) -> Self {
        Self(
            vista
                .lods
                .iter()
                .map(|lod| {
                    let half = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                        * lod.spacing_metres
                        * 0.5;
                    let origin = Vec2::new(
                        lod.origin_east_metres as f32,
                        lod.origin_north_metres as f32,
                    );
                    ExpectedVistaRing {
                        level: lod.level,
                        bounds: RingBounds {
                            minimum: ScenePlanPoint::from_metres(origin - half)
                                .expect("validated source ring minimum"),
                            maximum: ScenePlanPoint::from_metres(origin + half)
                                .expect("validated source ring maximum"),
                        },
                    }
                })
                .collect(),
        )
    }
    pub(super) fn supplied_lods(&self) -> usize {
        self.0.len()
    }
    pub(super) fn observe(
        &self,
        vistas: &VistaQuery<'_, '_>,
        maximum_lods: usize,
    ) -> VistaObservation {
        let mut bounds = BTreeMap::new();
        let mut chunks = 0;
        let mut colliders = 0;
        let mut valid_meshes = true;
        for (level, renderer_bounds, transform, collidable, terrain_mesh) in vistas {
            colliders += usize::from(collidable);
            if !terrain_mesh {
                continue;
            }
            chunks += 1;
            // Renderer AABBs survive upload even when CPU mesh arrays are
            // released. Observe their transformed full extent without retaining
            // another copy of the production terrain meshes.
            let observed = renderer_bounds.and_then(|bounds| {
                RingBounds::from_points([-1.0, 1.0].into_iter().flat_map(|x| {
                    [-1.0, 1.0].into_iter().flat_map(move |y| {
                        [-1.0, 1.0].into_iter().map(move |z| {
                            transform.transform_point(
                                Vec3::from(bounds.center)
                                    + Vec3::from(bounds.half_extents) * Vec3::new(x, y, z),
                            )
                        })
                    })
                }))
            });
            if let Some(observed) = observed {
                bounds
                    .entry(level.0)
                    .and_modify(|bounds: &mut RingBounds| {
                        bounds.include(RingBounds {
                            minimum: observed.minimum,
                            maximum: observed.maximum,
                        })
                    })
                    .or_insert(observed);
            } else {
                valid_meshes = false;
            }
        }
        let matches_source = valid_meshes && self.matches_bounds(&bounds, maximum_lods);
        VistaObservation {
            presented_lods: bounds.keys().copied().collect(),
            chunks,
            colliders,
            matches_source,
        }
    }
    fn matches_bounds(
        &self,
        bounds: &BTreeMap<VistaLevelIndex, RingBounds>,
        maximum_lods: usize,
    ) -> bool {
        bounds.len() == self.0.len().min(maximum_lods)
            && self.0.iter().take(maximum_lods).all(|expected| {
                bounds
                    .get(&expected.level)
                    .is_some_and(|observed| expected.bounds.matches(observed))
            })
    }
}

#[cfg(test)]
mod tests;
