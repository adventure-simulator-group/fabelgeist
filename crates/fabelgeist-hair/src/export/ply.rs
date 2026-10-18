//! Enough PLY to read a mesh: ASCII or binary little-endian, vertex
//! properties of any scalar type (read as `f32`), faces as lists -- and to
//! write one as DiffLocks' `scalp.ply` is laid out (binary, `x y z nx ny nz
//! s t`, triangles).

use crate::export::error::{HairDataError, HairDataFormat, Result};

pub struct Ply {
    pub vertex_properties: Vec<String>,
    pub vertices: Vec<Vec<f32>>,
    pub faces: Vec<Vec<u32>>,
}

#[derive(Clone, Copy)]
enum Scalar {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
    F64,
}

impl Scalar {
    fn parse(name: &str) -> Result<Self> {
        Ok(match name {
            "char" | "int8" => Scalar::I8,
            "uchar" | "uint8" => Scalar::U8,
            "short" | "int16" => Scalar::I16,
            "ushort" | "uint16" => Scalar::U16,
            "int" | "int32" => Scalar::I32,
            "uint" | "uint32" => Scalar::U32,
            "float" | "float32" => Scalar::F32,
            "double" | "float64" => Scalar::F64,
            other => return Err(HairDataError::PlyScalar { name: other.into() }),
        })
    }

    fn size(self) -> usize {
        match self {
            Scalar::I8 | Scalar::U8 => 1,
            Scalar::I16 | Scalar::U16 => 2,
            Scalar::I32 | Scalar::U32 | Scalar::F32 => 4,
            Scalar::F64 => 8,
        }
    }

    fn read(self, b: &[u8]) -> f64 {
        match self {
            Scalar::I8 => b[0] as i8 as f64,
            Scalar::U8 => b[0] as f64,
            Scalar::I16 => i16::from_le_bytes([b[0], b[1]]) as f64,
            Scalar::U16 => u16::from_le_bytes([b[0], b[1]]) as f64,
            Scalar::I32 => i32::from_le_bytes(b[..4].try_into().unwrap()) as f64,
            Scalar::U32 => u32::from_le_bytes(b[..4].try_into().unwrap()) as f64,
            Scalar::F32 => f32::from_le_bytes(b[..4].try_into().unwrap()) as f64,
            Scalar::F64 => f64::from_le_bytes(b[..8].try_into().unwrap()),
        }
    }
}

enum Property {
    Scalar(String, Scalar),
    List(String, Scalar, Scalar),
}

struct Element {
    name: String,
    count: usize,
    properties: Vec<Property>,
}

/// A surface as binary little-endian PLY: positions, normals and UVs (as
/// `s t`), and its triangles.
pub fn write_surface(surface: &crate::surface::Surface) -> Vec<u8> {
    let mut out = format!(
        "ply\nformat binary_little_endian 1.0\ncomment fabelgeist-hair\nelement vertex {}\n\
         property float x\nproperty float y\nproperty float z\n\
         property float nx\nproperty float ny\nproperty float nz\n\
         property float s\nproperty float t\n\
         element face {}\nproperty list uchar uint vertex_indices\nend_header\n",
        surface.positions.len(),
        surface.triangles.len()
    )
    .into_bytes();
    for ((p, n), uv) in surface
        .positions
        .iter()
        .zip(&surface.normals)
        .zip(&surface.uvs)
    {
        for v in [p.x, p.y, p.z, n.x, n.y, n.z, uv[0], uv[1]] {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    for t in &surface.triangles {
        out.push(3);
        for i in t {
            out.extend_from_slice(&i.to_le_bytes());
        }
    }
    out
}

struct Header {
    ascii: bool,
    body: usize,
    elements: Vec<Element>,
}

impl Header {
    fn read(bytes: &[u8]) -> Result<Self> {
        let end = bytes.windows(10).position(|w| w == b"end_header").ok_or(
            HairDataError::InvalidSignature {
                format: HairDataFormat::Ply,
            },
        )?;
        let header = std::str::from_utf8(&bytes[..end])?;
        let mut body = end + 10;
        while body < bytes.len() && (bytes[body] == b'\r' || bytes[body] == b'\n') {
            body += 1;
            if bytes[body - 1] == b'\n' {
                break;
            }
        }
        let mut ascii = false;
        let mut elements: Vec<Element> = Vec::new();
        for line in header.lines() {
            let words: Vec<&str> = line.split_whitespace().collect();
            match words.as_slice() {
                ["format", "ascii", ..] => ascii = true,
                ["format", "binary_little_endian", ..] => ascii = false,
                ["format", other, ..] => {
                    return Err(HairDataError::PlyEncoding {
                        name: (*other).into(),
                    });
                }
                ["element", name, count] => elements.push(Element {
                    name: name.to_string(),
                    count: count.parse()?,
                    properties: Vec::new(),
                }),
                ["property", "list", count, item, name] => elements
                    .last_mut()
                    .ok_or(HairDataError::PlyPropertyWithoutElement)?
                    .properties
                    .push(Property::List(
                        name.to_string(),
                        Scalar::parse(count)?,
                        Scalar::parse(item)?,
                    )),
                ["property", kind, name] => elements
                    .last_mut()
                    .ok_or(HairDataError::PlyPropertyWithoutElement)?
                    .properties
                    .push(Property::Scalar(name.to_string(), Scalar::parse(kind)?)),
                _ => {}
            }
        }
        Ok(Self {
            ascii,
            body,
            elements,
        })
    }
}

pub fn read(bytes: &[u8]) -> Result<Ply> {
    let Header {
        ascii,
        body,
        elements,
    } = Header::read(bytes)?;

    let mut ply = Ply {
        vertex_properties: Vec::new(),
        vertices: Vec::new(),
        faces: Vec::new(),
    };
    let mut tokens = if ascii {
        Some(std::str::from_utf8(&bytes[body..])?.split_whitespace())
    } else {
        None
    };
    let mut cursor = body;
    let mut next = |kind: Scalar| -> Result<f64> {
        if let Some(tokens) = tokens.as_mut() {
            Ok(tokens
                .next()
                .ok_or(HairDataError::Truncated {
                    format: HairDataFormat::Ply,
                })?
                .parse()?)
        } else {
            let size = kind.size();
            let b = bytes
                .get(cursor..cursor + size)
                .ok_or(HairDataError::Truncated {
                    format: HairDataFormat::Ply,
                })?;
            cursor += size;
            Ok(kind.read(b))
        }
    };
    for element in &elements {
        let is_vertex = element.name == "vertex";
        let is_face = element.name == "face";
        if is_vertex {
            ply.vertex_properties = element
                .properties
                .iter()
                .map(|p| match p {
                    Property::Scalar(n, _) | Property::List(n, _, _) => n.clone(),
                })
                .collect();
        }
        for _ in 0..element.count {
            let mut row = Vec::new();
            let mut face = Vec::new();
            for property in &element.properties {
                match property {
                    Property::Scalar(_, kind) => row.push(next(*kind)? as f32),
                    Property::List(name, count, item) => {
                        let n = next(*count)? as usize;
                        for _ in 0..n {
                            let v = next(*item)?;
                            if is_face && (name == "vertex_indices" || name == "vertex_index") {
                                face.push(v as u32);
                            }
                        }
                        row.push(0.0);
                    }
                }
            }
            if is_vertex {
                ply.vertices.push(row);
            } else if is_face {
                ply.faces.push(face);
            }
        }
    }
    Ok(ply)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_an_ascii_quad() {
        let text = "ply\nformat ascii 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nproperty float s\nproperty float t\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n0 0 0 0 0\n1 0 0 1 0\n1 1 0 1 1\n0 1 0 0 1\n4 0 1 2 3\n";
        let ply = read(text.as_bytes()).unwrap();
        assert_eq!(ply.vertices.len(), 4);
        assert_eq!(ply.faces, vec![vec![0, 1, 2, 3]]);
        assert_eq!(ply.vertex_properties[3], "s");
    }

    #[test]
    fn reads_binary() {
        let mut bytes = b"ply\nformat binary_little_endian 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nproperty list uchar uint vertex_indices\nend_header\n".to_vec();
        for v in [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        bytes.push(3);
        for i in [0u32, 1, 2] {
            bytes.extend_from_slice(&i.to_le_bytes());
        }
        let ply = read(&bytes).unwrap();
        assert_eq!(ply.vertices[1], vec![1.0, 0.0, 0.0]);
        assert_eq!(ply.faces, vec![vec![0, 1, 2]]);
    }
}
