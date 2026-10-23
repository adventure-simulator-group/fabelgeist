//! Fit fixed cloth triangles over inner layers in canonical unposed metres.
//! Freeze face ownership and solve outward plane constraints. These are
//! sampled plane constraints, not continuous cloth collision guarantees.
use crate::armor_frames::Wearer;
use crate::armor_layer::ArmorLayerSurface;
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;
use fabelgeist_math::Vec3 as BvhPoint;

pub(crate) use domain::CutTopology;

#[path = "cloth_layer_constraints.rs"]
mod constraints;
#[path = "cloth_layer_domain.rs"]
mod domain;
#[path = "cloth_layer_overlap.rs"]
mod overlap;
#[path = "cloth_layer_solver.rs"]
mod solver;
#[path = "cloth_layer_support.rs"]
mod support;

const CONTACT_REFINEMENTS: usize = 8;
pub(crate) const CLEARANCE_TOLERANCE_M: f32 = 0.0002;
// Lower articles already include their full material gauges. Rest fabric
// interfaces may contact: preserve authored ease, but require no extra air gap.
// Anatomical stand-off remains owned separately by cloth_panel.
pub(super) const INTERLAYER_REST_EASE_M: f32 = 0.;
/// Initial anatomy uses the complete retained native region. Later clothing
/// retains its component and anatomical ownership rules.
#[derive(Clone, Copy)]
pub(crate) enum SupportScope<'a> {
    GarmentLayers,
    RetainedAnatomy {
        clearance: fabelgeist_armor::Millimeters,
        banks: &'a crate::cloth_panel::banks::ContactBanks,
    },
}

impl SupportScope<'_> {
    pub(super) fn rest_ease_metres(self) -> f32 {
        match self {
            Self::GarmentLayers => INTERLAYER_REST_EASE_M,
            Self::RetainedAnatomy { clearance, .. } => {
                clearance
                    .metres()
                    .max(crate::cloth_panel::MINIMUM_CLOTH_CLEARANCE_M)
                    + CLEARANCE_TOLERANCE_M
            }
        }
    }
}

#[derive(Clone, Debug)]
struct DisplacementCoefficient {
    /// Native ordinal in the shared source-point displacement field.
    vertex: u32,
    /// Dimensionless weighted plane normal for a metre-space translation.
    response: Vec3,
}

#[derive(Clone, Debug)]
struct LayerConstraint {
    coefficients: Vec<DisplacementCoefficient>,
    query_direction: Vec3,
    sample: Vec3,
    hit: Vec3,
    required_metres: f32,
}

impl LayerConstraint {
    // Diagnostic support magnitude; not a convergence bound.
    fn uniform_displacement_metres(&self) -> f32 {
        let response = self
            .coefficients
            .iter()
            .map(|term| term.response.length())
            .sum::<f32>();
        if response > 0. {
            (self.required_metres / response).max(0.)
        } else {
            0.
        }
    }

    fn residual(&self, displacement: &[Vec3]) -> f32 {
        self.required_metres
            - self
                .coefficients
                .iter()
                .map(|term| displacement[term.vertex as usize].dot(term.response))
                .sum::<f32>()
    }

    fn project(&self, displacement: &mut [Vec3]) -> Result<()> {
        let missing = self.residual(displacement);
        if missing <= 0. {
            return Ok(());
        }
        let squared = self
            .coefficients
            .iter()
            .map(|term| term.response.length_squared())
            .sum::<f32>();
        ensure!(
            squared > 0.,
            "inner layer has no source response: query_direction={:?}, constraint={self:?}",
            self.query_direction
        );
        for term in &self.coefficients {
            displacement[term.vertex as usize] += missing * term.response / squared;
        }
        Ok(())
    }
}

pub(crate) fn seat(
    positions: &mut [[f32; 3]],
    faces: &[[u32; 3]],
    directions: &[[f32; 3]],
    body: &Wearer<'_>,
    layers: &[ArmorLayerSurface<'_>],
    cut: CutTopology<'_>,
    scope: SupportScope<'_>,
) -> Result<()> {
    if layers.is_empty() {
        return Ok(());
    }
    let reference = positions.to_vec();
    let solver = solver::SeatingSolver::new(&reference, faces, directions)?;
    let initial = domain::CutDomain::new(
        CutTopology {
            table: cut.table,
            faces: cut.faces,
        },
        &reference,
        directions,
    )?;
    let layer_banks = match scope {
        SupportScope::GarmentLayers => layers
            .iter()
            .map(|layer| {
                crate::cloth_panel::banks::ContactBanks::for_layer(
                    &initial.positions,
                    initial.faces,
                    layer,
                    &constraints::torso_mask(layer),
                )
            })
            .collect::<Result<Vec<_>>>()?,
        SupportScope::RetainedAnatomy { .. } => Vec::new(),
    };
    let mut displacement = vec![Vec3::ZERO; positions.len()];
    for refinement in 0..=CONTACT_REFINEMENTS {
        let domain = domain::CutDomain::new(
            CutTopology {
                table: cut.table,
                faces: cut.faces,
            },
            positions,
            directions,
        )?;
        let mut constraints = constraints::collect(&domain, body, layers, scope, &layer_banks)?;
        let gap = constraints
            .iter()
            .map(|c| c.required_metres)
            .fold(0_f32, f32::max);
        if gap <= CLEARANCE_TOLERANCE_M {
            return Ok(());
        }
        ensure!(
            refinement < CONTACT_REFINEMENTS,
            "refitted cloth contacts did not converge: remaining gap={gap}m"
        );
        for constraint in &mut constraints {
            // Rebase current contact planes onto the original displacement
            // unknowns; every solve keeps the original field gradient bound.
            constraint.required_metres += constraint
                .coefficients
                .iter()
                .map(|term| displacement[term.vertex as usize].dot(term.response))
                .sum::<f32>();
        }
        solver.solve(&constraints, &mut displacement)?;
        let mut next = reference.clone();
        apply_displacement(&mut next, faces, displacement.clone(), &constraints)?;
        positions.copy_from_slice(&next);
    }
    Ok(())
}

/// Apply canonical metre translations, then reject rotated/reversed source
/// facets before their normals can affect retained material thickness.
fn apply_displacement(
    positions: &mut [[f32; 3]],
    faces: &[[u32; 3]],
    displacement: Vec<Vec3>,
    constraints: &[LayerConstraint],
) -> Result<()> {
    let before = positions.to_vec();
    for (point, displacement) in positions.iter_mut().zip(displacement) {
        *point = (Vec3::from_array(*point) + displacement).to_array();
    }
    for face in faces {
        let [a, b, c] = face.map(|v| Vec3::from_array(before[v as usize]));
        let [next_a, next_b, next_c] = face.map(|v| Vec3::from_array(positions[v as usize]));
        let orientation = (b - a)
            .cross(c - a)
            .dot((next_b - next_a).cross(next_c - next_a));
        if !orientation.is_finite() || orientation <= 0. {
            let strongest = constraints.iter().max_by(|a, b| {
                a.uniform_displacement_metres()
                    .total_cmp(&b.uniform_displacement_metres())
            });
            anyhow::bail!(
                "inner layer fitting folds a cloth triangle: indices={face:?} before={:?} after={:?} strongest_uniform_support={strongest:?}",
                [a, b, c],
                [next_a, next_b, next_c]
            );
        }
    }
    Ok(())
}

/// A source domain can occupy a subset of the uploaded vertex array. Vertices
/// outside that domain have no fitted normal. This also excludes unreferenced
/// reserved vertices in lower articles from their support constraints.
pub(crate) fn referenced_normals(
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
) -> Result<Vec<Option<[f32; 3]>>> {
    let mut normals = vec![Vec3::ZERO; positions.len()];
    let mut referenced = vec![false; positions.len()];
    for face in faces {
        let [a, b, c] = face.map(|v| Vec3::from_array(positions[v as usize]));
        let cross = (b - a).cross(c - a);
        for &vertex in face {
            normals[vertex as usize] += cross;
            referenced[vertex as usize] = true;
        }
    }
    normals
        .into_iter()
        .zip(referenced)
        .map(|(normal, referenced)| {
            if !referenced {
                return Ok(None);
            }
            let normal = normal
                .try_normalize()
                .context("layer seating lost a referenced surface normal")?;
            Ok(Some(normal.to_array()))
        })
        .collect()
}

fn bvh([x, y, z]: [f32; 3]) -> BvhPoint {
    BvhPoint::new(x, y, z)
}
