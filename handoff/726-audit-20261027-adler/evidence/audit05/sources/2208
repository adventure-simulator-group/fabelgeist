use crate::Plate;
use fabelgeist_garment_code::garment::edge::{Edge, EdgeKind, EdgeSequence, linearize};

// GarmentCode patterns use centimetres. Armor dimensions and mesh output use metres.
pub fn plate_edges(p: &Plate) -> EdgeSequence {
    let w = p.width as f64 * 100.0;
    let h = p.height as f64 * 100.0;
    let r = p.roundness as f64 * w.min(h) * 0.48;
    let mut edges = vec![Edge::line([-w * 0.5, h * 0.5], [-w * 0.5, -h * 0.5 + r])];
    if r > 1e-5 {
        let k = 0.5522847498307936;
        edges.push(Edge::curve(
            [-w * 0.5, -h * 0.5 + r],
            [-w * 0.5 + r, -h * 0.5],
            vec![
                [-w * 0.5, -h * 0.5 + r * (1.0 - k)],
                [-w * 0.5 + r * (1.0 - k), -h * 0.5],
            ],
            false,
        ));
        edges.push(Edge::line(
            [-w * 0.5 + r, -h * 0.5],
            [w * 0.5 - r, -h * 0.5],
        ));
        edges.push(Edge::curve(
            [w * 0.5 - r, -h * 0.5],
            [w * 0.5, -h * 0.5 + r],
            vec![
                [w * 0.5 - r * (1.0 - k), -h * 0.5],
                [w * 0.5, -h * 0.5 + r * (1.0 - k)],
            ],
            false,
        ));
    } else {
        edges.push(Edge::line([-w * 0.5, -h * 0.5], [w * 0.5, -h * 0.5]));
    }
    edges.push(Edge::line([w * 0.5, -h * 0.5 + r], [w * 0.5, h * 0.5]));
    edges.push(Edge::line([w * 0.5, h * 0.5], [-w * 0.5, h * 0.5]));
    EdgeSequence::from_edges(edges)
}
pub fn plate_outline(p: &Plate) -> Vec<[f32; 2]> {
    plate_edges(p)
        .edges
        .iter()
        .flat_map(|e| {
            let samples = if matches!(e.borrow().kind, EdgeKind::Line) {
                0
            } else {
                7
            };
            linearize(e, samples)
                .edges
                .iter()
                .map(|segment| {
                    let p = segment.borrow().start_p();
                    [(p[0] * 0.01) as f32, (p[1] * 0.01) as f32]
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn garment_code_preserves_dimensions_and_roundness() {
        let mut p = crate::Tiling::scale().plate;
        for roundness in [0.0, 0.5, 1.0] {
            p.roundness = roundness;
            let outline = plate_outline(&p);
            for axis in 0..2 {
                let lo = outline
                    .iter()
                    .map(|p| p[axis])
                    .fold(f32::INFINITY, f32::min);
                let hi = outline
                    .iter()
                    .map(|p| p[axis])
                    .fold(f32::NEG_INFINITY, f32::max);
                assert!((hi - lo - [p.width, p.height][axis]).abs() < 1e-6);
            }
            assert_eq!(outline.len(), if roundness == 0.0 { 4 } else { 20 });
        }
    }
}
