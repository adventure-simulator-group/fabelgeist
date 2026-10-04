//! Native IEEE word decoding and encoding for independent solver snapshots.
use fabelgeist_math::Vec3;
use serde_json::Value;

pub(crate) fn decode_native_vectors(value: &Value) -> Vec<Vec3> {
    let mut vectors = Vec::new();
    for words in value.as_array().expect("native vector records") {
        let x = f32::from_bits(words[0].as_u64().expect("x word") as u32);
        let y = f32::from_bits(words[1].as_u64().expect("y word") as u32);
        let z = f32::from_bits(words[2].as_u64().expect("z word") as u32);
        vectors.push(Vec3::new(x, y, z));
    }
    vectors
}
pub(crate) fn encode_native_vectors(vectors: &[Vec3]) -> Value {
    let mut records = Vec::new();
    for vector in vectors {
        records.push([vector.x.to_bits(), vector.y.to_bits(), vector.z.to_bits()]);
    }
    serde_json::json!(records)
}
