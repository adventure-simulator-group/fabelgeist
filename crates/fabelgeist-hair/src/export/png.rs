//! Just enough PNG to write a grey image: DiffLocks' density maps are PNGs,
//! and the strands need no image library to write one.

/// An 8-bit greyscale PNG.
pub fn gray(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    // Each row behind a filter byte (0: none), deflated as one zlib stream.
    let mut raw = Vec::with_capacity((width as usize + 1) * height as usize);
    for row in pixels.chunks(width as usize).take(height as usize) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    // Bit depth 8, greyscale, deflate, adaptive filtering, no interlace.
    header.extend_from_slice(&[8, 0, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(
        &mut out,
        b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6),
    );
    chunk(&mut out, b"IEND", &[]);
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32fast::hash(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_png_has_its_signature_and_chunks() {
        let png = gray(3, 2, &[0, 128, 255, 1, 2, 3]);
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(&png[png.len() - 8..png.len() - 4], b"IEND");
    }
}
