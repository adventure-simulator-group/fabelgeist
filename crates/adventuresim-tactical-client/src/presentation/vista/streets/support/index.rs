//! Immutable bounds hierarchy over the presented ground triangles.
use super::*;

const TRIANGLES_PER_LEAF: usize = 8;

#[derive(Clone)]
pub(super) struct TriangleIndex {
    order: Vec<usize>,
    nodes: Vec<Node>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hierarchy_finds_crossing_triangles_without_scanning_the_whole_city() {
        let mut triangles: Vec<_> = (0..1000)
            .map(|x| {
                let origin = Vec3::X * x as f32 * 2.0;
                [origin, origin + Vec3::Z, origin + Vec3::X]
            })
            .collect();
        triangles.push([Vec3::new(-100.0, 0.0, -1.0), Vec3::Z, Vec3::X * 100.0]);
        let index = TriangleIndex::new(&triangles);
        let mut candidates = Vec::new();
        index.query(Vec2::splat(-0.5), Vec2::splat(0.5), |index| {
            candidates.push(index)
        });
        assert!(candidates.contains(&0));
        assert!(candidates.contains(&1000));
        assert!(
            candidates.len() < 32,
            "local query must reject distant leaves"
        );
        for x in [-100.0, 0.0, 50.0, 999.0, 2000.0] {
            let min = Vec2::new(x, -0.5);
            let max = min + Vec2::ONE;
            let mut candidates = Vec::new();
            index.query(min, max, |index| candidates.push(index));
            for (i, triangle) in triangles.iter().enumerate() {
                let tri_min = triangle
                    .iter()
                    .map(|p| p.xz())
                    .fold(Vec2::splat(f32::INFINITY), Vec2::min);
                let tri_max = triangle
                    .iter()
                    .map(|p| p.xz())
                    .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
                if tri_max.cmpge(min).all() && tri_min.cmple(max).all() {
                    assert!(candidates.contains(&i));
                }
            }
        }
    }
}

#[derive(Clone)]
struct Node {
    minimum: Vec2,
    maximum: Vec2,
    range: std::ops::Range<usize>,
    children: Option<(usize, usize)>,
}

impl TriangleIndex {
    pub fn new(triangles: &[[Vec3; 3]]) -> Self {
        let bounds: Vec<_> = triangles
            .iter()
            .map(|triangle| {
                triangle.iter().fold(
                    (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                    |(min, max), point| (min.min(point.xz()), max.max(point.xz())),
                )
            })
            .collect();
        let mut index = Self {
            order: (0..triangles.len()).collect(),
            nodes: Vec::new(),
        };
        index.partition(0..triangles.len(), &bounds);
        index
    }

    fn partition(&mut self, range: std::ops::Range<usize>, bounds: &[(Vec2, Vec2)]) -> usize {
        let (minimum, maximum) = self.order[range.clone()].iter().fold(
            (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
            |(min, max), &index| (min.min(bounds[index].0), max.max(bounds[index].1)),
        );
        let node = self.nodes.len();
        self.nodes.push(Node {
            minimum,
            maximum,
            range: range.clone(),
            children: None,
        });
        if range.len() > TRIANGLES_PER_LEAF {
            let extent = maximum - minimum;
            let axis = usize::from(extent.y > extent.x);
            let middle = range.len() / 2;
            self.order[range.clone()].select_nth_unstable_by(middle, |&a, &b| {
                (bounds[a].0[axis] + bounds[a].1[axis])
                    .total_cmp(&(bounds[b].0[axis] + bounds[b].1[axis]))
                    .then_with(|| a.cmp(&b))
            });
            let split = range.start + middle;
            let left = self.partition(range.start..split, bounds);
            let right = self.partition(split..range.end, bounds);
            self.nodes[node].children = Some((left, right));
        }
        node
    }

    pub fn query(&self, minimum: Vec2, maximum: Vec2, mut emit: impl FnMut(usize)) {
        self.visit(0, minimum, maximum, &mut emit);
    }

    fn visit(&self, index: usize, minimum: Vec2, maximum: Vec2, emit: &mut impl FnMut(usize)) {
        let node = &self.nodes[index];
        if !node.maximum.cmpge(minimum).all() || !node.minimum.cmple(maximum).all() {
            return;
        }
        if let Some((left, right)) = node.children {
            self.visit(left, minimum, maximum, emit);
            self.visit(right, minimum, maximum, emit);
        } else {
            for &triangle in &self.order[node.range.clone()] {
                emit(triangle);
            }
        }
    }
}
