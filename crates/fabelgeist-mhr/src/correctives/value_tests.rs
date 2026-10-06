use super::*;
use fabelgeist_numpy_storage::npy;
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
use std::io::Write;
const END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
const CENTRAL_FILE_HEADER: u32 = 0x0201_4b50;
const LOCAL_FILE_HEADER: u32 = 0x0403_4b50;

/// Builds a minimal stored-member zip so the reader can be tested without
/// depending on an external archiver.
fn zip(members: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, payload) in members {
        let offset = out.len() as u32;
        let crc = 0u32;
        out.extend_from_slice(&LOCAL_FILE_HEADER.to_le_bytes());
        out.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // version, flags, method, time, date
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.write_all(payload).unwrap();

        directory.extend_from_slice(&CENTRAL_FILE_HEADER.to_le_bytes());
        directory.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        directory.extend_from_slice(&crc.to_le_bytes());
        directory.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        directory.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        directory.extend_from_slice(&(name.len() as u16).to_le_bytes());
        directory.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        directory.extend_from_slice(&offset.to_le_bytes());
        directory.extend_from_slice(name.as_bytes());
    }

    let directory_offset = out.len() as u32;
    let directory_size = directory.len() as u32;
    out.extend_from_slice(&directory);
    out.extend_from_slice(&END_OF_CENTRAL_DIRECTORY.to_le_bytes());
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&(members.len() as u16).to_le_bytes());
    out.extend_from_slice(&(members.len() as u16).to_le_bytes());
    out.extend_from_slice(&directory_size.to_le_bytes());
    out.extend_from_slice(&directory_offset.to_le_bytes());
    out.extend_from_slice(&[0, 0]);
    out
}

fn activation_case(kind: usize) -> anyhow::Result<String> {
    let basis_values = (0..72)
        .map(|v| v as f64 / 8.0)
        .flat_map(f64::to_le_bytes)
        .collect::<Vec<_>>();
    let mut basis = vec![(
        "corrective_blendshapes.npy",
        encode("<f8", "24,1,3", &basis_values),
    )];
    let (indices, values): (Vec<i64>, Vec<f32>) = match kind {
        0 => (vec![2, 2, 1, 1], vec![1.0, -0.0]),
        1 => (vec![23, 5], vec![2.5]),
        2 => (vec![0], vec![1.0]),
        3 => (vec![-1, 0], vec![1.0]),
        4 => (vec![24, 0], vec![1.0]),
        5 => (vec![0, 6], vec![1.0]),
        6 => {
            basis.clear();
            (vec![], vec![])
        }
        _ => (vec![], vec![]),
    };
    let idx = indices
        .iter()
        .copied()
        .flat_map(i64::to_le_bytes)
        .collect::<Vec<_>>();
    let val = values
        .iter()
        .copied()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    let activation = zip(&[
        (
            "0.sparse_indices.npy",
            encode("<i8", &format!("{},", indices.len()), &idx),
        ),
        (
            "0.sparse_weight.npy",
            encode("<f4", &format!("{},", values.len()), &val),
        ),
    ]);
    observe_model(activation, zip(&basis))
}
fn tensor_observation() -> anyhow::Result<String> {
    let payload = [1.25f64, -2.5, 16_777_217.0]
        .into_iter()
        .flat_map(f64::to_le_bytes)
        .collect::<Vec<_>>();
    let array = npy::parse(&encode("<f8", "3,", &payload))?;
    let floats = array
        .to_tensor::<1>(&Device::default())?
        .into_data()
        .into_vec::<f32>()?;
    let integers = array
        .to_int_tensor::<1>(&Device::default())?
        .into_data()
        .into_vec::<i32>()?;
    Ok(format!(
        "float={:?};integer={integers:?}",
        floats.into_iter().map(f32::to_bits).collect::<Vec<_>>()
    ))
}

fn observe_model(activation: Vec<u8>, basis: Vec<u8>) -> anyhow::Result<String> {
    let model = PoseCorrectives::from_bytes(activation, basis, 3, 1, &Device::default())?;
    match model {
        None => Ok("none".into()),
        Some(model) => {
            let dims = model.activation.dims();
            let values = model.activation.into_data().into_vec::<f32>()?;
            let basis = model.basis.into_data().into_vec::<f32>()?;
            Ok(format!(
                "dims={dims:?};matrix={:?};basis={:?}",
                values.into_iter().map(f32::to_bits).collect::<Vec<_>>(),
                basis.into_iter().map(f32::to_bits).collect::<Vec<_>>()
            ))
        }
    }
}

#[test]
fn native_tensor_and_sparse_activation_match_original() -> Result<()> {
    let mut output = format!("tensor:{}\n", tensor_observation()?);
    for kind in 0..8 {
        output.push_str(&format!("activation{kind}:{:?}\n", activation_case(kind)));
    }
    let expected = include_str!("fixtures/values.txt")
        .replace("18446744073709551615", &usize::MAX.to_string());
    assert_eq!(output, expected);
    Ok(())
}
