//! Canonical body-atlas material selection follows construction, never equipment visibility.
use crate::{armor_recipes::ParametricDesign, export::SurfaceTextures};

pub fn textures(design: Option<&ParametricDesign>) -> Option<SurfaceTextures> {
    match design {
        Some(ParametricDesign::Underlayer(design)) if design.kind.is_mail() => {
            Some(SurfaceTextures {
                base_color_png: Some(
                    include_bytes!("../../../assets_src/equipment/materials/mail-base-color.png")
                        .to_vec(),
                ),
                normal_png: include_bytes!(
                    "../../../assets_src/equipment/materials/mail-normal.png"
                )
                .to_vec(),
                occlusion_png: Some(
                    include_bytes!("../../../assets_src/equipment/materials/mail-occlusion.png")
                        .to_vec(),
                ),
                cutout: true,
            })
        }
        _ => None,
    }
}
