//! The scratched parametric metal every rigid armor piece is shaded with.
//!
//! Plate armor edits its own [`Metal`]. Catalog plate steel is shaded with the
//! same material, taking its color and roughness from the catalog material and
//! the plate's default scratches, at the same texture density, with the
//! article's own engraving cut into it, and its own trim along its edges.
use crate::item_catalog_schema::EquipmentMaterial;
use fabelgeist_armor::{GeneratedArmor, TrimBand, TrimError};
use fabelgeist_armor::{engraving::Engraving, material::Metal, trim::Trim};

/// Whether a catalog material is plate steel, shaded with the scratched metal.
/// Mail keeps its ring weave; the other materials are not plate metal.
pub fn is_plate_steel(material: EquipmentMaterial) -> bool {
    use EquipmentMaterial::*;
    match material {
        PolishedSteel | RoughSteel | OxidizedSteel => true,
        MailSteel | VegetableTannedLeather | Linen | Wool | QuiltedTextile | Hardwood | Lead => {
            false
        }
    }
}

/// The scratched metal a catalog material is shaded with, when it is plate
/// steel, with `engraving` cut into it.
pub fn metal(material: EquipmentMaterial, engraving: Option<&Engraving>) -> Option<Metal> {
    is_plate_steel(material).then(|| {
        let ([red, green, blue, _], _, roughness) = crate::clothing_material::pbr(material);
        Metal {
            color: [red, green, blue],
            roughness,
            engraving: engraving.cloned(),
            ..Metal::default()
        }
    })
}

/// Rescale a fitted piece's body-surface texture coordinates to the metal's
/// texture density, so scratches are the same size on every armor.
pub fn scale_to_metal_density(armor: &mut GeneratedArmor) {
    let length = |a: &[f32], b: &[f32]| {
        a.iter()
            .zip(b)
            .map(|(a, b)| f64::from(a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    };
    let (mut surface, mut texture) = (0.0, 0.0);
    for &[a, b, c] in armor.indices.as_chunks::<3>().0 {
        for (from, to) in [(a, b), (b, c), (c, a)] {
            let (from, to) = (from as usize, to as usize);
            surface += length(&armor.positions[from], &armor.positions[to]);
            texture += length(&armor.texcoords[from], &armor.texcoords[to]);
        }
    }
    if !(texture > 0.0 && surface.is_finite()) {
        return;
    }
    let scale = (surface / texture) as f32 * Metal::TILES_PER_METRE;
    for uv in &mut armor.texcoords {
        *uv = uv.map(|coordinate| coordinate * scale);
    }
}

/// Cut `trim`'s band along the edges of a fitted piece. Returns the piece and
/// its band's texture coordinates, at the metal's texture density like the
/// rest of the plate, so the trim's ornament runs along each edge with the
/// top of its image on the edge.
pub fn trimmed(
    armor: GeneratedArmor,
    trim: &Trim,
) -> Result<(GeneratedArmor, Vec<[f32; 2]>), TrimError> {
    let armor = armor.trimmed(TrimBand {
        width: trim.width,
        period: trim.period(),
    })?;
    let origin = trim.cell_origin();
    let texcoords = armor
        .trim
        .as_ref()
        .map(|band| {
            band.coordinates
                .iter()
                .map(|metres| metres.map(|m| m * Metal::TILES_PER_METRE + origin))
                .collect()
        })
        .unwrap_or_default();
    Ok((armor, texcoords))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_polished_steel_is_the_plate_armor_default() {
        assert_eq!(
            metal(EquipmentMaterial::PolishedSteel, None),
            Some(Metal::default())
        );
        let rough = metal(EquipmentMaterial::RoughSteel, None).unwrap();
        assert!(rough.roughness > Metal::default().roughness);
        assert_eq!(rough.scratch_density, Metal::default().scratch_density);
        let engraving = Engraving::new("ornament.png");
        let engraved = metal(EquipmentMaterial::RoughSteel, Some(&engraving)).unwrap();
        assert_eq!(engraved.engraving.as_ref(), Some(&engraving));
        for material in [EquipmentMaterial::MailSteel, EquipmentMaterial::Linen] {
            assert!(metal(material, Some(&engraving)).is_none());
        }
    }

    /// One metre square of plate face, mapped onto half the texture.
    fn square() -> GeneratedArmor {
        GeneratedArmor {
            components: Vec::new(),
            design_hash: [0; 32],
            surface_domain: "test".into(),
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            texcoords: vec![[0.0, 0.0], [0.5, 0.0], [0.5, 0.5], [0.0, 0.5]],
            joint_indices: vec![[0; 8]; 4],
            joint_weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            faces: vec![fabelgeist_armor::PlateFace::Outer; 2],
            trim: None,
            grids: Vec::new(),
            morphs: Vec::new(),
        }
    }

    #[test]
    fn texture_coordinates_repeat_at_the_metal_density() {
        let mut armor = square();
        scale_to_metal_density(&mut armor);
        let tiles = Metal::TILES_PER_METRE;
        assert!((armor.texcoords[2][0] - tiles).abs() < 1e-5);
        assert!((armor.texcoords[2][1] - tiles).abs() < 1e-5);
    }

    #[test]
    fn a_trim_ornament_runs_along_the_edge_at_the_metal_density() {
        // The metre square in 5 cm cells, wider than the band.
        let cells = 20u32;
        let row = cells + 1;
        let positions = (0..row * row)
            .map(|v| [(v % row) as f32, (v / row) as f32, 0.0].map(|x| x / cells as f32))
            .collect::<Vec<_>>();
        let count = positions.len();
        let indices = (0..cells * cells)
            .flat_map(|cell| {
                let a = cell / cells * row + cell % cells;
                [a, a + 1, a + row + 1, a, a + row + 1, a + row]
            })
            .collect::<Vec<_>>();
        let grid = GeneratedArmor {
            texcoords: positions.iter().map(|p| [p[0], p[1]]).collect(),
            normals: vec![[0.0, 0.0, 1.0]; count],
            joint_indices: vec![[0; 8]; count],
            joint_weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; count],
            faces: vec![fabelgeist_armor::PlateFace::Outer; indices.len() / 3],
            positions,
            indices,
            ..square()
        };
        let trim = Trim::default();
        let (armor, texcoords) = trimmed(grid, &trim).unwrap();
        assert_eq!(texcoords.len(), armor.positions.len());
        let tiles = Metal::TILES_PER_METRE;
        let band = armor.trim.as_ref().unwrap().bands[0].clone();
        let reach = armor.indices[band]
            .iter()
            .map(|v| texcoords[*v as usize])
            .fold([0.0f32; 2], |most, uv| {
                [most[0].max(uv[0]), most[1].max(uv[1])]
            });
        // Four metres round, with the triangles straddling the rim's start
        // running on past it by at most a cell, and one band width in from
        // the edge.
        let cell = 1.0 / cells as f32;
        assert!(
            reach[0] >= 4.0 * tiles - 1e-3 && reach[0] <= (4.0 + cell) * tiles,
            "{reach:?}"
        );
        assert!((reach[1] - trim.width * tiles).abs() < 1e-5, "{reach:?}");
    }
}
