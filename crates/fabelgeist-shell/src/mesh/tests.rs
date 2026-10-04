use super::*;
use crate::{BendPoints, BendWeights, ParticleInputCount, ParticleInputIndex, ParticleMassCount};
use fabelgeist_xpbd::{ConstraintCount, ParticleIndex};
use std::error::Error;

fn square() -> ShellMesh {
    ShellMesh::new(
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
        ],
        vec![[0, 1, 2], [1, 3, 2]],
        0.2.into(),
    )
    .expect("valid rest geometry")
}

fn nonfinite_bend() -> crate::BendRecord {
    let mesh = square();
    let points = mesh.bends[0]
        .particles()
        .map(ParticleIndex::from)
        .map(|i: ParticleIndex| -> Vec3 { mesh.positions[usize::from(i)] });
    let weights = BendWeights::for_points(BendPoints::from(points)).unwrap();
    let mut invalid = points;
    invalid[0].x = f32::NAN;
    weights.observed_rest(BendPoints::from(invalid))
}

#[test]
fn construction_keeps_density_position_and_triangle_admission_order() {
    let positions = vec![Vec3::new(f32::NAN, 0.0, 0.0)];
    let error = ShellMesh::new(positions.clone(), vec![[1, 0, 0]], (-1.0).into()).unwrap_err();
    assert!(
        matches!(error, ShellMeshError::ArealDensity { density } if density == ParticleArealDensity::from(-1.0))
    );
    assert_eq!(error.to_string(), "areal density must be positive");
    let error = ShellMesh::new(positions, vec![[1, 0, 0]], 0.2.into()).unwrap_err();
    assert!(
        matches!(error, ShellMeshError::NonFinitePosition { particle, position } if particle == ParticleInputIndex::from(0) && position.x.is_nan())
    );
    assert_eq!(error.to_string(), "non-finite shell position");
    let error = ShellMesh::new(vec![Vec3::default()], vec![[1, 0, 0]], 0.2.into()).unwrap_err();
    assert!(
        matches!(error, ShellMeshError::TriangleIndex { index, particles } if index == ParticleIndex::from(1) && particles == ParticleInputCount::from(1))
    );
    assert_eq!(error.to_string(), "shell triangle index out of bounds");
}

#[test]
fn empty_rest_geometry_is_constructible_but_not_simulable() {
    let empty = ShellMesh::new(vec![], vec![], 0.2.into()).unwrap();
    let error = empty.validate().unwrap_err();
    assert!(
        matches!(error, ShellMeshError::ParticleCount { actual } if actual == ParticleInputCount::from(0))
    );
    assert_eq!(error.to_string(), "invalid shell particle count");
    assert!(square().validate().is_ok());
}

#[test]
fn mesh_conversion_retains_triangle_decoder_and_validation_causes() {
    let mut mesh = fabelgeist_mesh::MeshData {
        positions: vec![[0.0, 0.0, 0.0]; 3],
        indices: Some(vec![0, 1]),
        topology: fabelgeist_mesh::PrimitiveTopology::LineList,
        ..Default::default()
    };
    let error = ShellMesh::from_mesh(&mesh, (-1.0).into()).unwrap_err();
    assert!(matches!(
        error,
        ShellMeshError::TriangleSource {
            source: fabelgeist_mesh::MeshTriangleError::UnsupportedTopology { .. }
        }
    ));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<fabelgeist_mesh::MeshTriangleError>()
            .is_some()
    );
    assert_eq!(error.to_string(), "operation requires triangle geometry");
    mesh.indices = Some(vec![0, 3]);
    let error = ShellMesh::from_mesh(&mesh, 0.2.into()).unwrap_err();
    let triangle_cause = error.source().unwrap();
    assert!(
        triangle_cause
            .downcast_ref::<fabelgeist_mesh::MeshTriangleError>()
            .is_some()
    );
    assert!(
        triangle_cause
            .source()
            .unwrap()
            .downcast_ref::<fabelgeist_mesh::MeshValidationError>()
            .is_some()
    );
    assert_eq!(error.to_string(), "mesh index out of bounds");
}

#[test]
fn particle_admission_retains_mass_count_and_invalid_particle_roles() {
    let mut mesh = square();
    let original_position = mesh.positions[2];
    mesh.positions[2].x = f32::NAN;
    mesh.masses.pop();
    assert!(
        matches!(mesh.validate(), Err(ShellMeshError::NonFinitePosition { particle, .. }) if particle == ParticleInputIndex::from(2))
    );
    mesh.positions[2] = original_position;
    assert!(
        matches!(mesh.validate(), Err(ShellMeshError::MassCount { particles, masses }) if particles == ParticleInputCount::from(4) && masses == ParticleMassCount::from(3))
    );
    mesh.masses.push(ParticleMass::ZERO);
    mesh.masses[1] = (-1.0).into();
    mesh.rest_lengths.clear();
    let error = mesh.validate().unwrap_err();
    assert!(
        matches!(error, ShellMeshError::InvalidMass { particle, mass } if particle == ParticleInputIndex::from(1) && mass == ParticleMass::from(-1.0))
    );
    assert_eq!(error.to_string(), "invalid shell masses");
}

#[test]
fn constraint_cardinality_precedes_references_and_retains_its_family() {
    let mut mesh = square();
    let lengths = mesh.rest_lengths.clone();
    let weights = mesh.bend_weights.clone();
    mesh.rest_lengths.pop();
    mesh.bend_weights.clear();
    mesh.triangles[0][0] = 4;
    assert!(
        matches!(mesh.validate(), Err(ShellMeshError::ConstraintDataLength { family: ShellConstraintFamily::Stretch, constraints, records }) if constraints == ConstraintCount::from(5) && records == ConstraintCount::from(4))
    );
    mesh.rest_lengths = lengths;
    assert!(
        matches!(mesh.validate(), Err(ShellMeshError::ConstraintDataLength { family: ShellConstraintFamily::Bending, constraints, records }) if constraints == ConstraintCount::from(1) && records == ConstraintCount::from(0))
    );
    mesh.bend_weights = weights;
    assert!(matches!(
        mesh.validate(),
        Err(ShellMeshError::IndexOutOfBounds {
            family: ShellParticleReference::Triangle,
            ..
        })
    ));
}

#[test]
fn reference_order_keeps_triangle_stretch_seam_and_bend_roles() {
    let mut mesh = square();
    let triangle = mesh.triangles[0];
    let edge = mesh.edges[0];
    let bend = mesh.bends[0];
    mesh.triangles[0][0] = 4;
    mesh.edges[0][0] = 4;
    mesh.seams = vec![[0, 4]];
    mesh.bends[0].wings[0] = 4;
    mesh.rest_lengths[0] = f32::NAN;
    assert!(
        matches!(mesh.validate(), Err(ShellMeshError::IndexOutOfBounds { family: ShellParticleReference::Triangle, index, particles }) if index == ParticleIndex::from(4) && particles == ParticleInputCount::from(4))
    );
    mesh.triangles[0] = triangle;
    assert!(matches!(
        mesh.validate(),
        Err(ShellMeshError::IndexOutOfBounds {
            family: ShellParticleReference::Stretch,
            ..
        })
    ));
    mesh.edges[0] = edge;
    assert!(matches!(
        mesh.validate(),
        Err(ShellMeshError::IndexOutOfBounds {
            family: ShellParticleReference::Seam,
            ..
        })
    ));
    mesh.seams.clear();
    let error = mesh.validate().unwrap_err();
    assert!(
        matches!(error, ShellMeshError::BendIndex { index, .. } if index == ParticleIndex::from(4))
    );
    assert_eq!(error.to_string(), "bend index out of bounds");
    mesh.bends[0] = bend;
    assert!(matches!(
        mesh.validate(),
        Err(ShellMeshError::RestData {
            family: ShellConstraintFamily::Stretch
        })
    ));
}

#[test]
fn rest_data_classification_preserves_stretch_before_bending_and_signed_zero() {
    let mut mesh = square();
    mesh.rest_lengths[0] = -0.0;
    assert!(mesh.validate().is_ok());
    mesh.bend_weights[0] = nonfinite_bend();
    mesh.rest_lengths[0] = -1.0;
    assert!(matches!(
        mesh.validate(),
        Err(ShellMeshError::RestData {
            family: ShellConstraintFamily::Stretch
        })
    ));
    mesh.rest_lengths[0] = -0.0;
    let error = mesh.validate().unwrap_err();
    assert!(matches!(
        error,
        ShellMeshError::RestData {
            family: ShellConstraintFamily::Bending
        }
    ));
    assert_eq!(error.to_string(), "invalid shell rest data");
}
