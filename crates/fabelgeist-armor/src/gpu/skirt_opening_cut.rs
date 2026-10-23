//! Insert the intended meridian into carrier triangles before splitting lips.
//! The numerical chart uses radians and canonical upright metres. Intersections
//! retain original edge endpoints, so refits interpolate the same cloth edge.
use super::coord::CoordShell;
use super::skirt_opening::{OpeningChartPoint, OpeningEdgeSource, SkirtOpeningError, angle_delta};
use std::collections::BTreeMap;

pub(super) const CUT_EDGE_COORDINATE: f32 = 7.;
pub(super) const CUT_TIP_COORDINATE: f32 = 8.;
pub(super) const MERIDIAN_ANGLE_TOLERANCE_RAD: f32 = 0.000001;

pub(super) struct MeridianCut {
    intersections: BTreeMap<[u32; 2], u32>,
    pub sources: BTreeMap<usize, OpeningEdgeSource>,
}

impl MeridianCut {
    pub fn insert(
        shell: &mut CoordShell,
        chart: &mut Vec<Option<OpeningChartPoint>>,
        axis: f32,
    ) -> Result<Self, SkirtOpeningError> {
        let mut cut = Self {
            intersections: BTreeMap::new(),
            sources: BTreeMap::new(),
        };
        let mut triangles = Vec::new();
        for face in shell.indices.as_chunks::<3>().0.to_vec() {
            let Some(points) = face
                .map(|v| chart[v as usize])
                .into_iter()
                .collect::<Option<Vec<_>>>()
            else {
                triangles.extend(face);
                continue;
            };
            let distances = points
                .iter()
                .map(|p| angle_delta(p.angle, axis))
                .collect::<Vec<_>>();
            let low = distances.iter().copied().fold(f32::INFINITY, f32::min);
            let high = distances.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            // The discontinuity opposite this meridian is not a second cut.
            if low >= -MERIDIAN_ANGLE_TOLERANCE_RAD
                || high <= MERIDIAN_ANGLE_TOLERANCE_RAD
                || high - low > std::f32::consts::PI
            {
                triangles.extend(face);
                continue;
            }
            for positive in [true, false] {
                let mut polygon = Vec::new();
                for edge in 0..3 {
                    let next = (edge + 1) % 3;
                    let [a, b] = [face[edge], face[next]];
                    let [da, db] = [distances[edge], distances[next]];
                    if if positive {
                        da >= -MERIDIAN_ANGLE_TOLERANCE_RAD
                    } else {
                        da <= MERIDIAN_ANGLE_TOLERANCE_RAD
                    } {
                        polygon.push(a);
                    }
                    if (da < -MERIDIAN_ANGLE_TOLERANCE_RAD && db > MERIDIAN_ANGLE_TOLERANCE_RAD)
                        || (db < -MERIDIAN_ANGLE_TOLERANCE_RAD && da > MERIDIAN_ANGLE_TOLERANCE_RAD)
                    {
                        polygon.push(cut.intersection(
                            shell,
                            chart,
                            [a, b],
                            da / (da - db),
                            axis,
                        )?);
                    }
                }
                polygon.dedup();
                if polygon.first() == polygon.last() {
                    polygon.pop();
                }
                for i in 1..polygon.len().saturating_sub(1) {
                    triangles.extend([polygon[0], polygon[i], polygon[i + 1]]);
                }
            }
        }
        shell.indices = triangles;
        Ok(cut)
    }

    fn intersection(
        &mut self,
        shell: &mut CoordShell,
        chart: &mut Vec<Option<OpeningChartPoint>>,
        endpoints: [u32; 2],
        blend: f32,
        axis: f32,
    ) -> Result<u32, SkirtOpeningError> {
        let mut key = endpoints;
        key.sort_unstable();
        if let Some(&vertex) = self.intersections.get(&key) {
            return Ok(vertex);
        }
        let [Some(a), Some(b)] = endpoints.map(|v| chart[v as usize]) else {
            return Err(SkirtOpeningError::Source);
        };
        let vertex = shell.coords.len() as u32;
        shell.coords.push([
            endpoints[0] as f32,
            endpoints[1] as f32,
            blend,
            CUT_EDGE_COORDINATE,
        ]);
        chart.push(Some(OpeningChartPoint {
            angle: axis,
            y: a.y + (b.y - a.y) * blend,
        }));
        self.sources.insert(
            vertex as usize,
            OpeningEdgeSource {
                vertices: endpoints.map(|v| v as usize),
                blend,
            },
        );
        self.intersections.insert(key, vertex);
        Ok(vertex)
    }
}
