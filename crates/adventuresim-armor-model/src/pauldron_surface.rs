//! One fitted carrier shared by all proximal plates, preserving their overlaps.
use super::{PauldronDesign, mesh::Saddle};
use crate::{GenerateError, PartFrame, PartMesh};

const COLUMNS: usize = 96;
const ROWS: usize = 64;
const CLEARANCE_SMOOTHING_PASSES: usize = 180;
const CLEARANCE_DIFFUSION: f32 = 0.65;
const NORMAL_SAMPLE_STEP: f32 = 0.001;

/// A shoulder saddle in a semantic arm frame. Fitting happens before the
/// carrier is divided into overlapping plates, so adjacent lames cannot be
/// independently projected onto the same skin surface.
pub struct PauldronCarrier {
    pub(super) design: PauldronDesign,
    pub(super) frame: PartFrame,
    points: Vec<[f32; 3]>,
    formed: Vec<[f32; 3]>,
}

impl PauldronCarrier {
    pub fn new(design: &PauldronDesign, frame: &PartFrame) -> Result<Self, GenerateError> {
        crate::LimbArmorDesign::Pauldron(design.clone()).validate()?;
        frame.validate()?;
        let saddle = Saddle::new(design, frame)?;
        let points: Vec<[f32; 3]> = (0..=ROWS)
            .flat_map(|row| (0..=COLUMNS).map(move |column| (column, row)))
            .map(|(column, row)| {
                saddle.point(column as f32 / COLUMNS as f32, row as f32 / ROWS as f32)
            })
            .collect();
        Ok(Self {
            design: design.clone(),
            frame: *frame,
            formed: points.clone(),
            points,
        })
    }

    /// Apply body clearance in reference-body metre coordinates, then smoothly
    /// spread required displacement without reducing its magnitude. This is a
    /// sampled envelope, not a continuous collision guarantee.
    pub fn fit(
        &mut self,
        mut project: impl FnMut([f32; 3]) -> [f32; 3],
    ) -> Result<(), GenerateError> {
        let mut displacement = self
            .points
            .iter()
            .map(|p| {
                let world = self.frame.point(*p);
                let fitted = project(world);
                let delta = std::array::from_fn(|i| fitted[i] - world[i]);
                self.frame.axes.map(|axis| dot(axis, delta))
            })
            .collect::<Vec<_>>();
        if displacement.iter().flatten().any(|v| !v.is_finite()) {
            return Err(GenerateError::InvalidSurface);
        }
        for _ in 0..CLEARANCE_SMOOTHING_PASSES {
            let mut next = displacement.clone();
            for row in 0..=ROWS {
                for column in 0..=COLUMNS {
                    let index = row * (COLUMNS + 1) + column;
                    for axis in 0..3 {
                        let neighbors = [
                            index.saturating_sub(1).max(row * (COLUMNS + 1)),
                            (index + 1).min(row * (COLUMNS + 1) + COLUMNS),
                            if row > 0 { index - COLUMNS - 1 } else { index },
                            if row < ROWS {
                                index + COLUMNS + 1
                            } else {
                                index
                            },
                        ];
                        let average =
                            neighbors.map(|i| displacement[i][axis]).iter().sum::<f32>() * 0.25;
                        if average * displacement[index][axis] >= 0.0
                            && average.abs() > displacement[index][axis].abs()
                        {
                            next[index][axis] = displacement[index][axis]
                                * (1.0 - CLEARANCE_DIFFUSION)
                                + average * CLEARANCE_DIFFUSION;
                        }
                    }
                }
            }
            displacement = next;
        }
        for (point, delta) in self.points.iter_mut().zip(displacement) {
            *point = std::array::from_fn(|i| point[i] + delta[i]);
        }
        Ok(())
    }

    pub fn mesh(&self) -> Result<PartMesh, GenerateError> {
        super::mesh::plates(self)
    }

    pub(super) fn point(&self, u: f32, v: f32, offset: f32) -> [f32; 3] {
        let p = self.sample(u, v);
        if offset == 0.0 {
            return p;
        }
        // Layer separation follows the formed plate chart. Clearance projection
        // must not rotate this offset between identity samples: extrapolating
        // those rotations can fold a neck lame back through its own return.
        let du = difference(
            Self::interpolate(&self.formed, (u + NORMAL_SAMPLE_STEP).min(1.0), v),
            Self::interpolate(&self.formed, (u - NORMAL_SAMPLE_STEP).max(0.0), v),
        );
        let dv = difference(
            Self::interpolate(&self.formed, u, (v + NORMAL_SAMPLE_STEP).min(1.0)),
            Self::interpolate(&self.formed, u, (v - NORMAL_SAMPLE_STEP).max(0.0)),
        );
        let normal = [
            du[1] * dv[2] - du[2] * dv[1],
            du[2] * dv[0] - du[0] * dv[2],
            du[0] * dv[1] - du[1] * dv[0],
        ];
        let length = dot(normal, normal).sqrt();
        std::array::from_fn(|i| p[i] + normal[i] / length * offset)
    }

    fn sample(&self, u: f32, v: f32) -> [f32; 3] {
        Self::interpolate(&self.points, u, v)
    }

    fn interpolate(points: &[[f32; 3]], u: f32, v: f32) -> [f32; 3] {
        let x = (u * COLUMNS as f32) as usize;
        let y = (v * ROWS as f32) as usize;
        let x = x.min(COLUMNS - 1);
        let y = y.min(ROWS - 1);
        let a = u * COLUMNS as f32 - x as f32;
        let b = v * ROWS as f32 - y as f32;
        let index = y * (COLUMNS + 1) + x;
        std::array::from_fn(|i| {
            points[index][i] * (1.0 - a) * (1.0 - b)
                + points[index + 1][i] * a * (1.0 - b)
                + points[index + COLUMNS + 1][i] * (1.0 - a) * b
                + points[index + COLUMNS + 2][i] * a * b
        })
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn difference(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_fitting_directions_cannot_smooth_a_required_offset_inward() {
        let diagonal = std::f32::consts::FRAC_1_SQRT_2;
        let frame = PartFrame {
            origin: [0.18, 1.42, -0.03],
            axes: [
                [0.0, 0.0, -1.0],
                [-diagonal, diagonal, 0.0],
                [diagonal, diagonal, 0.0],
            ],
            half_extents: [0.083, 0.061, 0.058],
        };
        let mut carrier = PauldronCarrier::new(&PauldronDesign::default(), &frame).unwrap();
        let index = (ROWS / 2) * (COLUMNS + 1) + COLUMNS / 2;
        let original = frame.point(carrier.points[index]);
        carrier
            .fit(|mut point| {
                let isolated = point == original;
                point[2] += if isolated { 0.002 } else { -0.006 };
                point
            })
            .unwrap();
        assert!(frame.point(carrier.points[index])[2] - original[2] >= 0.002 - 1e-6);
    }
    #[test]
    fn clearance_projection_preserves_the_formed_lame_separation() {
        let design = PauldronDesign::default();
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[0.0, 0.0, 1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            half_extents: [0.07, 0.16, 0.065],
        };
        let mut carrier = PauldronCarrier::new(&design, &frame).unwrap();
        let samples = [[0.0, 0.9], [0.2, 0.85], [0.8, 0.95], [1.0, 0.9]];
        let separation = |carrier: &PauldronCarrier, [u, v]: [f32; 2]| {
            difference(carrier.point(u, v, 0.006), carrier.point(u, v, 0.0))
        };
        let expected = samples.map(|uv| separation(&carrier, uv));
        carrier
            .fit(|p| [p[0], p[1], p[2] + 0.03 * (12.0 * p[0]).sin()])
            .unwrap();
        for (sample, expected) in samples.into_iter().zip(expected) {
            let actual = separation(&carrier, sample);
            for axis in 0..3 {
                assert!((actual[axis] - expected[axis]).abs() < 1e-7);
            }
            assert!((dot(actual, actual).sqrt() - 0.006).abs() < 1e-7);
        }
    }
}
