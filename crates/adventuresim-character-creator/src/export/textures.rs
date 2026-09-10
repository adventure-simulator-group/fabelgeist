//! Surface maps embedded in standalone characters or shared by equipment files.
use super::*;
use std::{collections::BTreeMap, fs, io::ErrorKind};

#[cfg(test)]
mod tests;

/// The packaging contract for a GLB and its surface images.
#[derive(Clone, Copy)]
pub enum GlbOutput<'a> {
    /// A portable character file containing every image in its binary buffer.
    Standalone(&'a Path),
    /// Equipment images are content-addressed PNG siblings shared across GLBs.
    SharedTextures(&'a Path),
}

impl<'a> GlbOutput<'a> {
    pub(super) fn path(self) -> &'a Path {
        match self {
            Self::Standalone(path) | Self::SharedTextures(path) => path,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SurfaceTextures {
    pub base_color_png: &'static [u8],
    pub normal_png: &'static [u8],
    /// Linear ambient visibility, read from the red channel by glTF and Bevy.
    pub occlusion_png: Option<&'static [u8]>,
    pub cutout: bool,
}

pub struct ShellTextures {
    pub normal_png: Vec<u8>,
    pub metal_roughness_png: Vec<u8>,
}

impl ShellTextures {
    pub fn armor(metal: &fabelgeist_armor::material::Metal) -> Result<Self> {
        use image::ImageEncoder;
        let textures = metal.textures(512).map_err(anyhow::Error::msg)?;
        let encode = |pixels: &[u8]| -> Result<Vec<u8>> {
            let mut bytes = Vec::new();
            image::codecs::png::PngEncoder::new(&mut bytes).write_image(
                pixels,
                textures.size,
                textures.size,
                image::ExtendedColorType::Rgba8,
            )?;
            Ok(bytes)
        };
        Ok(Self {
            normal_png: encode(&textures.normal)?,
            metal_roughness_png: encode(&textures.metal_roughness)?,
        })
    }
}

#[derive(Default)]
pub(super) struct TextureImages {
    images: Vec<Value>,
    textures: Vec<Value>,
    shared_files: Option<BTreeMap<String, &'static [u8]>>,
    shell_cache: std::collections::HashMap<*const ShellTextures, (usize, usize)>,
}

impl TextureImages {
    pub fn new(output: GlbOutput<'_>) -> Self {
        Self {
            shared_files: matches!(output, GlbOutput::SharedTextures(_)).then(BTreeMap::new),
            ..Self::default()
        }
    }

    fn image(&mut self, bytes: &'static [u8], buffer: &mut BufferBuilder) -> usize {
        let index = self.images.len();
        let source = if let Some(files) = &mut self.shared_files {
            let filename = format!("texture-{}.png", blake3::hash(bytes).to_hex());
            files.insert(filename.clone(), bytes);
            json!({"uri":filename,"mimeType":"image/png"})
        } else {
            json!({"bufferView":buffer.push(bytes, None),"mimeType":"image/png"})
        };
        self.images.push(source);
        self.textures.push(json!({"source":index}));
        index
    }

    pub fn finish(self, output: GlbOutput<'_>, document: &mut Value) -> Result<()> {
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
                    Err(error) if error.kind() == ErrorKind::NotFound => fs::write(&path, bytes)
                        .with_context(|| format!("writing shared texture {}", path.display()))?,
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
        }
        Ok(())
    }

    pub fn apply(
        &mut self,
        maps: SurfaceTextures,
        buffer: &mut BufferBuilder,
        material: &mut Value,
    ) {
        let color = self.image(maps.base_color_png, buffer);
        let normal = self.image(maps.normal_png, buffer);
        material["pbrMetallicRoughness"]["baseColorFactor"] = json!([1.0, 1.0, 1.0, 1.0]);
        material["pbrMetallicRoughness"]["baseColorTexture"] = json!({"index":color});
        material["normalTexture"] = json!({"index":normal});
        if let Some(bytes) = maps.occlusion_png {
            let occlusion = self.image(bytes, buffer);
            material["occlusionTexture"] = json!({"index":occlusion,"strength":1.0});
        }
        if maps.cutout {
            material["alphaMode"] = "MASK".into();
            material["alphaCutoff"] = 0.5.into();
        }
    }

    fn image_bytes(&mut self, bytes: &[u8], buffer: &mut BufferBuilder) -> usize {
        let index = self.images.len();
        let source = json!({"bufferView": buffer.push(bytes, None), "mimeType": "image/png"});
        self.images.push(source);
        self.textures.push(json!({"source": index}));
        index
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
            let normal = self.image_bytes(&textures.normal_png, buffer);
            let metal_roughness = self.image_bytes(&textures.metal_roughness_png, buffer);
            self.shell_cache.insert(key, (normal, metal_roughness));
            (normal, metal_roughness)
        };
        material["normalTexture"] = json!({"index": normal});
        material["pbrMetallicRoughness"]["metallicRoughnessTexture"] =
            json!({"index": metal_roughness});
        material["pbrMetallicRoughness"]["roughnessFactor"] = 1.0.into();
    }
}
