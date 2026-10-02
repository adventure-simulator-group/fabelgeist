//! Deterministic uncompressed ZIP32, including CRCs, for browser downloads.
use super::ExportFile;
pub fn zip(files: &[ExportFile]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for file in files {
        let start = out.len() as u32;
        let length = file.bytes.len() as u32;
        let name = file.name.as_bytes();
        let crc = crc32(&file.bytes);
        u32s(&mut out, &[0x0403_4b50]);
        u16s(&mut out, &[20, 0, 0, 0, 33]);
        u32s(&mut out, &[crc, length, length]);
        u16s(&mut out, &[name.len() as u16, 0]);
        out.extend(name);
        out.extend(&file.bytes);
        u32s(&mut directory, &[0x0201_4b50]);
        u16s(&mut directory, &[20, 20, 0, 0, 0, 33]);
        u32s(&mut directory, &[crc, length, length]);
        u16s(&mut directory, &[name.len() as u16, 0, 0, 0, 0]);
        u32s(&mut directory, &[0, start]);
        directory.extend(name);
    }
    let offset = out.len() as u32;
    let length = directory.len() as u32;
    out.extend(directory);
    u32s(&mut out, &[0x0605_4b50]);
    u16s(&mut out, &[0, 0, files.len() as u16, files.len() as u16]);
    u32s(&mut out, &[length, offset]);
    u16s(&mut out, &[0]);
    out
}
fn u16s(out: &mut Vec<u8>, values: &[u16]) {
    for v in values {
        out.extend(v.to_le_bytes());
    }
}
fn u32s(out: &mut Vec<u8>, values: &[u32]) {
    for v in values {
        out.extend(v.to_le_bytes());
    }
}
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
