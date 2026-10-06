use super::ZipArchive;
use flate2::{Compression, write::DeflateEncoder};
use std::{borrow::Cow, io::Write};

fn put16(out: &mut Vec<u8>, value: u16) {
    out.extend(value.to_le_bytes());
}
fn put32(out: &mut Vec<u8>, value: u32) {
    out.extend(value.to_le_bytes());
}

// These are intentionally serialized ZIP fixture fields, not admitted entries.
fn archive(method: u16, local_method: u16, raw: &[u8], declared: u32) -> Vec<u8> {
    let name = b"entry";
    let mut out = Vec::new();
    put32(&mut out, 0x0403_4b50);
    for value in [20, 0, local_method, 0, 0] {
        put16(&mut out, value);
    }
    for value in [0, raw.len() as u32, 17] {
        put32(&mut out, value);
    }
    put16(&mut out, name.len() as u16);
    put16(&mut out, 0);
    out.extend(name);
    out.extend(raw);
    let central = out.len() as u32;
    put32(&mut out, 0x0201_4b50);
    for value in [20, 20, 0, method, 0, 0] {
        put16(&mut out, value);
    }
    for value in [0, declared, 17] {
        put32(&mut out, value);
    }
    for value in [name.len() as u16, 0, 0, 0, 0] {
        put16(&mut out, value);
    }
    for value in [0, 0] {
        put32(&mut out, value);
    }
    out.extend(name);
    let directory_size = out.len() as u32 - central;
    put32(&mut out, 0x0605_4b50);
    for value in [0, 0, 1, 1] {
        put16(&mut out, value);
    }
    for value in [directory_size, central] {
        put32(&mut out, value);
    }
    put16(&mut out, 0);
    out
}

fn snapshot(zip: &ZipArchive) -> String {
    let mut out = format!(
        "names={:?};contains={};size={:?};missing={:?};",
        zip.names().collect::<Vec<_>>(),
        zip.contains("entry"),
        zip.uncompressed_size("entry"),
        zip.uncompressed_size("missing")
    );
    for key in ["entry", "missing"] {
        let result = match zip.bytes(key) {
            Ok(Cow::Borrowed(data)) => format!("borrowed:{data:?}"),
            Ok(Cow::Owned(data)) => format!("owned:{data:?}"),
            Err(error) => format!("error:{error:#}"),
        };
        out.push_str(&format!("{key}={result};"));
    }
    out.push('\n');
    out
}

fn observe() -> String {
    let payload = b"compression-proof";
    let mut deflater = DeflateEncoder::new(Vec::new(), Compression::default());
    deflater.write_all(payload).unwrap();
    let deflated = deflater.finish().unwrap();
    let mut out = String::new();
    let mut cases = vec![
        ("stored", archive(0, 0, payload, payload.len() as u32)),
        ("deflated", archive(8, 8, &deflated, deflated.len() as u32)),
        (
            "stored-central-deflate-local",
            archive(0, 8, payload, payload.len() as u32),
        ),
        (
            "deflate-central-stored-local",
            archive(8, 0, &deflated, deflated.len() as u32),
        ),
        ("corrupt-deflate", archive(8, 8, &[0xff, 0xff], 2)),
        ("empty-stored", archive(0, 0, &[], 0)),
    ];
    for method in [1, 12, 99, u16::MAX] {
        let bytes = archive(method, method, payload, payload.len() as u32);
        out.push_str(&format!(
            "unknown-{method}:{}",
            snapshot(&ZipArchive::from_bytes(bytes).unwrap())
        ));
    }
    let mut corrupt = archive(99, 99, payload, payload.len() as u32);
    corrupt[0] = 0;
    cases.push(("unknown-corrupt-local", corrupt));
    cases.push(("unknown-past-end", archive(99, 99, payload, 65536)));
    for (label, bytes) in cases {
        let zip = ZipArchive::from_bytes(bytes.clone()).unwrap();
        out.push_str(&format!("owned-{label}:{}", snapshot(&zip)));
        let path = std::env::temp_dir().join(format!(
            "zip-compression-726-{}-{label}.zip",
            std::process::id()
        ));
        std::fs::write(&path, bytes).unwrap();
        let mapped = ZipArchive::open(&path).unwrap();
        out.push_str(&format!("mapped-{label}:{}", snapshot(&mapped)));
        drop(mapped);
        std::fs::remove_file(path).unwrap();
    }
    out
}

#[test]
fn member_decoding_preserves_actual_main_compression_behavior() {
    assert_eq!(observe(), include_str!("fixtures/compression.txt"));
}
