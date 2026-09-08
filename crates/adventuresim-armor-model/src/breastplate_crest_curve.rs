//! Sparse anatomical crest interpolation and geometric validation.
use crate::breastplate_shoulder_band::{BandError, CREST_SEED_COUNT};
use serde::Serialize;
const MIN_TANGENT_LENGTH: f64 = 1e-8;

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct Cubic {
    controls: [[f64; 3]; 4],
}

impl Cubic {
    pub(crate) fn validate_exact_lateral_monotonicity(self) -> Result<(), BandError> {
        let b: [f64; 3] =
            std::array::from_fn(|i| 3.0 * (self.controls[i + 1][0] - self.controls[i][0]));
        let mut minimum = b[0].min(b[2]);
        let quadratic = b[0] - 2.0 * b[1] + b[2];
        if quadratic > 0.0 {
            let t = (b[0] - b[1]) / quadratic;
            if (0.0..=1.0).contains(&t) {
                minimum = minimum.min(self.tangent(t)[0]);
            }
        }
        // Strict positive X speed also certifies nonzero 3D speed and simple XY.
        if !minimum.is_finite() || minimum <= MIN_TANGENT_LENGTH {
            return Err(BandError::InvalidCurve);
        }
        Ok(())
    }
    /// Endpoint-preserving least squares over evenly spaced query seeds.
    pub(crate) fn fit(samples: &[[f64; 3]; CREST_SEED_COUNT]) -> Result<Self, BandError> {
        if samples.iter().flatten().any(|v| !v.is_finite()) {
            return Err(BandError::InvalidCurve);
        }
        let mut controls = [
            samples[0],
            [0.0; 3],
            [0.0; 3],
            samples[CREST_SEED_COUNT - 1],
        ];
        let mut matrix = [[0.0; 2]; 2];
        let mut rhs = [[0.0; 3]; 2];
        for (i, sample) in samples.iter().enumerate() {
            let t = i as f64 / (CREST_SEED_COUNT - 1) as f64;
            let s = 1.0 - t;
            let weights = [s.powi(3), 3.0 * s * s * t, 3.0 * s * t * t, t.powi(3)];
            for j in 0..2 {
                for k in 0..2 {
                    matrix[j][k] += weights[j + 1] * weights[k + 1];
                }
                for axis in 0..3 {
                    rhs[j][axis] += weights[j + 1]
                        * (sample[axis]
                            - weights[0] * controls[0][axis]
                            - weights[3] * controls[3][axis]);
                }
            }
        }
        let determinant = matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0];
        for axis in 0..3 {
            controls[1][axis] =
                (rhs[0][axis] * matrix[1][1] - rhs[1][axis] * matrix[0][1]) / determinant;
            controls[2][axis] =
                (rhs[1][axis] * matrix[0][0] - rhs[0][axis] * matrix[1][0]) / determinant;
        }
        Ok(Self { controls })
    }

    pub(crate) fn at(self, t: f64) -> [f64; 3] {
        let s = 1.0 - t;
        let weights = [s.powi(3), 3.0 * s * s * t, 3.0 * s * t * t, t.powi(3)];
        std::array::from_fn(|axis| (0..4).map(|i| weights[i] * self.controls[i][axis]).sum())
    }

    pub(crate) fn tangent(self, t: f64) -> [f64; 3] {
        let weights = [(1.0 - t).powi(2), 2.0 * (1.0 - t) * t, t * t];
        std::array::from_fn(|axis| {
            (0..3)
                .map(|i| 3.0 * weights[i] * (self.controls[i + 1][axis] - self.controls[i][axis]))
                .sum()
        })
    }

    pub(crate) fn validate_lateral_monotonicity(self) -> Result<(), BandError> {
        // A positive derivative Bernstein hull certifies the entire curve,
        // not merely the sampled arc table. Reject rather than switch branch.
        if self.controls.iter().flatten().any(|v| !v.is_finite())
            || self
                .controls
                .windows(2)
                .any(|p| p[1][0] - p[0][0] <= MIN_TANGENT_LENGTH)
        {
            return Err(BandError::InvalidCurve);
        }
        Ok(())
    }
}

pub(crate) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(x, y)| x * y).sum()
}

pub(crate) fn normalized(v: [f64; 3]) -> Result<[f64; 3], BandError> {
    let length = dot(v, v).sqrt();
    if !length.is_finite() || length <= MIN_TANGENT_LENGTH {
        return Err(BandError::InvalidNormal);
    }
    Ok(v.map(|x| x / length))
}
