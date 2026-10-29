//! Lift actual trimmed cloth triangles into a shared source displacement field.
//! Native boundary: canonical metres, packed source ordinals and barycentrics.
use super::{DisplacementCoefficient, LayerConstraint};
use crate::surface_cut::POINT_WORDS;
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;
use std::collections::BTreeMap;

pub(crate) struct CutTopology<'a> {
    pub table: &'a [u32],
    pub faces: &'a [[u32; 3]],
}

pub(super) struct CutDomain<'a> {
    pub positions: Vec<[f32; 3]>,
    /// Interpolated source motion; deliberately unnormalized before lifting.
    pub directions: Vec<[f32; 3]>,
    pub faces: &'a [[u32; 3]],
    rows: &'a [[u32; POINT_WORDS]],
}

pub(super) struct PlaneSupport {
    pub face: [u32; 3],
    pub weights: [f32; 3],
    pub normal: Vec3,
    pub sample: Vec3,
    pub hit: Vec3,
    pub required_metres: f32,
    pub query_direction: Vec3,
}

impl<'a> CutDomain<'a> {
    pub fn new(
        topology: CutTopology<'a>,
        positions: &[[f32; 3]],
        source_directions: &[[f32; 3]],
    ) -> Result<Self> {
        ensure!(
            positions.len() == source_directions.len(),
            "cloth source motion is incomplete"
        );
        ensure!(
            topology.table.len().is_multiple_of(POINT_WORDS),
            "partial cloth source row"
        );
        let rows = topology.table.as_chunks::<POINT_WORDS>().0;
        ensure!(!topology.faces.is_empty(), "cloth cut has no support faces");
        ensure!(
            topology
                .faces
                .iter()
                .flatten()
                .all(|&v| (v as usize) < rows.len()),
            "cloth cut index is out of range"
        );
        let mut points = Vec::with_capacity(rows.len());
        let mut directions = Vec::with_capacity(rows.len());
        for row in rows {
            let mut point = Vec3::ZERO;
            let mut direction = Vec3::ZERO;
            for corner in 0..3 {
                let weight = f32::from_bits(row[6 + corner]);
                ensure!(weight.is_finite(), "cloth source weight is nonfinite");
                if weight == 0. {
                    continue;
                }
                let vertex = row[corner] as usize;
                point += Vec3::from_array(
                    *positions
                        .get(vertex)
                        .context("cloth source point is out of range")?,
                ) * weight;
                direction += Vec3::from_array(
                    *source_directions
                        .get(vertex)
                        .context("cloth source motion is out of range")?,
                ) * weight;
            }
            ensure!(
                point.is_finite() && direction.is_finite(),
                "cloth cut evaluation is nonfinite"
            );
            points.push(point.to_array());
            directions.push(direction.to_array());
        }
        // The actual query mesh follows the same area policy as its Boolean
        // producer, rather than the old coarse-source triangle threshold.
        for face in topology.faces {
            let [a, b, c] = face.map(|v| Vec3::from_array(points[v as usize]));
            let area_squared = (b - a).cross(c - a).length_squared() * 0.25;
            ensure!(
                area_squared.is_finite()
                    && area_squared > crate::surface_cut::MINIMUM_TRIANGLE_AREA_SQUARED,
                "cloth cut has a degenerate support triangle"
            );
        }
        Ok(Self {
            positions: points,
            directions,
            faces: topology.faces,
            rows,
        })
    }

    pub fn lift(&self, plane: PlaneSupport) -> Result<LayerConstraint> {
        // Compose the triangle hit and each packed source row first. Repeated
        // source IDs are one degree of freedom, including in projection norms.
        let mut weights = BTreeMap::<u32, f32>::new();
        for (cut_vertex, cut_weight) in plane.face.into_iter().zip(plane.weights) {
            let row = self
                .rows
                .get(cut_vertex as usize)
                .context("support has no cut row")?;
            for corner in 0..3 {
                let weight = cut_weight * f32::from_bits(row[6 + corner]);
                if weight != 0. {
                    *weights.entry(row[corner]).or_default() += weight;
                }
            }
        }
        let coefficients = weights
            .into_iter()
            .map(|(vertex, weight)| {
                let response = weight * plane.normal;
                ensure!(response.is_finite(), "lifted cloth support is nonfinite");
                Ok(DisplacementCoefficient { vertex, response })
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            !coefficients.is_empty(),
            "cloth support has no source motion"
        );
        Ok(LayerConstraint {
            coefficients,
            query_direction: plane.query_direction,
            sample: plane.sample,
            hit: plane.hit,
            required_metres: plane.required_metres,
        })
    }
}
