//! Projection solve for a shared cloth displacement field. The reference
//! gradient bound remains fixed while contact planes are refreshed.
use super::{CLEARANCE_TOLERANCE_M, LayerConstraint};
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;

const SEATING_ITERATIONS: usize = 2048;
// A norm below one preserves each cell's in-plane orientation.
const MAXIMUM_DISPLACEMENT_GRADIENT: f32 = 0.5;
const GRADIENT_CONVERGENCE_TOLERANCE: f32 = 0.0001;

pub(super) struct SeatingSolver<'a> {
    reference: &'a [[f32; 3]],
    directions: &'a [[f32; 3]],
    field: Vec<FieldTriangle>,
}

struct FieldTriangle {
    vertices: [u32; 3],
    gradients: [Vec3; 3],
}

impl FieldTriangle {
    fn new(vertices: [u32; 3], positions: &[[f32; 3]]) -> Result<Self> {
        let [a, b, c] = vertices.map(|v| Vec3::from_array(positions[v as usize]));
        let normal = (b - a).cross(c - a);
        let squared = normal.length_squared();
        ensure!(
            squared > 0. && squared.is_finite(),
            "clearance field has a degenerate cell"
        );
        let gradient_b = (c - a).cross(normal) / squared;
        let gradient_c = normal.cross(b - a) / squared;
        Ok(Self {
            vertices,
            gradients: [-gradient_b - gradient_c, gradient_b, gradient_c],
        })
    }

    fn gradient_norm(&self, displacement: &[Vec3]) -> f32 {
        let shifts = self.vertices.map(|v| displacement[v as usize]);
        // Remove constant translation before arithmetic. Thin cells must not
        // amplify cancellation of absolute shifts in barycentric gradients.
        (0..3)
            .map(|axis| {
                ((shifts[1] - shifts[0]) * self.gradients[1][axis]
                    + (shifts[2] - shifts[0]) * self.gradients[2][axis])
                    .length_squared()
            })
            .sum::<f32>()
            .sqrt()
    }

    fn residual(&self, displacement: &[Vec3]) -> f32 {
        self.gradient_norm(displacement) - MAXIMUM_DISPLACEMENT_GRADIENT
    }

    fn project(&self, displacement: &mut [Vec3]) {
        let norm = self.gradient_norm(displacement);
        if norm <= MAXIMUM_DISPLACEMENT_GRADIENT {
            return;
        }
        let mean = self
            .vertices
            .into_iter()
            .map(|v| displacement[v as usize])
            .sum::<Vec3>()
            / 3.;
        let scale = MAXIMUM_DISPLACEMENT_GRADIENT / norm;
        for vertex in self.vertices {
            let shift = &mut displacement[vertex as usize];
            *shift = mean + (*shift - mean) * scale;
        }
    }
}

impl<'a> SeatingSolver<'a> {
    pub(super) fn new(
        reference: &'a [[f32; 3]],
        faces: &[[u32; 3]],
        directions: &'a [[f32; 3]],
    ) -> Result<Self> {
        let field = faces
            .iter()
            .map(|&face| FieldTriangle::new(face, reference))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            reference,
            directions,
            field,
        })
    }

    pub(super) fn solve(
        &self,
        constraints: &[LayerConstraint],
        displacement: &mut [Vec3],
    ) -> Result<()> {
        for iteration in 0..=SEATING_ITERATIONS {
            let maximum_gap = constraints
                .iter()
                .map(|constraint| constraint.residual(displacement))
                .fold(0_f32, f32::max);
            let field_gap = self
                .field
                .iter()
                .map(|cell| cell.residual(displacement))
                .fold(0., f32::max);
            let outward_gap = displacement
                .iter()
                .zip(self.directions)
                .map(|(shift, direction)| {
                    let direction = Vec3::from_array(*direction);
                    let length = direction.length();
                    if length > 0. {
                        (-shift.dot(direction) / length).max(0.)
                    } else {
                        shift.length()
                    }
                })
                .fold(0., f32::max);
            if outward_gap <= CLEARANCE_TOLERANCE_M
                && maximum_gap <= CLEARANCE_TOLERANCE_M
                && field_gap <= GRADIENT_CONVERGENCE_TOLERANCE
            {
                break;
            }
            if iteration == SEATING_ITERATIONS {
                let worst = constraints
                    .iter()
                    .max_by(|a, b| {
                        a.residual(displacement)
                            .total_cmp(&b.residual(displacement))
                    })
                    .context("no failed layer constraint")?;
                anyhow::bail!(
                    "frozen inner-layer constraints did not converge: maximum gap={maximum_gap}m outward gap={outward_gap}m displacement gradient excess={field_gap} required={}m sample={:?} hit={:?} query={:?} face={:?} coefficients={:?}",
                    worst.required_metres,
                    worst.sample,
                    worst.hit,
                    worst.query_direction,
                    worst
                        .coefficients
                        .iter()
                        .map(|term| self.reference[term.vertex as usize])
                        .collect::<Vec<_>>(),
                    worst.coefficients
                );
            }
            for constraint in constraints {
                constraint.project(displacement)?;
            }
            for cell in &self.field {
                cell.project(displacement);
            }
            // Keep the anatomical outward half-space without constraining all
            // motion to that one ray. Shared translations can now remain smooth
            // across shoulders whose anatomical directions differ sharply.
            for (shift, direction) in displacement.iter_mut().zip(self.directions) {
                let direction = Vec3::from_array(*direction);
                let inward = shift.dot(direction);
                if inward < 0. {
                    *shift -= direction * (inward / direction.length_squared());
                }
            }
        }
        Ok(())
    }
}
