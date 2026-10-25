use crate::item_catalog_schema::EquipmentMaterial;
use fabelgeist_armor::{SurfaceMaterial, material::Metal};
use fabelgeist_color::Srgba;
use fabelgeist_math::units::Ratio;

pub fn pbr(material: EquipmentMaterial) -> SurfaceMaterial {
    match material {
        EquipmentMaterial::PolishedSteel => Metal::default().appearance(Ratio::from_ratio(1.0)),
        EquipmentMaterial::RoughSteel => SurfaceMaterial {
            base_color: Srgba::from_encoded_channels([0.769, 0.776, 0.776, 1.0]),
            metallic: Ratio::from_ratio(1.0),
            roughness: Ratio::from_ratio(0.58),
        },
        EquipmentMaterial::OxidizedSteel => SurfaceMaterial {
            base_color: Srgba::from_encoded_channels([0.420, 0.275, 0.196, 1.0]),
            metallic: Ratio::from_ratio(0.0),
            roughness: Ratio::from_ratio(0.82),
        },
        EquipmentMaterial::MailSteel => SurfaceMaterial {
            base_color: Srgba::from_encoded_channels([0.769, 0.776, 0.776, 1.0]),
            metallic: Ratio::from_ratio(1.0),
            roughness: Ratio::from_ratio(0.42),
        },
        EquipmentMaterial::VegetableTannedLeather => SurfaceMaterial {
            base_color: Srgba::from_encoded_channels([0.502, 0.353, 0.231, 1.0]),
            metallic: Ratio::from_ratio(0.0),
            roughness: Ratio::from_ratio(0.58),
        },
        EquipmentMaterial::Linen => SurfaceMaterial {
            base_color: Srgba::from_encoded_channels([0.722, 0.663, 0.510, 1.0]),
            metallic: Ratio::from_ratio(0.0),
            roughness: Ratio::from_ratio(0.88),
        },
        EquipmentMaterial::Wool => SurfaceMaterial {
            base_color: Srgba::from_encoded_channels([0.561, 0.510, 0.408, 1.0]),
            metallic: Ratio::from_ratio(0.0),
            roughness: Ratio::from_ratio(0.92),
        },
        EquipmentMaterial::QuiltedTextile => SurfaceMaterial {
            base_color: Srgba::from_encoded_channels([0.459, 0.416, 0.314, 1.0]),
            metallic: Ratio::from_ratio(0.0),
            roughness: Ratio::from_ratio(0.90),
        },
        EquipmentMaterial::Hardwood => SurfaceMaterial {
            base_color: Srgba::from_encoded_channels([0.235, 0.118, 0.047, 1.0]),
            metallic: Ratio::from_ratio(0.0),
            roughness: Ratio::from_ratio(0.72),
        },
        EquipmentMaterial::Lead => SurfaceMaterial {
            base_color: Srgba::from_encoded_channels([0.310, 0.322, 0.337, 1.0]),
            metallic: Ratio::from_ratio(1.0),
            roughness: Ratio::from_ratio(0.68),
        },
    }
}
