//! Material distances to the course cuts, independent of shell extrusion.
use crate::SurfaceEdgeDistance;

#[derive(Clone, Debug)]
pub(crate) struct CourseCoordinates {
    levels: Vec<f64>,
    low: f64,
    high: Option<f64>,
    plane_scale: f64,
    cut_candidates: Vec<[bool; 2]>,
}

pub(super) struct CourseSeams<'a> {
    course: &'a CourseCoordinates,
    vertices: Vec<[bool; 2]>,
}

impl CourseCoordinates {
    pub fn new(
        levels: Vec<f64>,
        cut_candidates: Vec<[bool; 2]>,
        low: f64,
        high: Option<f64>,
        slope: f32,
    ) -> Self {
        Self {
            levels,
            low,
            high,
            plane_scale: (1.0 + f64::from(slope).powi(2)).sqrt(),
            cut_candidates,
        }
    }

    /// Only surviving boundary edges define the snapped seam. A near-plane
    /// interior vertex retains its signed distance instead of becoming a seam.
    pub fn on_boundaries(&self, edges: &[[u32; 2]]) -> CourseSeams<'_> {
        let mut vertices = vec![[false; 2]; self.levels.len()];
        for &[a, b] in edges {
            for (end, (&a_cut, &b_cut)) in self.cut_candidates[a as usize]
                .iter()
                .zip(&self.cut_candidates[b as usize])
                .enumerate()
            {
                if a_cut && b_cut {
                    vertices[a as usize][end] = true;
                    vertices[b as usize][end] = true;
                }
            }
        }
        CourseSeams {
            course: self,
            vertices,
        }
    }
}

impl CourseSeams<'_> {
    pub fn edge_distances(&self, vertex: usize) -> [SurfaceEdgeDistance; 2] {
        let course = self.course;
        let level = course.levels[vertex];
        let cut = |distance: f64| SurfaceEdgeDistance::Cut {
            metres: (distance / course.plane_scale) as f32,
        };
        [
            course.high.map_or(SurfaceEdgeDistance::Rail, |high| {
                cut(if self.vertices[vertex][0] {
                    0.0
                } else {
                    high - level
                })
            }),
            cut(if self.vertices[vertex][1] {
                0.0
            } else {
                level - course.low
            }),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapped_boundary_membership_does_not_erase_signed_interior_distances() {
        let course = CourseCoordinates::new(
            vec![-0.0001, 0.0001, 1.0, 0.0001, -0.0001],
            vec![
                [false, true],
                [false, true],
                [false; 2],
                [false, true],
                [false, true],
            ],
            0.0,
            None,
            0.3,
        );
        let seams = course.on_boundaries(&[[0, 1], [1, 2], [2, 0]]);
        assert_eq!(
            seams.edge_distances(0)[1],
            SurfaceEdgeDistance::Cut { metres: 0.0 }
        );
        assert_eq!(
            seams.edge_distances(1)[1],
            SurfaceEdgeDistance::Cut { metres: 0.0 }
        );
        for (vertex, sign) in [(3, 1.0), (4, -1.0)] {
            let SurfaceEdgeDistance::Cut { metres } = seams.edge_distances(vertex)[1] else {
                panic!("lower cut must have a material coordinate");
            };
            assert!(metres * sign > 0.0);
            assert!((metres.abs() - 0.0001 / 1.09_f32.sqrt()).abs() < 1e-9);
        }
        assert_eq!(seams.edge_distances(3)[0], SurfaceEdgeDistance::Rail);
    }
}
