use crate::{
    ArealDensityValidity, MassValidity, ParticleArealDensity, ParticleInverseMass, ParticleMass,
};
use crate::{BendQuad, SelfCollision, topology};
use anyhow::{Result, ensure};
use fabelgeist_math::Vec3;

/// Rest geometry and constraints, independent of any garment or material model.
#[derive(Clone, Debug, Default)]
pub struct ShellMesh {
    pub positions: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
    pub edges: Vec<[u32; 2]>,
    pub rest_lengths: Vec<f32>,
    pub bends: Vec<BendQuad>,
    pub bend_weights: Vec<[f32; 8]>,
    pub seams: Vec<[u32; 2]>,
    pub masses: Vec<ParticleMass>,
}
impl ShellMesh {
    pub fn from_mesh(
        mesh: &fabelgeist_mesh::MeshData,
        areal_density: ParticleArealDensity,
    ) -> Result<Self> {
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
    ) -> Result<Self> {
        ensure!(
            areal_density.validity() == ArealDensityValidity::PositiveFinite,
            "areal density must be positive"
        );
        ensure!(
            positions.iter().all(|p| p.is_finite()),
            "non-finite shell position"
        );
        ensure!(
            triangles
                .iter()
                .flatten()
                .all(|i| (*i as usize) < positions.len()),
            "shell triangle index out of bounds"
        );
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
            if let Some(k) = topology::bending_weights(points) {
                let rest = points
                    .iter()
                    .zip(k)
                    .fold(Vec3::default(), |s, (p, w)| s + *p * w)
                    .length();
                bends.push(bend);
                bend_weights.push([k[0], k[1], k[2], k[3], rest, 0.0, 0.0, 0.0]);
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
    pub fn validate(&self) -> Result<()> {
        let n = self.positions.len();
        ensure!(
            n > 0 && n <= u32::MAX as usize,
            "invalid shell particle count"
        );
        ensure!(
            self.positions.iter().all(|p| p.is_finite()),
            "non-finite shell position"
        );
        ensure!(
            self.masses.len() == n
                && self
                    .masses
                    .iter()
                    .all(|m| m.validity() == MassValidity::FiniteNonnegative),
            "invalid shell masses"
        );
        ensure!(
            self.edges.len() == self.rest_lengths.len()
                && self.bends.len() == self.bend_weights.len(),
            "constraint data length mismatch"
        );
        ensure!(
            self.triangles
                .iter()
                .flatten()
                .chain(self.edges.iter().flatten())
                .chain(self.seams.iter().flatten())
                .all(|i| (*i as usize) < n),
            "shell index out of bounds"
        );
        ensure!(
            self.bends
                .iter()
                .flat_map(|b| b.particles())
                .all(|i| (i as usize) < n),
            "bend index out of bounds"
        );
        ensure!(
            self.rest_lengths.iter().all(|v| v.is_finite() && *v >= 0.0)
                && self.bend_weights.iter().flatten().all(|v| v.is_finite()),
            "invalid shell rest data"
        );
        Ok(())
    }
    pub fn inverse_masses(&self) -> Vec<ParticleInverseMass> {
        topology::inverse_masses(&self.masses)
    }
    pub fn adjacency(&self) -> Vec<Vec<u32>> {
        let mut edges = self.edges.clone();
        edges.extend_from_slice(&self.seams);
        SelfCollision::adjacency(self.positions.len().into(), &edges)
    }
}
