use std::io::Write;
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
fn zip_bytes(members: &[(&[u8], Vec<u8>)], method: u16, zip64: bool) -> Vec<u8> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, payload) in members {
        let offset = out.len() as u32;
        let raw = if method == 8 {
            let mut encoder =
                flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(payload).unwrap();
            encoder.finish().unwrap()
        } else {
            payload.clone()
        };
        out.extend(0x0403_4b50u32.to_le_bytes());
        out.extend(20u16.to_le_bytes());
        out.extend(0u16.to_le_bytes());
        out.extend(method.to_le_bytes());
        out.extend([0; 4]);
        out.extend([0; 4]);
        out.extend((raw.len() as u32).to_le_bytes());
        out.extend((payload.len() as u32).to_le_bytes());
        out.extend((name.len() as u16).to_le_bytes());
        out.extend(0u16.to_le_bytes());
        out.extend(*name);
        out.extend(&raw);
        directory.extend(0x0201_4b50u32.to_le_bytes());
        directory.extend([20, 0, 20, 0, 0, 0]);
        directory.extend(method.to_le_bytes());
        directory.extend([0; 8]);
        directory.extend((raw.len() as u32).to_le_bytes());
        directory.extend((payload.len() as u32).to_le_bytes());
        directory.extend((name.len() as u16).to_le_bytes());
        directory.extend([0; 12]);
        directory.extend(offset.to_le_bytes());
        directory.extend(*name);
    }
    let directory_offset = out.len() as u64;
    let directory_size = directory.len() as u32;
    out.extend(directory);
    if zip64 {
        let record = out.len() as u64;
        out.extend(0x0606_4b50u32.to_le_bytes());
        out.extend(44u64.to_le_bytes());
        out.extend([45, 0, 45, 0]);
        out.extend([0; 8]);
        out.extend((members.len() as u64).to_le_bytes());
        out.extend((members.len() as u64).to_le_bytes());
        out.extend((directory_size as u64).to_le_bytes());
        out.extend(directory_offset.to_le_bytes());
        out.extend(0x0706_4b50u32.to_le_bytes());
        out.extend([0; 4]);
        out.extend(record.to_le_bytes());
        out.extend(1u32.to_le_bytes());
    }
    out.extend(0x0605_4b50u32.to_le_bytes());
    out.extend([0; 4]);
    let count = if zip64 {
        u16::MAX
    } else {
        members.len() as u16
    };
    out.extend(count.to_le_bytes());
    out.extend(count.to_le_bytes());
    out.extend(directory_size.to_le_bytes());
    out.extend(
        if zip64 {
            u32::MAX
        } else {
            directory_offset as u32
        }
        .to_le_bytes(),
    );
    out.extend([0; 2]);
    out
}
use crate::{ArchiveMemberName, Npz, NpzArrayName, ZipArchive};
fn lookup_observation() -> anyhow::Result<String> {
    let names: Vec<&[u8]> = vec![
        b"weights.npy",
        b"weights.npy.npy",
        b"weights",
        "目录/β.npy".as_bytes(),
        b"\x80.npy",
        b"\x81.npy",
        b".npy",
        b"",
        b"path/../raw.npy",
        b"\0.npy",
        b"weights.npy",
    ];
    let members = names
        .iter()
        .enumerate()
        .map(|(slot, name)| {
            (
                *name,
                encode(
                    "<f4",
                    &format!("{},", slot % 3 + 1),
                    &(0..slot % 3 + 1)
                        .flat_map(|_| (slot as f32 + 1.0).to_le_bytes())
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect::<Vec<_>>();
    let queries = [
        "weights",
        "weights.npy",
        "weights.npy.npy",
        "Weights",
        "",
        "missing",
        "\0",
        "目录/β",
        "\u{fffd}",
        "path/../raw",
    ];
    let mut out = String::new();
    for (method, zip64, reversed) in [
        (0, false, false),
        (8, false, false),
        (0, true, false),
        (8, true, false),
        (0, false, true),
    ] {
        let mut ordered = members.clone();
        if reversed {
            ordered.reverse();
        }
        let bytes = zip_bytes(&ordered, method, zip64);
        let archive = ZipArchive::from_bytes(bytes.clone())?;
        let npz = Npz::from_bytes(bytes)?;
        out.push_str(&format!(
            "archive:{method},{zip64},{reversed};names={:?};keys={:?}\n",
            archive.names().collect::<Vec<_>>(),
            npz.keys().collect::<Vec<_>>()
        ));
        for query in queries {
            let data = archive
                .bytes(&ArchiveMemberName::from(query))
                .map(|data| data.to_vec())
                .map_err(|error| format!("{error:#}"));
            let values = npz
                .array(&NpzArrayName::from(query))
                .map(|array| {
                    array
                        .to_f32()
                        .into_iter()
                        .map(f32::to_bits)
                        .collect::<Vec<_>>()
                })
                .map_err(|error| format!("{error:#}"));
            out.push_str(&format!(
                "query:{query:?};zip={:?},{:?},{data:?};npz={:?},{:?},{values:?}\n",
                bool::from(archive.contains(&ArchiveMemberName::from(query))),
                archive.uncompressed_size(&ArchiveMemberName::from(query)),
                bool::from(npz.contains(&NpzArrayName::from(query))),
                npz.uncompressed_size(&NpzArrayName::from(query))
            ));
        }
    }
    for (label, method, payload, corrupt) in [
        ("decode", 0, b"not-npy".to_vec(), false),
        ("compression", 99, encode("<f4", "1,", &[0; 4]), false),
        ("local-header", 0, encode("<f4", "1,", &[0; 4]), true),
    ] {
        let mut bytes = zip_bytes(&[(b"bad.npy", payload)], method, false);
        if corrupt {
            bytes[0] = 0;
        }
        let archive = ZipArchive::from_bytes(bytes.clone())?;
        let npz = Npz::from_bytes(bytes)?;
        out.push_str(&format!(
            "failure:{label};zip={:?};npz={:?}\n",
            archive
                .bytes(&ArchiveMemberName::from("bad.npy"))
                .map(|v| v.to_vec())
                .map_err(|e| format!("{e:#}")),
            npz.array(&NpzArrayName::new("bad"))
                .map(|a| a.to_f32())
                .map_err(|e| format!("{e:#}"))
        ));
    }
    Ok(out)
}
#[test]
fn lookup_identity_preserves_archive_order_spelling_and_failures() {
    let actual = lookup_observation().unwrap();
    assert_eq!(actual, include_str!("fixtures/lookups.txt"));
}
