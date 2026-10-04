use super::{CharacterDecodeError, Mesh, PolygonCornerCount, RigArrayCount, RigObjectContext};
use fabelgeist_fbx::{FbxRecordName, Object};

/// Splits an FBX `PolygonVertexIndex` stream (negative index ends a polygon)
/// into fan-triangulated triangles.
pub(super) fn triangulate(
    polygon_vertex_index: &[i64],
) -> Result<Vec<[u32; 3]>, CharacterDecodeError> {
    let mut faces = Vec::new();
    let mut polygon: Vec<u32> = Vec::with_capacity(4);
    for raw in polygon_vertex_index {
        let (index, last) = if *raw < 0 {
            ((-(*raw + 1)) as u32, true)
        } else {
            (*raw as u32, false)
        };
        polygon.push(index);
        if !last {
            continue;
        }
        if polygon.len() < 3 {
            return Err(CharacterDecodeError::PolygonTooShort(
                PolygonCornerCount::from(polygon.len()),
            ));
        }
        for j in 1..polygon.len() - 1 {
            faces.push([polygon[0], polygon[j], polygon[j + 1]]);
        }
        polygon.clear();
    }
    if !polygon.is_empty() {
        return Err(CharacterDecodeError::TrailingPolygon(
            PolygonCornerCount::from(polygon.len()),
        ));
    }
    Ok(faces)
}

pub(super) fn parse_uvs(
    geometry: &Object,
    polygon_count: usize,
) -> Result<(Vec<[f32; 2]>, Vec<i64>), CharacterDecodeError> {
    let Some(layer) = geometry.node.child(&FbxRecordName::LAYER_ELEMENT_UV) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let Some(values) = layer.child(&FbxRecordName::UV).and_then(|n| n.f64_array()) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let (uv_pairs, remainder) = values.as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(CharacterDecodeError::IncompleteTexcoord {
            geometry: RigObjectContext::from(geometry),
            values: RigArrayCount::from(values.len()),
        });
    }
    // FBX texture coordinates are bottom-up.
    let texcoords: Vec<[f32; 2]> = uv_pairs
        .iter()
        .map(|uv| [uv[0] as f32, 1.0 - uv[1] as f32])
        .collect();

    let reference = layer
        .child(&FbxRecordName::REFERENCE_INFORMATION_TYPE)
        .and_then(|n| n.str_prop(0))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let indices = if reference == b"IndexToDirect" {
        layer
            .child(&FbxRecordName::UV_INDEX)
            .and_then(|n| n.i64_array())
            .unwrap_or_default()
    } else {
        (0..polygon_count as i64).collect()
    };
    Ok((texcoords, indices))
}

/// Re-splits UV indices onto the polygon boundaries of the vertex index stream.
pub(super) fn triangulate_texcoords(
    polygon_vertex_index: &[i64],
    texcoord_indices: &[i64],
) -> Vec<[u32; 3]> {
    if texcoord_indices.len() != polygon_vertex_index.len() {
        return Vec::new();
    }
    let mut faces = Vec::new();
    let mut polygon: Vec<u32> = Vec::with_capacity(4);
    for (raw, tex) in polygon_vertex_index.iter().zip(texcoord_indices) {
        polygon.push(*tex as u32);
        if *raw >= 0 {
            continue;
        }
        for j in 1..polygon.len().saturating_sub(1) {
            faces.push([polygon[0], polygon[j], polygon[j + 1]]);
        }
        polygon.clear();
    }
    faces
}

impl Mesh {
    pub(super) fn from_geometry(geometry: &Object) -> Result<Self, CharacterDecodeError> {
        let vertex_values = geometry
            .node
            .child(&FbxRecordName::VERTICES)
            .and_then(|n| n.f64_array())
            .ok_or_else(|| -> CharacterDecodeError {
                CharacterDecodeError::MissingVertices(RigObjectContext::from(geometry))
            })?;
        let (vertex_triples, remainder) = vertex_values.as_chunks::<3>();
        if !remainder.is_empty() {
            return Err(CharacterDecodeError::IncompletePosition {
                geometry: RigObjectContext::from(geometry),
                values: RigArrayCount::from(vertex_values.len()),
            });
        }
        let vertices: Vec<[f32; 3]> = vertex_triples
            .iter()
            .map(|v| [v[0] as f32, v[1] as f32, v[2] as f32])
            .collect();
        let polygon_vertex_index = geometry
            .node
            .child(&FbxRecordName::POLYGON_VERTEX_INDEX)
            .and_then(|n| n.i64_array())
            .ok_or_else(|| -> CharacterDecodeError {
                CharacterDecodeError::MissingPolygons(RigObjectContext::from(geometry))
            })?;

        let faces = triangulate(&polygon_vertex_index)?;
        let (texcoords, texcoord_indices) = parse_uvs(geometry, polygon_vertex_index.len())?;
        let texcoord_faces = triangulate_texcoords(&polygon_vertex_index, &texcoord_indices);

        Ok(Self {
            vertices,
            faces,
            texcoords,
            texcoord_faces,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulates_polygon_fans() {
        // Two polygons: a quad and a triangle, FBX-style negative terminators.
        let stream = [0, 1, 2, -4, 4, 5, -7];
        let faces = triangulate(&stream).unwrap();
        assert_eq!(faces, vec![[0, 1, 2], [0, 2, 3], [4, 5, 6]]);
    }

    #[test]
    fn rejects_degenerate_polygons() {
        assert!(triangulate(&[0, -2]).is_err());
        assert!(triangulate(&[0, 1, 2]).is_err());
    }
}
