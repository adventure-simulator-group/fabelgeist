//! Frozen waist-arc samples for independent folded support bands.
use crate::armor_frames::Wearer;
use crate::cloth_opening::ClothSurfaceSample;
use crate::skirt_join::{SkirtJoinPlan, angular_sample};
use crate::surface_cut::interpolate;
use anyhow::{Result, ensure};
use fabelgeist_armor::{GownDesign, GownWaistSupport, Micrometers};
use fabelgeist_math::Vec3;
const ARC_STATION_ROUNDOFF_M: f32 = 0.000001;
/// Native canonical arc table; angles are radians and distances are metres.
struct WaistArcStation {
    angle: f32,
    distance: f32,
}
/// A frozen waist carrier corner and its canonical arc-length chart share.
pub(crate) struct WaistBandStation {
    pub source: ClothSurfaceSample,
    pub progress: f32,
}
pub(crate) struct WaistSupportPlan {
    pub design: GownWaistSupport,
    pub main: Vec<WaistBandStation>,
    pub front: Vec<WaistBandStation>,
    pub inset: Micrometers,
    /// Native canonical main-band arc in metres; endpoints keep these shares.
    pub main_span_metres: f32,
    pub closing_side: Option<WaistClosingSide>,
    pub lacing: Option<fabelgeist_armor::ClothLacing>,
    pub body_points: usize,
}
#[derive(Clone, Copy)]
pub(crate) enum WaistClosingSide {
    FrontStart,
    FrontEnd,
}
struct WaistArc {
    stations: Vec<WaistArcStation>,
    length: f32,
}
impl WaistSupportPlan {
    pub fn new(
        gown: &GownDesign,
        join: &SkirtJoinPlan,
        body: &Wearer<'_>,
        opening: Option<&fabelgeist_armor::gpu::SkirtOpeningSource>,
    ) -> Result<Option<Self>> {
        let Some(design) = &gown.waist_support else {
            return Ok(None);
        };
        let arc = WaistArc::new(join, body)?;
        let distance = arc.length;
        let width = design.front_width.metres();
        ensure!(
            distance > width * 2.,
            "front waist support leaves insufficient main band"
        );
        let mut front_start = distance - width * 0.5;
        let mut main_start = width * 0.5;
        let mut main_span = distance - width;
        if let Some(opening) = opening {
            let closing = arc.distance_at_angle(opening.axis_angle)?;
            let overlap = opening.overlap.metres();
            if opening.axis_angle.rem_euclid(std::f32::consts::TAU) > std::f32::consts::PI {
                front_start = closing;
                main_start = closing + width;
            } else {
                front_start = closing - width;
                main_start = closing - overlap;
            }
            main_span += overlap;
        }
        let main = arc.sample_span(join, main_start, main_span)?;
        let front = arc.sample_span(join, front_start, width)?;
        // Nominal seating behind the face cloth; this is not collision clearance.
        let inset = Micrometers(gown.bodice.thickness.0 * Micrometers::PER_MILLIMETRE as u16);
        Ok(Some(Self {
            design: design.clone(),
            main,
            front,
            inset,
            main_span_metres: main_span,
            closing_side: opening.map(|o| {
                if o.axis_angle.rem_euclid(std::f32::consts::TAU) > std::f32::consts::PI {
                    WaistClosingSide::FrontStart
                } else {
                    WaistClosingSide::FrontEnd
                }
            }),
            lacing: gown.lacing.clone(),
            body_points: join.table.len() / crate::surface_cut::POINT_WORDS,
        }))
    }
}
fn sample_at_angle(join: &SkirtJoinPlan, angle: f32) -> ClothSurfaceSample {
    let [a, b, t] = angular_sample(&join.patch.waist, angle);
    let t = f32::from_bits(t);
    ClothSurfaceSample {
        vertices: [a as usize, b as usize, b as usize],
        weights: [1. - t, t, 0.],
    }
}

impl WaistArc {
    fn new(join: &SkirtJoinPlan, body: &Wearer<'_>) -> Result<Self> {
        let points = join
            .table
            .as_chunks::<{ crate::surface_cut::POINT_WORDS }>()
            .0
            .iter()
            .map(|p| {
                let ids = [p[0], p[1], p[2]];
                let weights = [
                    f32::from_bits(p[6]),
                    f32::from_bits(p[7]),
                    f32::from_bits(p[8]),
                ];
                interpolate(ids.map(|v| body.positions[v as usize]), weights)
            })
            .collect::<Vec<_>>();
        let ring = &join.patch.waist;
        let mut angles = vec![0.];
        angles.extend(ring.iter().filter(|p| p.angle > 0.).map(|p| p.angle));
        angles.push(std::f32::consts::TAU);
        let mut arc = Vec::new();
        let mut distance = 0.;
        let mut previous: Option<Vec3> = None;
        for angle in angles {
            let sample = sample_at_angle(join, angle.rem_euclid(std::f32::consts::TAU));
            let point = sample.interpolate(&points)?;
            if let Some(last) = previous {
                distance += (point - last).length();
            }
            arc.push(WaistArcStation { angle, distance });
            previous = Some(point);
        }
        Ok(Self {
            stations: arc,
            length: distance,
        })
    }
    /// Retain every parent waist corner instead of spanning it with a chord.
    /// Native distances are canonical arc metres; connectivity is frozen here.
    fn sample_span(
        &self,
        join: &SkirtJoinPlan,
        start: f32,
        span: f32,
    ) -> Result<Vec<WaistBandStation>> {
        ensure!(span > 0. && span < self.length, "invalid support-band arc");
        let mut distances = vec![0., span];
        for station in &self.stations {
            let distance = (station.distance - start).rem_euclid(self.length);
            if distance > ARC_STATION_ROUNDOFF_M && distance < span - ARC_STATION_ROUNDOFF_M {
                distances.push(distance);
            }
        }
        distances.sort_by(f32::total_cmp);
        distances.dedup_by(|a, b| (*a - *b).abs() <= ARC_STATION_ROUNDOFF_M);
        distances
            .into_iter()
            .map(|distance| {
                Ok(WaistBandStation {
                    source: self.sample(join, start + distance)?,
                    progress: distance / span,
                })
            })
            .collect()
    }
    fn distance_at_angle(&self, angle: f32) -> Result<f32> {
        let angle = angle.rem_euclid(std::f32::consts::TAU);
        let span = self
            .stations
            .windows(2)
            .find(|p| p[0].angle <= angle && angle <= p[1].angle)
            .ok_or_else(|| anyhow::anyhow!("waist closing angle has no canonical arc span"))?;
        let t = (angle - span[0].angle) / (span[1].angle - span[0].angle);
        Ok(span[0].distance + (span[1].distance - span[0].distance) * t)
    }
    fn sample(&self, join: &SkirtJoinPlan, at: f32) -> Result<ClothSurfaceSample> {
        let at = at.rem_euclid(self.length);
        let span = self
            .stations
            .windows(2)
            .find(|p| p[0].distance <= at && at <= p[1].distance)
            .ok_or_else(|| anyhow::anyhow!("waist support arc is incomplete"))?;
        let t = (at - span[0].distance) / (span[1].distance - span[0].distance);
        Ok(sample_at_angle(
            join,
            span[0].angle + (span[1].angle - span[0].angle) * t,
        ))
    }
}
