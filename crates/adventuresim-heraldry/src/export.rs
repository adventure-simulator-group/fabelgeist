//! Portable bundles use one validated recipe and the same material/mesh contract.
mod archive;
mod glb;
mod paint;
use crate::{
    Error,
    artwork::Artwork,
    bake::{Baked, Resolution, TextureKind, mips},
    document::Document,
};
pub use archive::zip;
pub use glb::glb;
use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
pub struct ExportFile {
    pub name: String,
    pub bytes: Vec<u8>,
}
pub fn png(rgba: &[u8], size: u32) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes).write_image(rgba, size, size, ExtendedColorType::Rgba8)?;
    Ok(bytes)
}
pub fn bundle(d: &Document, b: &Baked, r: Resolution) -> Result<Vec<ExportFile>, Error> {
    d.validate()?;
    if !b.matches(d, r) {
        return Err(Error::Invalid(
            "the bake does not match this document and resolution".into(),
        ));
    }
    let mut files = vec![
        ExportFile {
            name: "arms.json".into(),
            bytes: d.to_json()?.into_bytes(),
        },
        ExportFile {
            name: "arms.svg".into(),
            bytes: Artwork::compose(d)?.svg(&d.surface.palette).into_bytes(),
        },
        ExportFile {
            name: "flat.png".into(),
            bytes: png(&b.flat, b.size)?,
        },
        ExportFile {
            name: "paint-recipes.json".into(),
            bytes: serde_json::to_vec_pretty(&paint::manifest(&d.surface.palette))?,
        },
    ];
    if d.surface
        .palette
        .0
        .iter()
        .any(|p| matches!(p, crate::paint::Paint::Mixed { .. }))
    {
        files.push(ExportFile {
            name: "paint-calibration.json".into(),
            bytes: crate::paint::mixing::CALIBRATION_JSON.as_bytes().to_vec(),
        });
    }
    for (name, bytes, kind) in [
        ("base-color", &b.albedo, TextureKind::Color),
        ("normal", &b.normal, TextureKind::Normal),
        ("orm", &b.orm, TextureKind::Linear),
        ("coat", &b.coat, TextureKind::Linear),
    ] {
        for (i, mip) in mips(bytes, b.size, kind).into_iter().enumerate() {
            files.push(ExportFile {
                name: if i == 0 {
                    format!("{name}.png")
                } else {
                    format!("mips/{name}-{i}.png")
                },
                bytes: png(&mip.rgba, mip.size)?,
            });
        }
    }
    let (height, low, high) = height_png(b)?;
    files.push(ExportFile {
        name: "height.png".into(),
        bytes: height,
    });
    files.push(ExportFile {
        name: "display.glb".into(),
        bytes: glb(d, b)?,
    });
    let metadata = serde_json::json!({
        "bake":b.stamp,"size":b.size,"lengthUnit":"metre",
        "height":{"unit":"millimetre","zero":low,"scale":high-low,"encoding":"unorm16"},
        "normal":"tangent +Y up; linear RGB",
        "orm":"linear R=1 occlusion G=perceptual roughness B=metallic",
        "coat":"linear R=clearcoat coverage G=coat roughness B=0; both glTF factors are 1",
        "baseColor":"sRGB, straight alpha; dielectric pigment or conductor reflectance; no baked lighting",
        "gilding":"Application profiles follow conservation evidence; numeric responses are artist approximations. Yellow-glazed silver uses RGB absorption and a neutral clearcoat, not a spectral coating model.",
        "paintRecipes":"paint-recipes.json records sourced catalog estimates or bounded measured-stock recipes, with their distinct evidence limits; no arbitrary pigment optics",
        "support":"display object only; wood backing; no fittings",
        "mips":"color filtered in linear space with alpha weighting; normals renormalized"
    });
    files.push(ExportFile {
        name: "material.json".into(),
        bytes: serde_json::to_vec_pretty(&metadata)?,
    });
    if let Some(credit) = crate::provenance::attribution(d) {
        files.push(ExportFile {
            name: "ATTRIBUTION.txt".into(),
            bytes: credit.as_bytes().to_vec(),
        });
    }
    Ok(files)
}
fn height_png(b: &Baked) -> Result<(Vec<u8>, f32, f32), Error> {
    let low = b.height.iter().copied().fold(f32::INFINITY, f32::min);
    let high = b.height.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let scale = (high - low).max(f32::EPSILON);
    let raw: Vec<u8> = b
        .height
        .iter()
        .flat_map(|v| (((v - low) / scale * f32::from(u16::MAX)).round() as u16).to_ne_bytes())
        .collect();
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes).write_image(&raw, b.size, b.size, ExtendedColorType::L16)?;
    Ok((bytes, low, high))
}
