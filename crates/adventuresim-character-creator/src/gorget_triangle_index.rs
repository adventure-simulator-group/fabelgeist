//! Conservative broad-phase queries over gorget support triangles.

const LEAF_FACES: usize = 8;

pub(super) struct TriangleSupport<'a> {
    pub positions: &'a [[f32; 3]],
    pub triangles: &'a [[u32; 3]],
    nodes: Vec<Node>,
    faces: Vec<usize>,
}

#[derive(Clone, Copy)]
struct Node {
    lower: [f32; 3],
    upper: [f32; 3],
    children: Option<[usize; 2]>,
    range: [usize; 2],
}

impl<'a> TriangleSupport<'a> {
    pub fn new(positions: &'a [[f32; 3]], triangles: &'a [[u32; 3]]) -> Self {
        let mut index = Self {
            positions,
            triangles,
            nodes: Vec::new(),
            faces: (0..triangles.len()).collect(),
        };
        if !triangles.is_empty() {
            index.build(positions, triangles, 0, triangles.len());
        }
        index
    }

    pub fn for_each_candidate(
        &self,
        direction: [f32; 3],
        lower: [f32; 2],
        upper: [f32; 2],
        mut visit: impl FnMut(usize),
    ) {
        if !self.nodes.is_empty() {
            self.query(0, direction, lower, upper, &mut visit);
        }
    }

    fn build(
        &mut self,
        positions: &[[f32; 3]],
        triangles: &[[u32; 3]],
        start: usize,
        end: usize,
    ) -> usize {
        let (lower, upper) = bounds(positions, triangles, &self.faces[start..end]);
        let node = self.nodes.len();
        self.nodes.push(Node {
            lower,
            upper,
            children: None,
            range: [start, end],
        });
        if end - start > LEAF_FACES {
            let axis = (0..3)
                .max_by(|&a, &b| (upper[a] - lower[a]).total_cmp(&(upper[b] - lower[b])))
                .expect("three spatial axes");
            let middle = start + (end - start) / 2;
            self.faces[start..end].select_nth_unstable_by(middle - start, |&a, &b| {
                centroid(positions, triangles[a], axis)
                    .total_cmp(&centroid(positions, triangles[b], axis))
                    .then_with(|| a.cmp(&b))
            });
            let left = self.build(positions, triangles, start, middle);
            let right = self.build(positions, triangles, middle, end);
            self.nodes[node].children = Some([left, right]);
        }
        node
    }

    fn query(
        &self,
        node: usize,
        direction: [f32; 3],
        lower: [f32; 2],
        upper: [f32; 2],
        visit: &mut impl FnMut(usize),
    ) {
        let node = self.nodes[node];
        if !overlaps(node, direction, lower, upper) {
            return;
        }
        if let Some(children) = node.children {
            self.query(children[0], direction, lower, upper, visit);
            self.query(children[1], direction, lower, upper, visit);
        } else {
            for &face in &self.faces[node.range[0]..node.range[1]] {
                visit(face);
            }
        }
    }
}

fn bounds(positions: &[[f32; 3]], triangles: &[[u32; 3]], faces: &[usize]) -> ([f32; 3], [f32; 3]) {
    let mut lower = [f32::INFINITY; 3];
    let mut upper = [f32::NEG_INFINITY; 3];
    for &face in faces {
        for &vertex in &triangles[face] {
            let point = positions[vertex as usize];
            for axis in 0..3 {
                lower[axis] = lower[axis].min(point[axis]);
                upper[axis] = upper[axis].max(point[axis]);
            }
        }
    }
    (lower, upper)
}

fn centroid(positions: &[[f32; 3]], triangle: [u32; 3], axis: usize) -> f32 {
    triangle
        .into_iter()
        .map(|vertex| positions[vertex as usize][axis])
        .sum::<f32>()
        / 3.0
}

fn overlaps(node: Node, direction: [f32; 3], lower: [f32; 2], upper: [f32; 2]) -> bool {
    let coefficients = [direction[2], -direction[1]];
    let projected = |maximum: bool| {
        (0..2)
            .map(|axis| {
                let bound = if (coefficients[axis] >= 0.0) == maximum {
                    node.upper[axis + 1]
                } else {
                    node.lower[axis + 1]
                };
                coefficients[axis] * bound
            })
            .sum::<f32>()
    };
    node.lower[0] <= upper[0]
        && node.upper[0] >= lower[0]
        && projected(false) <= upper[1]
        && projected(true) >= lower[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projected_queries_keep_every_overlapping_triangle() {
        let positions = (0..36)
            .map(|i| {
                let x = (i % 6) as f32 * 0.1 - 0.25;
                let y = (i / 6) as f32 * 0.08 - 0.20;
                [x, y, (x * 7.0 + y * 11.0).sin() * 0.12]
            })
            .collect::<Vec<_>>();
        let triangles = (0..5)
            .flat_map(|row| {
                (0..5).flat_map(move |column| {
                    let a = row * 6 + column;
                    [[a, a + 1, a + 7], [a, a + 7, a + 6]]
                })
            })
            .map(|face| face.map(|vertex| vertex as u32))
            .collect::<Vec<_>>();
        let index = TriangleSupport::new(&positions, &triangles);
        for direction in [[0.0, 1.0, 0.0], [0.0, 0.6, -0.8], [0.0, -0.3, 0.954]] {
            let lower = [-0.07, -0.05];
            let upper = [0.16, 0.11];
            let mut candidates = Vec::new();
            index.for_each_candidate(direction, lower, upper, |face| candidates.push(face));
            for (face, triangle) in triangles.iter().enumerate() {
                let projected = triangle.map(|vertex| {
                    let point = positions[vertex as usize];
                    [point[0], point[1] * direction[2] - point[2] * direction[1]]
                });
                let triangle_lower: [f32; 2] = std::array::from_fn(|axis| {
                    projected
                        .iter()
                        .map(|point| point[axis])
                        .fold(f32::INFINITY, f32::min)
                });
                let triangle_upper: [f32; 2] = std::array::from_fn(|axis| {
                    projected
                        .iter()
                        .map(|point| point[axis])
                        .fold(f32::NEG_INFINITY, f32::max)
                });
                let overlaps = (0..2).all(|axis| {
                    triangle_lower[axis] <= upper[axis] && triangle_upper[axis] >= lower[axis]
                });
                assert!(
                    !overlaps || candidates.contains(&face),
                    "missed face {face}"
                );
            }
        }
    }
}
