use crate::{Dtype, NpyByteStates, NpyFloatValues, NpyIntegerValues, npy};
fn encode(dtype: &str, shape: &str, payload: &[u8]) -> Vec<u8> {
    let header = format!("{{'descr': '{dtype}', 'fortran_order': False, 'shape': ({shape}), }}");
    let mut padded = header.into_bytes();
    while (10 + padded.len()) % 64 != 63 {
        padded.push(b' ');
    }
    padded.push(b'\n');
    let mut out = b"\x93NUMPY\x01\x00".to_vec();
    out.extend((padded.len() as u16).to_le_bytes());
    out.extend(padded);
    out.extend(payload);
    out
}
fn numeric_cases() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (
            "<f4",
            [
                0.0f32,
                -0.0,
                1.9,
                -1.9,
                f32::from_bits(0x7fc1_2345),
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::MAX,
            ]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect(),
        ),
        (
            "<f8",
            [
                16_777_217.0f64,
                -16_777_217.0,
                f64::MAX,
                f64::NAN,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ]
            .into_iter()
            .flat_map(f64::to_le_bytes)
            .collect(),
        ),
        (
            "<i4",
            [i32::MIN, 0, i32::MAX]
                .into_iter()
                .flat_map(i32::to_le_bytes)
                .collect(),
        ),
        (
            "<i8",
            [i64::MIN, -9_007_199_254_740_993, i64::MAX]
                .into_iter()
                .flat_map(i64::to_le_bytes)
                .collect(),
        ),
        ("|u1", vec![0, 1, 2, 255]),
        ("|b1", vec![0, 1, 2, 255]),
    ]
}
fn values_observation() -> anyhow::Result<String> {
    let mut out = String::new();
    for (descr, payload) in numeric_cases() {
        let width = match descr {
            "<f4" | "<i4" => 4,
            "<f8" | "<i8" => 8,
            _ => 1,
        };
        let count = payload.len() / width;
        let array = npy::parse(&encode(descr, &format!("{count},"), &payload))?;
        out.push_str(&format!(
            "{descr}:floats={:?};integers={:?};bytes={:?}\n",
            Vec::<f32>::from(NpyFloatValues::from(&array))
                .into_iter()
                .map(f32::to_bits)
                .collect::<Vec<_>>(),
            Vec::<i64>::from(NpyIntegerValues::from(&array)),
            Vec::<bool>::from(NpyByteStates::from(&array))
        ));
    }
    for (shape, payload) in [("0,", vec![]), ("", 1.25f32.to_le_bytes().to_vec())] {
        let array = npy::parse(&encode("<f4", shape, &payload))?;
        out.push_str(&format!(
            "shape:{shape:?};floats={:?};integers={:?};bytes={:?}\n",
            Vec::<f32>::from(NpyFloatValues::from(&array))
                .into_iter()
                .map(f32::to_bits)
                .collect::<Vec<_>>(),
            Vec::<i64>::from(NpyIntegerValues::from(&array)),
            Vec::<bool>::from(NpyByteStates::from(&array))
        ));
    }
    let payload = [1.0f32, 2.0, 3.0]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    let mut array = npy::parse(&encode("<f4", "3,", &payload))?;
    array.dtype = Dtype::F64;
    out.push_str(&format!(
        "changed-dtype:floats={:?};integers={:?};bytes={:?}\n",
        Vec::<f32>::from(NpyFloatValues::from(&array))
            .into_iter()
            .map(f32::to_bits)
            .collect::<Vec<_>>(),
        Vec::<i64>::from(NpyIntegerValues::from(&array)),
        Vec::<bool>::from(NpyByteStates::from(&array))
    ));
    Ok(out)
}

#[test]
fn decoded_numeric_words_and_byte_states_match_original() -> anyhow::Result<()> {
    assert_eq!(values_observation()?, include_str!("fixtures/values.txt"));
    Ok(())
}
