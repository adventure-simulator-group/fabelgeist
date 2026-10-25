//! Checked conversion from a fitted surface chart to the metal's tile density.

use super::Metal;
use crate::GeneratedArmor;
use fabelgeist_math::units::Ratio;
use std::num::TryFromIntError;

pub type MetalDensityResult<T> = Result<T, MetalDensityError>;

/// Borrowed columns whose finite values, triangle rows and indices agree.
struct DensityInput<'a> {
    positions: &'a [[f32; 3]],
    texcoords: &'a [[f32; 2]],
    triangles: &'a [[u32; 3]],
}

#[derive(Debug, thiserror::Error)]
pub enum MetalDensityError {
    #[error("metal texture density needs complete triangle triples, received {indices} indices")]
    TriangleIndices { indices: usize },
    #[error("metal texture density has {vertices} positions but {texcoords} texture coordinates")]
    TexcoordCount { vertices: usize, texcoords: usize },
    #[error("metal texture density requires finite positions and texture coordinates")]
    NonFiniteInput,
    #[error("metal texture density index exceeds host addresses")]
    VertexAddress(#[source] TryFromIntError),
    #[error("metal texture density index {index} is outside {vertices} vertices")]
    VertexIndex { index: u32, vertices: usize },
    #[error("metal texture density edge lengths exceed finite arithmetic")]
    EdgeLengths,
    #[error("metal texture density scale exceeds finite native arithmetic")]
    Scale,
    #[error("metal texture density replacement coordinates exceed finite native arithmetic")]
    Coordinates,
}

impl GeneratedArmor {
    /// Rescale model-space metre geometry's chart to the metal's tile density.
    ///
    /// Empty triangles and a constant finite chart leave coordinates unchanged.
    /// A zero-length surface with positive accumulated chart-edge length
    /// retains zero scaling.
    /// Malformed or unrepresentable input returns an error before any coordinate
    /// changes. This admits only the geometry and chart used by this operation;
    /// it does not validate normals, skin, components or morphs.
    pub fn scale_to_metal_density(&mut self) -> MetalDensityResult<()> {
        let Some(scale) = DensityInput::new(self)?.scale()? else {
            return Ok(());
        };
        // The immutable admission checked every product. Exclusive access keeps
        // those exact columns unchanged until this numerical replacement pass.
        for uv in &mut self.texcoords {
            *uv = uv.map(|coordinate| coordinate * scale.ratio());
        }
        Ok(())
    }
}

impl<'a> DensityInput<'a> {
    fn new(armor: &'a GeneratedArmor) -> MetalDensityResult<Self> {
        let (triangles, remainder) = armor.indices.as_chunks::<3>();
        if !remainder.is_empty() {
            return Err(MetalDensityError::TriangleIndices { indices: armor.indices.len() });
        }
        if armor.positions.len() != armor.texcoords.len() {
            return Err(MetalDensityError::TexcoordCount {
                vertices: armor.positions.len(),
                texcoords: armor.texcoords.len(),
            });
        }
        if armor.positions.iter().flatten().chain(armor.texcoords.iter().flatten())
            .any(|value| !value.is_finite())
        {
            return Err(MetalDensityError::NonFiniteInput);
        }
        for &index in &armor.indices {
            let address = usize::try_from(index).map_err(MetalDensityError::VertexAddress)?;
            if address >= armor.positions.len() {
                return Err(MetalDensityError::VertexIndex {
                    index,
                    vertices: armor.positions.len(),
                });
            }
        }
        Ok(Self { positions: &armor.positions, texcoords: &armor.texcoords, triangles })
    }

    fn scale(&self) -> MetalDensityResult<Option<Ratio>> {
        let (mut surface, mut texture) = (0.0, 0.0);
        for &[a, b, c] in self.triangles {
            for (from, to) in [(a, b), (b, c), (c, a)] {
                // Host index membership and representability were admitted above.
                let (from, to) = (from as usize, to as usize);
                surface += edge_length(&self.positions[from], &self.positions[to]);
                texture += edge_length(&self.texcoords[from], &self.texcoords[to]);
            }
        }
        if !surface.is_finite() || !texture.is_finite() {
            return Err(MetalDensityError::EdgeLengths);
        }
        if texture == 0.0 {
            return Ok(None);
        }
        // Preserve the original native order: f64 ratio, f32 narrowing, then
        // multiplication by the number of metal tiles per metre.
        let scale = (surface / texture) as f32 * Metal::TILES_PER_METRE;
        if !scale.is_finite() {
            return Err(MetalDensityError::Scale);
        }
        if self.texcoords.iter().flatten().any(|coordinate| !(coordinate * scale).is_finite()) {
            return Err(MetalDensityError::Coordinates);
        }
        Ok(Some(Ratio::from_ratio(scale)))
    }
}

/// Model-space metre or dimensionless UV edges at the numerical kernel.
/// Subtract in f32 before widening, preserving the original sum and root order.
fn edge_length<const N: usize>(from: &[f32; N], to: &[f32; N]) -> f64 {
    from.iter().zip(to).map(|(from, to)| f64::from(from - to).powi(2))
        .sum::<f64>().sqrt()
}
