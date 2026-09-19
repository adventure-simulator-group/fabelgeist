//! The scratched parametric metal every rigid armor piece is shaded with.
//!
//! Plate armor edits its own [`Metal`]. Catalog plate steel is shaded with the
//! same material, taking its color and roughness from the catalog material and
//! the plate's default scratches, at the same texture density.
use crate::item_catalog_schema::EquipmentMaterial;
use adventuresim_armor_model::GeneratedArmor;
use fabelgeist_armor::material::Metal;

/// The scratched metal a catalog material is shaded with, when it is plate
/// steel. Mail keeps its ring weave; the other materials are not plate metal.
pub fn metal(material: EquipmentMaterial) -> Option<Metal> {
    use EquipmentMaterial::*;
    match material {
        PolishedSteel | RoughSteel | OxidizedSteel => {
            let ([red, green, blue, _], _, roughness) = crate::clothing_material::pbr(material);
            Some(Metal {
                color: [red, green, blue],
                roughness,
                ..Metal::default()
            })
        }
        MailSteel | VegetableTannedLeather | Linen | Wool | QuiltedTextile | Hardwood | Lead => {
            None
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_polished_steel_is_the_plate_armor_default() {
        assert_eq!(
            metal(EquipmentMaterial::PolishedSteel),
            Some(Metal::default())
        );
        let rough = metal(EquipmentMaterial::RoughSteel).unwrap();
        assert!(rough.roughness > Metal::default().roughness);
        assert_eq!(rough.scratch_density, Metal::default().scratch_density);
        for material in [EquipmentMaterial::MailSteel, EquipmentMaterial::Linen] {
            assert!(metal(material).is_none());
        }
    }

    #[test]
    fn texture_coordinates_repeat_at_the_metal_density() {
        let mut armor = GeneratedArmor {
            components: Vec::new(),
            design_hash: [0; 32],
            surface_domain: "test".into(),
            // One metre square, mapped onto half the texture.
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
            morphs: Vec::new(),
        };
        scale_to_metal_density(&mut armor);
        let tiles = Metal::TILES_PER_METRE;
        assert!((armor.texcoords[2][0] - tiles).abs() < 1e-5);
        assert!((armor.texcoords[2][1] - tiles).abs() < 1e-5);
    }
}
