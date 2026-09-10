use super::plane::Plane;
use super::polygon::Polygon;

#[derive(Debug, Clone, Default)]
pub struct BspNode {
    pub plane: Option<Plane>,
    pub polygons: Vec<Polygon>,
    pub front: Option<Box<BspNode>>,
    pub back: Option<Box<BspNode>>,
}

impl BspNode {
    pub fn new(polygons: Vec<Polygon>) -> Self {
        let mut node = Self::default();
        node.build(polygons);
        node
    }

    pub fn invert(&mut self) {
        for poly in &mut self.polygons {
            poly.invert();
        }
        if let Some(ref mut p) = self.plane {
            p.invert();
        }
        if let Some(ref mut f) = self.front {
            f.invert();
        }
        if let Some(ref mut b) = self.back {
            b.invert();
        }
        std::mem::swap(&mut self.front, &mut self.back);
    }

    pub fn clip_polygons(&self, polygons: Vec<Polygon>) -> Vec<Polygon> {
        if self.plane.is_none() {
            return polygons;
        }
        let plane = self.plane.unwrap();
        let mut front = Vec::new();
        let mut back = Vec::new();

        for poly in polygons {
            let mut coplanar_front = Vec::new();
            let mut coplanar_back = Vec::new();
            plane.split_polygon(
                poly,
                &mut coplanar_front,
                &mut coplanar_back,
                &mut front,
                &mut back,
            );
            front.extend(coplanar_front);
            back.extend(coplanar_back);
        }

        if let Some(ref f) = self.front {
            front = f.clip_polygons(front);
        }
        if let Some(ref b) = self.back {
            back = b.clip_polygons(back);
        } else {
            back.clear();
        }

        front.extend(back);
        front
    }

    pub fn clip_to(&mut self, other: &BspNode) {
        self.polygons = other.clip_polygons(std::mem::take(&mut self.polygons));
        if let Some(ref mut f) = self.front {
            f.clip_to(other);
        }
        if let Some(ref mut b) = self.back {
            b.clip_to(other);
        }
    }

    pub fn all_polygons(&self) -> Vec<Polygon> {
        let mut result = Vec::new();
        self.collect_polygons(&mut result);
        result
    }

    fn collect_polygons(&self, result: &mut Vec<Polygon>) {
        result.extend(self.polygons.clone());
        if let Some(ref f) = self.front {
            f.collect_polygons(result);
        }
        if let Some(ref b) = self.back {
            b.collect_polygons(result);
        }
    }

    pub fn build(&mut self, polygons: Vec<Polygon>) {
        self.build_with_depth(polygons, 0);
    }

    fn build_with_depth(&mut self, polygons: Vec<Polygon>, depth: u32) {
        if polygons.is_empty() {
            return;
        }
        if depth > 64 {
            self.polygons.extend(polygons);
            return;
        }
        if self.plane.is_none() {
            // Pick a plane that is a good splitter.
            let mut best_plane = polygons[0].plane;
            let mut best_score = f32::MAX;

            // Choose a sample of candidates. To keep it fast, sample up to 32 polygons.
            let candidate_count = std::cmp::min(polygons.len(), 32);
            let stride = polygons.len() / candidate_count;

            for i in 0..candidate_count {
                let candidate_idx = i * stride;
                let candidate_plane = polygons[candidate_idx].plane;

                let mut front_count: i32 = 0;
                let mut back_count: i32 = 0;
                let mut split_count: i32 = 0;

                // Evaluate the candidate plane against a sample of polygons.
                let eval_count = std::cmp::min(polygons.len(), 100);
                let eval_stride = polygons.len() / eval_count;

                for j in 0..eval_count {
                    let poly_idx = j * eval_stride;
                    let poly = &polygons[poly_idx];

                    let mut has_front = false;
                    let mut has_back = false;
                    for v in &poly.vertices {
                        let t = candidate_plane.normal.dot(v.position) - candidate_plane.w;
                        if t > Plane::EPSILON {
                            has_front = true;
                        } else if t < -Plane::EPSILON {
                            has_back = true;
                        }
                    }

                    if has_front && has_back {
                        split_count += 1;
                    } else if has_front {
                        front_count += 1;
                    } else if has_back {
                        back_count += 1;
                    }
                }

                // Score: minimize splits and balance the distribution.
                let balance = (front_count - back_count).abs() as f32;
                let score = balance + (split_count as f32) * 5.0;

                if score < best_score {
                    best_score = score;
                    best_plane = candidate_plane;
                }
            }

            self.plane = Some(best_plane);
        }
        let plane = self.plane.unwrap();

        let mut front = Vec::new();
        let mut back = Vec::new();

        for poly in polygons {
            let mut coplanar_front = Vec::new();
            let mut coplanar_back = Vec::new();
            plane.split_polygon(
                poly,
                &mut coplanar_front,
                &mut coplanar_back,
                &mut front,
                &mut back,
            );
            self.polygons.extend(coplanar_front);
            self.polygons.extend(coplanar_back);
        }

        if !front.is_empty() {
            if self.front.is_none() {
                self.front = Some(Box::new(BspNode::default()));
            }
            self.front
                .as_mut()
                .unwrap()
                .build_with_depth(front, depth + 1);
        }
        if !back.is_empty() {
            if self.back.is_none() {
                self.back = Some(Box::new(BspNode::default()));
            }
            self.back
                .as_mut()
                .unwrap()
                .build_with_depth(back, depth + 1);
        }
    }
}
