//! Binary products encode six-vertex cells once, with their existing topology.
//! Documents retain explicit indices. Executable-bound caches never mix codecs.
use super::*;
use serde::{Deserializer, Serializer, de::Error};

#[derive(Serialize, Deserialize)]
#[serde(remote = "PropertyFoundationMesh")]
struct FoundationDocument {
    property_id: CityPropertyId,
    member_building_ids: Vec<crate::scene_input::SceneBuildingId>,
    #[serde(with = "crate::geometry_transport")]
    positions: Vec<Vec3>,
    #[serde(with = "crate::geometry_transport")]
    solid_triangles: Vec<[u32; 3]>,
    #[serde(with = "crate::geometry_transport")]
    support_triangles: Vec<[u32; 3]>,
    #[serde(with = "crate::geometry_transport")]
    cut_faces: Vec<[Vec3; 3]>,
}

#[derive(Serialize, Deserialize)]
struct FoundationCells {
    property_id: CityPropertyId,
    member_building_ids: Vec<crate::scene_input::SceneBuildingId>,
    #[serde(with = "crate::geometry_transport::binary")]
    positions: Vec<Vec3>,
    #[serde(with = "crate::geometry_transport::binary")]
    cut_faces: Vec<[Vec3; 3]>,
}

/// Borrow geometry during encoding rather than cloning a settlement's cells.
#[derive(Serialize)]
struct FoundationCellsRef<'a> {
    property_id: CityPropertyId,
    member_building_ids: &'a [crate::scene_input::SceneBuildingId],
    #[serde(with = "crate::geometry_transport::binary")]
    positions: &'a [Vec3],
    #[serde(with = "crate::geometry_transport::binary")]
    cut_faces: &'a [[Vec3; 3]],
}

#[derive(Debug, thiserror::Error)]
enum FoundationEncodingIssue {
    #[error("foundation must contain complete six-vertex cells within u32 indexing")]
    Cells,
    #[error("foundation indices differ from the closed six-vertex prism topology")]
    Topology,
}

fn cell_count(positions: &[Vec3]) -> Result<usize, FoundationEncodingIssue> {
    if !positions.len().is_multiple_of(6) || u32::try_from(positions.len()).is_err() {
        return Err(FoundationEncodingIssue::Cells);
    }
    Ok(positions.len() / 6)
}

impl Serialize for PropertyFoundationMesh {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.validate().map_err(serde::ser::Error::custom)?;
        if serializer.is_human_readable() {
            return FoundationDocument::serialize(self, serializer);
        }
        let cells = cell_count(&self.positions).map_err(serde::ser::Error::custom)?;
        let topology_matches = self.support_triangles.len() == cells
            && self.solid_triangles.len() == cells * FOUNDATION_PRISM_TRIANGLES.len()
            && (0..cells).all(|cell| {
                let start = (cell * 6) as u32;
                self.support_triangles[cell] == [start, start + 2, start + 1]
                    && self.solid_triangles[cell * FOUNDATION_PRISM_TRIANGLES.len()
                        ..(cell + 1) * FOUNDATION_PRISM_TRIANGLES.len()]
                        == FOUNDATION_PRISM_TRIANGLES.map(|t| t.map(|i| i + start))
            });
        if !topology_matches {
            return Err(serde::ser::Error::custom(FoundationEncodingIssue::Topology));
        }
        FoundationCellsRef {
            property_id: self.property_id,
            member_building_ids: &self.member_building_ids,
            positions: &self.positions,
            cut_faces: &self.cut_faces,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PropertyFoundationMesh {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let mesh = FoundationDocument::deserialize(deserializer)?;
            mesh.validate().map_err(D::Error::custom)?;
            return Ok(mesh);
        }
        let cells = FoundationCells::deserialize(deserializer)?;
        let count = cell_count(&cells.positions).map_err(D::Error::custom)?;
        let mut mesh = Self {
            property_id: cells.property_id,
            member_building_ids: cells.member_building_ids,
            positions: cells.positions,
            cut_faces: cells.cut_faces,
            solid_triangles: Vec::with_capacity(count * FOUNDATION_PRISM_TRIANGLES.len()),
            support_triangles: Vec::with_capacity(count),
        };
        for cell in 0..count {
            let start = (cell * 6) as u32;
            mesh.support_triangles.push([start, start + 2, start + 1]);
            mesh.solid_triangles
                .extend(FOUNDATION_PRISM_TRIANGLES.map(|t| t.map(|i| i + start)));
        }
        mesh.validate().map_err(D::Error::custom)?;
        Ok(mesh)
    }
}

#[cfg(test)]
mod tests;
