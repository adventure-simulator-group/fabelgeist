//! ZIP's stored method keeps exports portable without a second compression runtime.
const LOCAL_HEADER: u32 = 0x0403_4b50;
const CENTRAL_HEADER: u32 = 0x0201_4b50;
const END_HEADER: u32 = 0x0605_4b50;
const CRC_POLYNOMIAL: u32 = 0xedb8_8320;

fn crc(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (CRC_POLYNOMIAL & (0_u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}
fn word(out: &mut Vec<u8>, n: u16) {
    out.extend_from_slice(&n.to_le_bytes());
}
fn dword(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}

pub(super) fn stored_zip(files: Vec<(String, Vec<u8>)>) -> Vec<u8> {
    let mut output = vec![];
    let mut directory = vec![];
    let count = files.len() as u16;
    for (name, bytes) in files {
        let offset = output.len() as u32;
        let checksum = crc(&bytes);
        let size = bytes.len() as u32;
        dword(&mut output, LOCAL_HEADER);
        word(&mut output, 20);
        for n in [0_u16, 0, 0, 0] {
            word(&mut output, n);
        }
        for n in [checksum, size, size] {
            dword(&mut output, n);
        }
        word(&mut output, name.len() as u16);
        word(&mut output, 0);
        output.extend_from_slice(name.as_bytes());
        output.extend(bytes);
        dword(&mut directory, CENTRAL_HEADER);
        for n in [20_u16, 20, 0, 0, 0, 0] {
            word(&mut directory, n);
        }
        for n in [checksum, size, size] {
            dword(&mut directory, n);
        }
        word(&mut directory, name.len() as u16);
        for _ in 0..4 {
            word(&mut directory, 0);
        }
        dword(&mut directory, 0);
        dword(&mut directory, offset);
        directory.extend_from_slice(name.as_bytes());
    }
    let offset = output.len() as u32;
    let size = directory.len() as u32;
    output.extend(directory);
    dword(&mut output, END_HEADER);
    word(&mut output, 0);
    word(&mut output, 0);
    word(&mut output, count);
    word(&mut output, count);
    dword(&mut output, size);
    dword(&mut output, offset);
    word(&mut output, 0);
    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn crc_matches_the_zip_standard_check_value() {
        assert_eq!(super::crc(b"123456789"), 0xcbf4_3926);
    }
}
