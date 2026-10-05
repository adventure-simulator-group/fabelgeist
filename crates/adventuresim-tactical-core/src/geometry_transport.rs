//! Structured source documents and bulk binary geometry share exact scalar bits.
//! Binary cache keys include the executable and its target. Human-readable
//! scene documents retain their portable arrays; worker and replication buffers
//! use aligned POD copies rather than encoding each vertex or index separately.
use bytemuck::Pod;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub mod binary;

pub fn serialize<T: Pod + Serialize, S: Serializer>(
    values: &[T],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if serializer.is_human_readable() {
        values.serialize(serializer)
    } else {
        binary::serialize(values, serializer)
    }
}

pub fn deserialize<'de, T: Pod + Deserialize<'de>, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<T>, D::Error> {
    if deserializer.is_human_readable() {
        Vec::deserialize(deserializer)
    } else {
        binary::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec3;
    #[derive(Serialize, Deserialize)]
    struct Geometry(#[serde(with = "super")] Vec<[Vec3; 3]>);

    #[test]
    fn human_documents_keep_arrays_and_binary_geometry_preserves_every_bit() {
        let geometry = Geometry(vec![[Vec3::new(-0.0, 1.25, -41.125); 3]; 20_000]);
        let json = serde_json::to_value(&geometry).unwrap();
        assert!(json[0][0].is_array());
        let document: Geometry = serde_json::from_value(json).unwrap();
        let mut bytes = Vec::new();
        ciborium::into_writer(&geometry, &mut bytes).unwrap();
        let restored: Geometry = ciborium::from_reader(bytes.as_slice()).unwrap();
        let expected = bytemuck::cast_slice::<_, u8>(&geometry.0);
        assert_eq!(expected, bytemuck::cast_slice::<_, u8>(&document.0));
        assert_eq!(expected, bytemuck::cast_slice::<_, u8>(&restored.0));
        assert!(bytes.len() <= expected.len() + 8);
        assert!(ciborium::from_reader::<Geometry, _>(&[0x41_u8, 0][..]).is_err());
    }
}
