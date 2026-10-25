//! Bulk transport for client-generated vertex buffers, without per-scalar CBOR.
//! Cache identity includes the executable, so native byte order cannot cross
//! target architectures or generator revisions.
use bytemuck::Pod;
use serde::{
    Deserializer, Serializer,
    de::{Error, Visitor},
};
use std::{fmt, marker::PhantomData};

pub fn serialize<T: Pod, S: Serializer>(values: &[T], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(bytemuck::cast_slice(values))
}

pub fn deserialize<'de, T: Pod, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<T>, D::Error> {
    struct Buffer<T>(PhantomData<T>);
    impl<'de, T: Pod> Visitor<'de> for Buffer<T> {
        type Value = Vec<T>;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a complete packed vertex or index buffer")
        }
        fn visit_bytes<E: Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
            if !bytes.len().is_multiple_of(size_of::<T>()) {
                return Err(E::custom("packed buffer has a partial element"));
            }
            // Copy into an aligned allocation; byte buffers need not be aligned.
            Ok(bytemuck::pod_collect_to_vec(bytes))
        }
        fn visit_byte_buf<E: Error>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
            self.visit_bytes(&bytes)
        }
    }
    deserializer.deserialize_byte_buf(Buffer(PhantomData))
}

#[cfg(test)]
mod tests {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Vertices(#[serde(with = "super")] Vec<[f32; 3]>);

    #[test]
    fn packed_transport_preserves_float_bits_and_rejects_partial_vertices() {
        // Exercise payloads larger than the CBOR decoder's scratch buffer.
        let source = Vertices(vec![[f32::from_bits(0x80000000), 1.25, -41.125]; 20_000]);
        let mut bytes = Vec::new();
        ciborium::into_writer(&source, &mut bytes).unwrap();
        let restored: Vertices = ciborium::from_reader(bytes.as_slice()).unwrap();
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&source.0),
            bytemuck::cast_slice::<_, u8>(&restored.0)
        );
        assert!(ciborium::from_reader::<Vertices, _>(&[0x41_u8, 0][..]).is_err());
    }
}
