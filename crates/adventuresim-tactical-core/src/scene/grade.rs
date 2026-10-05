//! Bound the gradient of the triangles shared by rendering and collision.
use thiserror::Error;

/// A finite work bound; exhausting it rejects preparation rather than publishing
/// a surface whose advertised grade differs from its physical bearing faces.
const MAXIMUM_REPAIR_SWEEPS: usize = 1_024;
const CARDINAL_PRECONDITION_SWEEPS: usize = 4;
const HEIGHTMAP_ROUNDOFF_ULPS: f64 = 4.0;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum TerrainGradeError {
    #[error("grade repair requires a finite sampled heightmap and nonnegative grade")]
    InvalidInput,
    #[error("selected owned support cannot be rewritten as a sampled heightmap")]
    OwnedSurface,
    #[error(
        "terrain triangle at sample {sample:?} has grade {measured}, above {permitted}, after {sweeps} repair sweeps"
    )]
    Convergence {
        sample: [usize; 2],
        measured: f64,
        permitted: f32,
        sweeps: usize,
    },
}

pub(crate) fn constrain(
    heights: &mut [f32],
    width: usize,
    depth: usize,
    spacing: f32,
    maximum_grade: f32,
) -> Result<(), TerrainGradeError> {
    if width < 2
        || depth < 2
        || width.checked_mul(depth) != Some(heights.len())
        || !spacing.is_finite()
        || spacing <= 0.0
        || !maximum_grade.is_finite()
        || maximum_grade < 0.0
        || heights.iter().any(|h| !h.is_finite())
    {
        return Err(TerrainGradeError::InvalidInput);
    }
    let bound = f64::from(spacing) * f64::from(maximum_grade);
    let mut values: Vec<_> = heights.iter().map(|h| f64::from(*h)).collect();
    let magnitude = values
        .iter()
        .fold(f64::from(spacing), |m, h| m.max(h.abs()));
    let roundoff = magnitude * f64::from(f32::EPSILON) * HEIGHTMAP_ROUNDOFF_ULPS;
    let (_, initial) = worst(&values, width, depth, None);
    if initial <= bound + roundoff {
        // Geographic relief already satisfying the physical limit retains every
        // original bit, including axis-aligned slopes near the permitted bound.
        return Ok(());
    }

    precondition(&mut values, width, depth, bound);
    let target = (bound - roundoff).max(0.0);
    let mut dirty = vec![true; (width - 1) * (depth - 1)];
    for sweep in 0..MAXIMUM_REPAIR_SWEEPS {
        repair_sweep(
            &mut values,
            &mut dirty,
            [width, depth],
            target,
            SweepDirection::for_pass(sweep),
        );
        // Clean cells were proved valid at their last visit and have no
        // subsequently changed vertex. Only dirty cells can prevent convergence.
        let (_, residual) = worst(&values, width, depth, Some(&dirty));
        if residual <= target + roundoff * 0.5 {
            for (height, value) in heights.iter_mut().zip(values) {
                *height = value as f32;
            }
            return Ok(());
        }
    }
    let (sample, measured) = worst(&values, width, depth, None);
    Err(TerrainGradeError::Convergence {
        sample: sample.unwrap_or([0, 0]),
        measured: measured / f64::from(spacing),
        permitted: maximum_grade,
        sweeps: MAXIMUM_REPAIR_SWEEPS,
    })
}

#[derive(Clone, Copy)]
enum SweepDirection {
    Forward,
    Reverse,
}
impl SweepDirection {
    fn for_pass(pass: usize) -> Self {
        if pass.is_multiple_of(2) {
            Self::Forward
        } else {
            Self::Reverse
        }
    }
    fn index(self, offset: usize, vertices: usize) -> usize {
        match self {
            Self::Forward => offset,
            Self::Reverse => vertices - 2 - offset,
        }
    }
}

// A triangle can change only when one of its shared vertices changes. Dirty
// cells retain the original raster order, including changes earlier in this
// same pass; skipping a clean cell therefore skips only a proven no-op.
fn repair_sweep(
    values: &mut [f64],
    dirty: &mut [bool],
    [width, depth]: [usize; 2],
    target: f64,
    direction: SweepDirection,
) {
    for row in 0..depth - 1 {
        let z = direction.index(row, depth);
        for column in 0..width - 1 {
            let x = direction.index(column, width);
            let cell = z * (width - 1) + x;
            if !dirty[cell] {
                continue;
            }
            dirty[cell] = false;
            let i = z * width + x;
            let first = project(values, [i, i + 1, i + width], target);
            let second = project(values, [i + width + 1, i + width, i + 1], target);
            if first || second {
                for adjacent_z in z.saturating_sub(1)..=(z + 1).min(depth - 2) {
                    for adjacent_x in x.saturating_sub(1)..=(x + 1).min(width - 2) {
                        dirty[adjacent_z * (width - 1) + adjacent_x] = true;
                    }
                }
            }
        }
    }
}

/// Retain the established cardinal repair as a deterministic preconditioner.
/// It bounds isolated source-cell jumps before the physical face projection;
/// it is insufficient by itself because two permitted axes can form a steep face.
fn precondition(values: &mut [f64], width: usize, depth: usize, bound: f64) {
    let clamp = |values: &mut [f64], source: usize, target: usize| {
        values[target] = values[target].clamp(values[source] - bound, values[source] + bound);
    };
    for _ in 0..CARDINAL_PRECONDITION_SWEEPS {
        for z in 0..depth {
            for x in 1..width {
                clamp(values, z * width + x - 1, z * width + x);
            }
            for x in (0..width - 1).rev() {
                clamp(values, z * width + x + 1, z * width + x);
            }
        }
        for x in 0..width {
            for z in 1..depth {
                clamp(values, (z - 1) * width + x, z * width + x);
            }
            for z in (0..depth - 1).rev() {
                clamp(values, (z + 1) * width + x, z * width + x);
            }
        }
    }
}

fn worst(
    values: &[f64],
    width: usize,
    depth: usize,
    dirty: Option<&[bool]>,
) -> (Option<[usize; 2]>, f64) {
    let mut sample = None;
    let mut maximum = 0.0_f64;
    for z in 0..depth - 1 {
        for x in 0..width - 1 {
            if dirty.is_some_and(|cells| !cells[z * (width - 1) + x]) {
                continue;
            }
            let i = z * width + x;
            let [a, b, c, d] = [
                values[i],
                values[i + 1],
                values[i + width],
                values[i + width + 1],
            ];
            let first = (b - a) * (b - a) + (c - a) * (c - a);
            let second = (b - d) * (b - d) + (c - d) * (c - d);
            let magnitude_squared = first.max(second);
            if magnitude_squared > maximum {
                maximum = magnitude_squared;
                sample = Some([x, z]);
            }
        }
    }
    (sample, libm::sqrt(maximum))
}

#[inline]
fn project(values: &mut [f64], [a, b, c]: [usize; 3], bound: f64) -> bool {
    let u = values[b] - values[a];
    let v = values[c] - values[a];
    let norm_squared = u * u + v * v;
    if norm_squared < bound * bound {
        return false;
    }
    let norm = libm::sqrt(norm_squared);
    if norm <= bound {
        return false;
    }
    let mean = (values[a] + values[b] + values[c]) / 3.0;
    // In the triangle's height metric the sum/difference gradient axes have
    // inverse eigenvalues 3 and 1. Solving their multiplier gives the nearest
    // three-height projection; radial scaling is not an orthogonal projection
    // and can cycle when adjacent triangles share vertices.
    let plus = (u + v) * 0.5;
    let minus = (u - v) * 0.5;
    let mut lower = 0.0;
    let mut upper = if bound > 0.0 {
        norm / bound
    } else {
        f64::INFINITY
    };
    let (p, m) = if bound > 0.0 {
        for _ in 0..f32::MANTISSA_DIGITS {
            let multiplier = (lower + upper) * 0.5;
            let p = plus / (1.0 + 3.0 * multiplier);
            let m = minus / (1.0 + multiplier);
            if 2.0 * (p * p + m * m) > bound * bound {
                lower = multiplier;
            } else {
                upper = multiplier;
            }
        }
        (plus / (1.0 + 3.0 * upper), minus / (1.0 + upper))
    } else {
        (0.0, 0.0)
    };
    values[a] = mean - 2.0 * p / 3.0;
    values[b] = values[a] + p + m;
    values[c] = values[a] + p - m;
    true
}

#[cfg(test)]
mod tests;
