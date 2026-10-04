use crate::{ArealDensityValidity, ParticleArealDensity, ParticleInverseMass, ParticleMass};
use crate::{BendQuad, SelfCollision, topology};
use fabelgeist_math::Vec3;

mod error;
mod validation;
pub use error::{ShellConstraintFamily, ShellMeshError, ShellParticleReference};

/// Rest geometry and constraints, independent of any garment or material model.
#[derive(Clone, Debug, Default)]
pub struct ShellMesh {
    pub positions: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
    pub edges: Vec<[u32; 2]>,
    pub rest_lengths: Vec<f32>,
    pub bends: Vec<BendQuad>,
    pub bend_weights: Vec<crate::BendRecord>,
    pub seams: Vec<[u32; 2]>,
    pub masses: Vec<ParticleMass>,
}
impl ShellMesh {
    pub fn from_mesh(
        mesh: &fabelgeist_mesh::MeshData,
        areal_density: ParticleArealDensity,
    ) -> Result<Self, ShellMeshError> {
        let triangles = mesh.triangles()?;
        let positions = mesh
            .positions
            .iter()
            .map(|p| Vec3::new(p[0], p[1], p[2]))
            .collect();
        Self::new(positions, triangles, areal_density)
    }
    pub fn new(
        positions: Vec<Vec3>,
        triangles: Vec<[u32; 3]>,
        areal_density: ParticleArealDensity,
    ) -> Result<Self, ShellMeshError> {
        if areal_density.validity() != ArealDensityValidity::PositiveFinite {
            return Err(ShellMeshError::ArealDensity {
                density: areal_density,
            });
        }
        validation::validate_positions(&positions)?;
        for &index in triangles.iter().flatten() {
            if (index as usize) >= positions.len() {
                return Err(ShellMeshError::TriangleIndex {
                    index: index.into(),
                    particles: positions.len().into(),
                });
            }
        }
        let topology = topology::build(&triangles);
        let rest_lengths = topology
            .edges
            .iter()
            .map(|[a, b]| (positions[*a as usize] - positions[*b as usize]).length())
            .collect();
        let mut bends = Vec::new();
        let mut bend_weights = Vec::new();
        for bend in topology.bends {
            let points = bend.particles().map(|i| positions[i as usize]);
            let points = crate::BendPoints::from(points);
            if let Ok(weights) = crate::BendWeights::for_points(points) {
                bends.push(bend);
                bend_weights.push(weights.observed_rest(points));
            }
        }
        let masses = topology::vertex_masses(&positions, &triangles, areal_density);
        Ok(Self {
            positions,
            triangles,
            edges: topology.edges,
            rest_lengths,
            bends,
            bend_weights,
            seams: vec![],
            masses,
        })
    }
    pub fn inverse_masses(&self) -> Vec<ParticleInverseMass> {
        topology::inverse_masses(&self.masses)
    }
    pub fn adjacency(&self) -> Vec<Vec<u32>> {
        let mut edges = self.edges.clone();
        edges.extend_from_slice(&self.seams);
        SelfCollision::adjacency(
            fabelgeist_xpbd::ParticleInputCount::from(self.positions.len()),
            &edges,
        )
    }
}

#[cfg(test)]
mod tests;
