//! A fitted piece rebuilt in its construction: its solid plate replaced by
//! rows of lamellar lames or scales laid over its surface, and the cord
//! lacing them together.
//!
//! The plates lie on the piece's [`SurfaceGrid`]s, the outer faces its
//! generator laid out as rows along the plate and columns across it. The
//! base fit decides where every plate lies (see [`layout`]); each plate and
//! cord vertex is then a place on a grid and a height off it, so the same
//! vertices are realized on every morph of the piece, skinned as the surface
//! beneath them. The piece's other shells, which no grid describes, are not
//! rebuilt.

mod embedded;
mod lacing;
mod layout;
mod surface;
mod template;
mod tile_map;

use embedded::{Embedded, EmbeddedMesh};
use lacing::{Lacer, Style};
use layout::Layout;
use surface::GridSurface;
use template::TileTemplate;

use crate::{Construction, ConstructionError, GeneratedArmor, Plate, material::Metal};

/// The most plates one piece may take.
pub const MAX_TILES: usize = 20_000;

/// Offsets that scatter consecutive plates across the metal's texture tile:
/// the fractional parts of multiples of the plastic number's reciprocal
/// powers, which fill the square evenly.
const TEXTURE_SCATTER: [f32; 2] = [0.754_877_7, 0.569_840_3];

/// A piece in its construction.
#[derive(Clone, Debug, PartialEq)]
pub struct Constructed {
    /// The plates: the fitted piece itself when it is solid.
    pub plates: GeneratedArmor,
    /// The cord lacing the plates, when they are laced.
    pub lacing: Option<GeneratedArmor>,
}

impl GeneratedArmor {
    /// Rebuild the piece in `construction`. A solid piece is returned as it
    /// is; small plates need the piece's surface grids, so they are built
    /// before the piece is trimmed.
    pub fn constructed(
        self,
        construction: &Construction,
    ) -> Result<Constructed, ConstructionError> {
        construction.validate()?;
        let (tiling, style) = match construction {
            Construction::Solid => {
                return Ok(Constructed {
                    plates: self,
                    lacing: None,
                });
            }
            Construction::Lamellar(tiling) => (tiling, Style::Lamellar),
            Construction::Scale(tiling) => (tiling, Style::Scale),
        };
        let surfaces = self
            .grids
            .iter()
            .filter(|grid| GridSurface::is_spanning(grid))
            .map(|grid| GridSurface::base(grid, &self.positions, &self.normals))
            .collect::<Option<Vec<_>>>()
            .ok_or(ConstructionError::DegenerateSurface)?;
        if surfaces.is_empty() {
            return Err(ConstructionError::NoSurfaceGrid);
        }
        let plate = &tiling.plate;
        let radius = tiling.lacing.as_ref().map(|lacing| lacing.radius);
        let layout = Layout::new(&surfaces, plate, radius, MAX_TILES)?;
        let hash = design_hash(self.design_hash, construction);
        let plates = tiles(&layout, plate)?.armor(&self, &surfaces, hash);
        let lacing = tiling
            .lacing
            .as_ref()
            .map(|lacing| {
                let lacer = Lacer {
                    layout: &layout,
                    surfaces: &surfaces,
                    plate,
                    lacing,
                };
                Ok::<_, ConstructionError>(lacer.mesh(style)?.armor(&self, &surfaces, hash))
            })
            .transpose()?;
        Ok(Constructed { plates, lacing })
    }
}

/// Every plate of the layout, each a copy of the template bent onto the
/// surface and tilted by its row's slope.
fn tiles(layout: &Layout, plate: &Plate) -> Result<EmbeddedMesh, ConstructionError> {
    let template = TileTemplate::new(plate).ok_or(ConstructionError::HoleSpacing)?;
    let mut mesh = EmbeddedMesh::default();
    for (index, tile) in layout.tiles.iter().enumerate() {
        let first = mesh.vertices.len() as u32;
        let slope = layout.slopes[tile.grid];
        let scatter = TEXTURE_SCATTER.map(|step| (index as f32 * step).fract());
        for vertex in &template.vertices {
            let [x, y, z] = vertex.position;
            // The tilt shears the plate along its height; its normals follow
            // the shear's inverse transpose.
            let [nx, ny, nz] = vertex.normal;
            mesh.vertices.push(Embedded {
                grid: tile.grid,
                at: tile.map.at(x, y),
                height: z + layout.rise(tile.grid, plate, y),
                normal: [nx, ny + slope * nz, nz],
                texcoord: std::array::from_fn(|k| {
                    vertex.texcoord[k] * Metal::TILES_PER_METRE + scatter[k]
                }),
            });
        }
        for triangle in &template.triangles {
            mesh.indices.extend(triangle.map(|corner| first + corner));
        }
        mesh.faces.extend(&template.faces);
    }
    Ok(mesh)
}

/// The provenance of a piece rebuilt in `construction`.
fn design_hash(piece: [u8; 32], construction: &Construction) -> [u8; 32] {
    let mut bytes = piece.to_vec();
    // A construction always serializes: it holds only numbers and variants.
    bytes.extend(serde_json::to_vec(construction).unwrap_or_default());
    crate::parametric_design_hash(&bytes)
}

#[cfg(test)]
mod tests;
