//! Sorted groove landmarks follow the actual recessed face, including inactive lanes.
use super::*;
struct GrooveCut {
    center: f64,
    mouth: f64,
    floor: f64,
    depth: f64,
}
impl GrooveCut {
    fn at(&self, x: f64) -> f64 {
        let distance = (x - self.center).abs();
        if distance >= self.mouth {
            0.0
        } else if distance <= self.floor {
            self.depth
        } else {
            self.depth * (self.mouth - distance) / (self.mouth - self.floor)
        }
    }
}
pub(super) fn section(
    f: &FullerParameters,
    y: f64,
    w: f64,
    d: f64,
    minimum_envelope: f64,
) -> Vec<PlanarPoint> {
    let flat = w * (1.0 - f.bevel_width_ratio.get());
    let mut result = vec![[-w, 0.0]];
    for front in [true, false] {
        let sign = if front { 1.0 } else { -1.0 };
        let cuts: Vec<_> = f
            .grooves
            .iter()
            .filter(|g| g.on_face(front))
            .map(|g| {
                let q = g.envelope(y);
                let q = if q < minimum_envelope { 0.0 } else { q };
                let mouth = g.mouth_width.get() * q / 2.0;
                GrooveCut {
                    center: flat * g.lateral_position.get(),
                    mouth,
                    floor: mouth * g.floor_width_ratio.get(),
                    depth: g.depth.get() * q * q,
                }
            })
            .collect();
        let mut face = vec![[-flat, sign * d]];
        for (index, c) in cuts.iter().enumerate() {
            for (x, depth) in [
                (c.center - c.mouth, 0.0),
                (c.center - c.floor, c.depth),
                (c.center + c.floor, c.depth),
                (c.center + c.mouth, 0.0),
            ] {
                let cut = depth
                    + cuts
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != index)
                        .map(|(_, other)| other.at(x))
                        .sum::<f64>();
                face.push([x, sign * (d - cut)]);
            }
        }
        face.push([flat, sign * d]);
        face.sort_by(|a, b| a[0].total_cmp(&b[0]));
        if !front {
            face.reverse();
        }
        result.extend(face);
        result.push([if front { w } else { -w }, 0.0]);
    }
    result.pop();
    result
}
