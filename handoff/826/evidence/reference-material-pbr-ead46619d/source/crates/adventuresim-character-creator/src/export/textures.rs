//! Surface maps embedded in standalone characters or shared by equipment files.
use super::*;
use std::{collections::BTreeMap, fs, io::ErrorKind};

#[derive(Clone)]
pub struct SurfaceTextures {
    pub base_color_png: Option<Vec<u8>>,
    pub normal_png: Vec<u8>,
    /// Linear ambient visibility, read from the red channel by glTF and Bevy.
    pub occlusion_png: Option<Vec<u8>>,
    pub alpha: SurfaceAlpha,
    pub sides: SurfaceSides,
    pub sampling: SurfaceSampling,
}

pub struct ShellTextures {
    pub normal_png: Vec<u8>,
    pub metal_roughness_png: Vec<u8>,
}

#[derive(Default)]
pub(super) struct TextureImages {
    uses_textile_finish: bool,
    images: Vec<Value>,
    textures: Vec<Value>,
    shared_files: Option<BTreeMap<String, Vec<u8>>>,
    shell_cache: std::collections::HashMap<*const ShellTextures, (usize, usize)>,
}

/// The packaging contract for a GLB and its surface images.
#[derive(Clone, Copy)]
pub enum GlbOutput<'a> {
    /// A portable character file containing every image in its binary buffer.
    Standalone(&'a Path),
    /// Equipment images are content-addressed PNG siblings shared across GLBs.
    SharedTextures(&'a Path),
}

/// Addressing follows the physical chart: a tile repeats both axes; a wrapped
/// garment repeats around its seam and clamps at its hem and proximal edge.
/// An open cloth chart clamps both directions at its free edges.
#[derive(Clone, Copy)]
pub enum SurfaceSampling {
    Tile,
    WrappedGarment,
    OpenGarment,
}

#[derive(Clone, Copy)]
pub enum SurfaceAlpha {
    Opaque,
    Mask,
    Blend,
}

#[derive(Clone, Copy)]
pub enum SurfaceSides {
    Front,
    Both,
}

impl<'a> GlbOutput<'a> {
    pub(super) fn path(self) -> &'a Path {
        match self {
            Self::Standalone(path) | Self::SharedTextures(path) => path,
        }
    }
}

impl SurfaceTextures {
    /// Color bytes already contain color and opacity; the scalar factor is white.
    /// Normal-only maps retain the authored scalar appearance.
    pub(super) fn scalar_appearance(
        &self,
        mut appearance: fabelgeist_armor::SurfaceMaterial,
    ) -> fabelgeist_armor::SurfaceMaterial {
        if self.base_color_png.is_some() {
            appearance.base_color = fabelgeist_color::Srgba::WHITE;
        }
        appearance
    }
}

impl ShellTextures {
    pub async fn armor(metal: &fabelgeist_armor::material::Metal) -> Result<Self> {
        use image::ImageEncoder;
        let textures = crate::metal_gpu()
            .await?
            .textures(metal, fabelgeist_armor::material::MetalTextureSize::PREVIEW)
            .await?;
        let encode = |pixels: &[u8]| -> Result<Vec<u8>> {
            let mut bytes = Vec::new();
            image::codecs::png::PngEncoder::new(&mut bytes).write_image(
                pixels,
                textures.size().into(),
                textures.size().into(),
                image::ExtendedColorType::Rgba8,
            )?;
            Ok(bytes)
        };
        Ok(Self {
            normal_png: encode(textures.normal())?,
            metal_roughness_png: encode(textures.metal_roughness())?,
        })
    }
}

impl TextureImages {
    pub fn new(output: GlbOutput<'_>) -> Self {
        Self {
            shared_files: matches!(output, GlbOutput::SharedTextures(_)).then(BTreeMap::new),
            ..Self::default()
        }
    }

    fn image(
        &mut self,
        bytes: &[u8],
        buffer: &mut BufferBuilder,
        sampling: SurfaceSampling,
    ) -> usize {
        let index = self.images.len();
        let source = if let Some(files) = &mut self.shared_files {
            let filename = format!("texture-{}.png", blake3::hash(bytes).to_hex());
            files.insert(filename.clone(), bytes.to_vec());
            json!({"uri":filename,"mimeType":"image/png"})
        } else {
            json!({"bufferView":buffer.push(bytes, None),"mimeType":"image/png"})
        };
        self.images.push(source);
        let sampler = match sampling {
            SurfaceSampling::Tile => 0,
            SurfaceSampling::WrappedGarment => 1,
            SurfaceSampling::OpenGarment => 2,
        };
        self.textures
            .push(json!({"source":index,"sampler":sampler}));
        index
    }

    pub fn finish(self, output: GlbOutput<'_>, document: &mut Value) -> Result<()> {
        if self.uses_textile_finish {
            document["extensionsUsed"] = json!(["KHR_materials_specular"]);
        }
        if let Some(files) = self.shared_files {
            let directory = output
                .path()
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            fs::create_dir_all(directory)
                .with_context(|| format!("creating texture directory {}", directory.display()))?;
            for (filename, bytes) in files {
                let path = directory.join(filename);
                match fs::read(&path) {
                    Ok(existing) => anyhow::ensure!(
                        existing == bytes,
                        "shared texture {} has unexpected contents",
                        path.display()
                    ),
                    Err(error) if error.kind() == ErrorKind::NotFound => {
                        fs::write(&path, &bytes)
                            .with_context(|| format!("writing shared texture {}", path.display()))?
                    }
                    Err(error) => {
                        return Err(error)
                            .with_context(|| format!("reading shared texture {}", path.display()));
                    }
                }
            }
        }
        if !self.images.is_empty() {
            document["images"] = json!(self.images);
            document["textures"] = json!(self.textures);
            // glTF native sampler constants; linear filtering matches preview.
            document["samplers"] = json!([
                {"wrapS":10497,"wrapT":10497,"magFilter":9729,"minFilter":9729},
                {"wrapS":10497,"wrapT":33071,"magFilter":9729,"minFilter":9729},
                {"wrapS":33071,"wrapT":33071,"magFilter":9729,"minFilter":9729}
            ]);
        }
        Ok(())
    }

    pub fn apply(
        &mut self,
        maps: SurfaceTextures,
        buffer: &mut BufferBuilder,
        material: &mut Value,
    ) {
        material["doubleSided"] = matches!(maps.sides, SurfaceSides::Both).into();
        if let Some(bytes) = maps.base_color_png {
            let color = self.image(&bytes, buffer, maps.sampling);
            material["pbrMetallicRoughness"]["baseColorTexture"] = json!({"index":color});
        }
        let normal = self.image(&maps.normal_png, buffer, maps.sampling);
        material["normalTexture"] = json!({"index":normal});
        if let Some(bytes) = maps.occlusion_png {
            let occlusion = self.image(&bytes, buffer, maps.sampling);
            material["occlusionTexture"] = json!({"index":occlusion,"strength":1.0});
        }
        match maps.alpha {
            SurfaceAlpha::Opaque => {
                material["alphaMode"] = "OPAQUE".into();
            }
            SurfaceAlpha::Mask => {
                material["alphaMode"] = "MASK".into();
                material["alphaCutoff"] = 0.5.into();
            }
            SurfaceAlpha::Blend => {
                material["alphaMode"] = "BLEND".into();
            }
        }
    }

    pub fn apply_textile_finish(
        &mut self,
        finish: fabelgeist_armor::TextileFinish,
        material: &mut Value,
    ) {
        self.uses_textile_finish = true;
        material["extensions"]["KHR_materials_specular"] =
            json!({"specularFactor":finish.specular_factor()});
    }

    pub fn apply_shell_textures(
        &mut self,
        textures: &ShellTextures,
        buffer: &mut BufferBuilder,
        material: &mut Value,
    ) {
        let key = textures as *const ShellTextures;
        let (normal, metal_roughness) = if let Some(&cached) = self.shell_cache.get(&key) {
            cached
        } else {
            let normal = self.image(&textures.normal_png, buffer, SurfaceSampling::Tile);
            let metal_roughness =
                self.image(&textures.metal_roughness_png, buffer, SurfaceSampling::Tile);
            self.shell_cache.insert(key, (normal, metal_roughness));
            (normal, metal_roughness)
        };
        material["normalTexture"] = json!({"index": normal});
        material["pbrMetallicRoughness"]["metallicRoughnessTexture"] =
            json!({"index": metal_roughness});
        material["pbrMetallicRoughness"]["roughnessFactor"] = 1.0.into();
    }
}

#[cfg(test)]
mod tests;
