//! Bounds for fixed obstacles are independent of cloth iterations.
use fabelgeist_bvh::{Aabb, Bvh};
use fabelgeist_math::Vec3;

pub(super) struct Bounds {
    boxes: Vec<Aabb>,
    tree: Bvh,
}
impl Bounds {
    fn new(boxes: Vec<Aabb>) -> Self {
        Self {
            tree: Bvh::build(&boxes),
            boxes,
        }
    }
    pub(super) fn query(&self, query: &Aabb, mut visit: impl FnMut(usize)) {
        self.tree
            .query_aabb(&self.boxes, query, |i| visit(i as usize));
    }
}

pub(super) struct FixedBounds {
    pub(super) faces: Bounds,
    pub(super) edges: Bounds,
    /// Obstacle vertices, indexed from the first fixed particle.
    pub(super) vertices: Bounds,
    pub(super) face_offset: usize,
    pub(super) edge_offset: usize,
}
impl FixedBounds {
    pub(super) fn new(
        faces: &[[u32; 3]],
        edges: &[[u32; 2]],
        dynamic_count: usize,
        positions: &[Vec3],
    ) -> Option<Self> {
        if positions.is_empty() {
            return None;
        }
        let face_offset = faces
            .iter()
            .position(|f| f.iter().all(|&i| i as usize >= dynamic_count))
            .unwrap_or(faces.len());
        let edge_offset = edges
            .iter()
            .position(|e| e.iter().all(|&i| i as usize >= dynamic_count))
            .unwrap_or(edges.len());
        let faces = Bounds::new(
            faces[face_offset..]
                .iter()
                .map(|f| {
                    Aabb::from_points(f.iter().map(|&i| positions[i as usize - dynamic_count]))
                })
                .collect(),
        );
        let edges = Bounds::new(
            edges[edge_offset..]
                .iter()
                .map(|e| {
                    Aabb::from_points(e.iter().map(|&i| positions[i as usize - dynamic_count]))
                })
                .collect(),
        );
        let vertices = Bounds::new(positions.iter().map(|&p| Aabb::point(p)).collect());
        Some(Self {
            faces,
            edges,
            vertices,
            face_offset,
            edge_offset,
        })
    }
}
