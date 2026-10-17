//! Neighboring source faces retain the same subdivisions on their shared edge.
use super::{Polygon, triangulation::BARYCENTRIC_COLLINEAR_EPSILON};
use std::collections::BTreeMap;

/// Native source vertex pairs and barycentric fractions; no evaluated fitting.
pub(super) struct EdgeSplits {
    fractions: BTreeMap<[u32; 2], BTreeMap<i64, f64>>,
}
impl EdgeSplits {
    pub fn new(faces: &[[u32; 3]], polygons: &[Vec<Polygon>]) -> Self {
        let mut result = Self {
            fractions: BTreeMap::new(),
        };
        for (face, fragments) in faces.iter().zip(polygons) {
            for point in fragments.iter().flatten() {
                let axes = (0..3)
                    .filter(|&i| point[i] > BARYCENTRIC_COLLINEAR_EPSILON)
                    .collect::<Vec<_>>();
                let &[mut low, mut high] = axes.as_slice() else {
                    continue;
                };
                if face[low] > face[high] {
                    std::mem::swap(&mut low, &mut high);
                }
                let fraction = point[high] / (point[low] + point[high]);
                let key = (fraction / BARYCENTRIC_COLLINEAR_EPSILON).round() as i64;
                result
                    .fractions
                    .entry([face[low], face[high]])
                    .or_default()
                    .entry(key)
                    .or_insert(fraction);
            }
        }
        result
    }
    pub fn points(&self, face: &[u32; 3]) -> Vec<[f64; 3]> {
        let mut result = Vec::new();
        for [mut low, mut high] in [[0, 1], [1, 2], [2, 0]] {
            if face[low] > face[high] {
                std::mem::swap(&mut low, &mut high);
            }
            if let Some(fractions) = self.fractions.get(&[face[low], face[high]]) {
                for &fraction in fractions.values() {
                    let mut weights = [0.; 3];
                    weights[low] = 1. - fraction;
                    weights[high] = fraction;
                    result.push(weights);
                }
            }
        }
        result
    }
}
