//! glTF 2.0 binary adapter. Embedded PNG maps use the editor's material channels.
use crate::{Error, bake::Baked, document::Document, geometry::Mesh};
use serde_json::{Value, json};
const GLB_MAGIC: u32 = 0x4654_6c67;
const JSON_CHUNK: u32 = 0x4e4f_534a;
const BINARY_CHUNK: u32 = 0x004e_4942;
struct Buffer {
    bytes: Vec<u8>,
    views: Vec<Value>,
    accessors: Vec<Value>,
}
impl Buffer {
    fn view(&mut self, bytes: &[u8]) -> usize {
        while !self.bytes.len().is_multiple_of(4) {
            self.bytes.push(0);
        }
        let index = self.views.len();
        self.views
            .push(json!({"buffer":0,"byteOffset":self.bytes.len(),"byteLength":bytes.len()}));
        self.bytes.extend(bytes);
        index
    }
    fn floats<const N: usize>(&mut self, values: &[[f32; N]], kind: &str, bounds: bool) -> usize {
        let bytes: Vec<_> = values
            .iter()
            .flatten()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let view = self.view(&bytes);
        let mut accessor =
            json!({"bufferView":view,"componentType":5126,"count":values.len(),"type":kind});
        if bounds {
            accessor["min"] = json!(
                (0..N)
                    .map(|c| values.iter().map(|p| p[c]).fold(f32::INFINITY, f32::min))
                    .collect::<Vec<_>>()
            );
            accessor["max"] = json!(
                (0..N)
                    .map(|c| values
                        .iter()
                        .map(|p| p[c])
                        .fold(f32::NEG_INFINITY, f32::max))
                    .collect::<Vec<_>>()
            );
        }
        let index = self.accessors.len();
        self.accessors.push(accessor);
        index
    }
    fn indices(&mut self, indices: &[u32]) -> usize {
        let view = self.view(
            &indices
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>(),
        );
        let index = self.accessors.len();
        self.accessors.push(
            json!({"bufferView":view,"componentType":5125,"count":indices.len(),"type":"SCALAR"}),
        );
        index
    }
}
pub fn glb(d: &Document, b: &Baked) -> Result<Vec<u8>, Error> {
    d.validate()?;
    if !crate::bake::Resolution::ALL
        .into_iter()
        .any(|r| b.matches(d, r))
    {
        return Err(Error::Invalid("GLB requires a matching bake".into()));
    }
    let mesh = Mesh::generate(d)?;
    let mut buffer = Buffer {
        bytes: vec![],
        views: vec![],
        accessors: vec![],
    };
    let attributes = json!({"POSITION":buffer.floats(&mesh.positions,"VEC3",true),"NORMAL":buffer.floats(&mesh.normals,"VEC3",false),"TEXCOORD_0":buffer.floats(&mesh.uv,"VEC2",false),"TANGENT":buffer.floats(&mesh.tangents,"VEC4",false)});
    let front = buffer.indices(&mesh.front);
    let support = buffer.indices(&mesh.support);
    let mut images = Vec::new();
    for pixels in [&b.albedo, &b.normal, &b.orm, &b.coat] {
        let bytes = super::png(pixels, b.size)?;
        images.push(json!({"bufferView":buffer.view(&bytes),"mimeType":"image/png"}));
    }
    let document = json!({
        "asset":{"version":"2.0","generator":"Fabelgeist Heraldry","copyright":crate::provenance::attribution(d).unwrap_or("")},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"name":d.name,"mesh":0}],
        "extensionsUsed":["KHR_materials_clearcoat"],
        "extras":{"paintRecipes":super::paint::manifest(&d.surface.palette)},
        "meshes":[{"primitives":[{"attributes":attributes,"indices":front,"material":0},{"attributes":attributes,"indices":support,"material":1}]}],
        "materials":[{"name":"Intact painted face","pbrMetallicRoughness":{"baseColorTexture":{"index":0},"metallicRoughnessTexture":{"index":2},"metallicFactor":1.0,"roughnessFactor":1.0},"normalTexture":{"index":1},"occlusionTexture":{"index":2},"extensions":{"KHR_materials_clearcoat":{"clearcoatFactor":1.0,"clearcoatRoughnessFactor":1.0,"clearcoatTexture":{"index":3},"clearcoatRoughnessTexture":{"index":3}}}},
            {"name":"Wood support","pbrMetallicRoughness":{"baseColorFactor":[0.16,0.08,0.035,1.0],"metallicFactor":0.0,"roughnessFactor":0.8}}],
        "textures":[{"source":0,"sampler":0},{"source":1,"sampler":0},{"source":2,"sampler":0},{"source":3,"sampler":0}],
        "samplers":[{"magFilter":9729,"minFilter":9987,"wrapS":33071,"wrapT":33071}],
        "images":images,"accessors":buffer.accessors,"bufferViews":buffer.views,"buffers":[{"byteLength":buffer.bytes.len()}]
    });
    let mut json = serde_json::to_vec(&document)?;
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    while !buffer.bytes.len().is_multiple_of(4) {
        buffer.bytes.push(0);
    }
    let length = 12 + 8 + json.len() + 8 + buffer.bytes.len();
    let mut out = Vec::with_capacity(length);
    for v in [GLB_MAGIC, 2, length as u32, json.len() as u32, JSON_CHUNK] {
        out.extend(v.to_le_bytes());
    }
    out.extend(json);
    out.extend((buffer.bytes.len() as u32).to_le_bytes());
    out.extend(BINARY_CHUNK.to_le_bytes());
    out.extend(buffer.bytes);
    Ok(out)
}
