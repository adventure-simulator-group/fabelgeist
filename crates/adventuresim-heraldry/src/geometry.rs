//! Closed display support, in metres, using the artwork's exact outline.
use crate::{Error, composition, document::Document};
const OUTLINE_SEGMENTS: usize = 192;
const RADIAL_SEGMENTS: usize = 24;
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
    pub tangents: Vec<[f32; 4]>,
    pub front: Vec<u32>,
    pub support: Vec<u32>,
}
impl Mesh {
    pub fn generate(d: &Document) -> Result<Self, Error> {
        d.validate()?;
        let outline = resample(&composition::outline(&d.surface).flattened(64));
        let mut mesh = Self::default();
        mesh.face(d, &outline, Side::Front);
        mesh.face(d, &outline, Side::Back);
        mesh.edge(d, &outline);
        Ok(mesh)
    }
    fn vertex(&mut self, p: [f32; 3], normal: [f32; 3], uv: [f32; 2], tangent: [f32; 4]) -> u32 {
        let index = self.positions.len() as u32;
        self.positions.push(p);
        self.normals.push(normal);
        self.uv.push(uv);
        self.tangents.push(tangent);
        index
    }
    fn face(&mut self, d: &Document, outline: &[[f32; 2]], side: Side) {
        let base = self.positions.len() as u32;
        self.face_vertex(d, [0.5; 2], side);
        for ring in 1..=RADIAL_SEGMENTS {
            for p in outline {
                let t = ring as f32 / RADIAL_SEGMENTS as f32;
                self.face_vertex(d, p.map(|v| 0.5 + (v - 0.5) * t), side);
            }
        }
        let mut indices = Vec::new();
        for i in 0..OUTLINE_SEGMENTS {
            let next = (i + 1) % OUTLINE_SEGMENTS;
            indices.extend([base, base + 1 + next as u32, base + 1 + i as u32]);
            for ring in 1..RADIAL_SEGMENTS {
                let a = base + 1 + ((ring - 1) * OUTLINE_SEGMENTS + i) as u32;
                let b = base + 1 + ((ring - 1) * OUTLINE_SEGMENTS + next) as u32;
                let c = b + OUTLINE_SEGMENTS as u32;
                let e = a + OUTLINE_SEGMENTS as u32;
                indices.extend([a, b, c, a, c, e]);
            }
        }
        match side {
            Side::Front => self.front = indices,
            Side::Back => {
                for t in indices.as_chunks_mut::<3>().0 {
                    t.swap(1, 2);
                }
                self.support = indices;
            }
        }
    }
    fn face_vertex(&mut self, d: &Document, uv: [f32; 2], side: Side) {
        let slope = -8.0 * d.surface.curvature.0 * (uv[0] - 0.5) / d.surface.width.0;
        let length = (1.0 + slope * slope).sqrt();
        let sign = match side {
            Side::Front => 1.0,
            Side::Back => -1.0,
        };
        self.vertex(
            position(d, uv, side),
            [-slope / length * sign, 0.0, sign / length],
            uv,
            [1.0 / length, 0.0, slope / length, sign],
        );
    }
    fn edge(&mut self, d: &Document, outline: &[[f32; 2]]) {
        for i in 0..outline.len() {
            let a = outline[i];
            let b = outline[(i + 1) % outline.len()];
            let pa = position(d, a, Side::Front);
            let pb = position(d, b, Side::Front);
            let dx = pb[0] - pa[0];
            let dy = pb[1] - pa[1];
            let length = dx.hypot(dy);
            let normal = [-dy / length, dx / length, 0.0];
            let tangent = [dx / length, dy / length, 0.0, 1.0];
            let start = self.vertex(pa, normal, [0.0, 0.0], tangent);
            self.vertex(pb, normal, [1.0, 0.0], tangent);
            self.vertex(position(d, b, Side::Back), normal, [1.0, 1.0], tangent);
            self.vertex(position(d, a, Side::Back), normal, [0.0, 1.0], tangent);
            self.support
                .extend([start, start + 1, start + 2, start, start + 2, start + 3]);
        }
    }
}
#[derive(Clone, Copy)]
enum Side {
    Front,
    Back,
}
fn position(d: &Document, [u, v]: [f32; 2], side: Side) -> [f32; 3] {
    let s = &d.surface;
    let z = s.curvature.0 * (1.0 - (u * 2.0 - 1.0).powi(2))
        - match side {
            Side::Front => 0.0,
            Side::Back => s.thickness.0,
        };
    [
        (u - 0.5) * s.width.metres(),
        (0.5 - v) * s.height.metres(),
        crate::document::Millimeters(z).metres(),
    ]
}
fn resample(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut lengths = vec![0.0];
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        lengths.push(lengths.last().unwrap() + (b[0] - a[0]).hypot(b[1] - a[1]));
    }
    let total = *lengths.last().unwrap();
    let mut edge = 0;
    (0..OUTLINE_SEGMENTS)
        .map(|i| {
            let along = total * i as f32 / OUTLINE_SEGMENTS as f32;
            while lengths[edge + 1] < along {
                edge += 1;
            }
            let t = (along - lengths[edge]) / (lengths[edge + 1] - lengths[edge]);
            let a = points[edge];
            let b = points[(edge + 1) % points.len()];
            [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
        })
        .collect()
}
