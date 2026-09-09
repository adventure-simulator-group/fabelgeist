//! Linear-filterable 16-bit height in RG, occlusion in B. Mips average decoded values.
use crate::*;
/// Decode the normalized height; linear interpolation of RG preserves this value.
pub fn decode_height_ao(pixel: &[u8]) -> f32 {
    f32::from(u16::from_be_bytes([pixel[0], pixel[1]])) / f32::from(u16::MAX)
}
pub(crate) fn image(mut values: Vec<[f32; 2]>, size: u32) -> Image {
    assert_eq!(values.len(), (size * size) as usize);
    let mut bytes = Vec::new();
    let mut side = size;
    loop {
        for [height, ao] in &values {
            let encoded = (height.clamp(0.0, 1.0) * f32::from(u16::MAX)).round() as u16;
            let [high, low] = encoded.to_be_bytes();
            bytes.extend_from_slice(&[high, low, (ao.clamp(0.0, 1.0) * 255.0).round() as u8, 255]);
        }
        if side == 1 {
            break;
        }
        let mut next = Vec::with_capacity((side * side / 4) as usize);
        for y in 0..side / 2 {
            for x in 0..side / 2 {
                let mut average = [0.0; 2];
                for dy in 0..2 {
                    for dx in 0..2 {
                        let sample = values[((y * 2 + dy) * side + x * 2 + dx) as usize];
                        for channel in 0..2 {
                            average[channel] += sample[channel] * 0.25;
                        }
                    }
                }
                next.push(average);
            }
        }
        side /= 2;
        values = next;
    }
    let mut image =
        crate::image_rgba_mipped(bytes[..(size * size * 4) as usize].to_vec(), size, true);
    image.data = Some(bytes);
    image
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shallow_ramp_survives_encoding_and_mips_without_byte_carry_artifacts() {
        let ramp = (0..64).map(|i| [0.49 + i as f32 * 0.0001, 0.75]).collect();
        let image = image(ramp, 8);
        let bytes = image.data.unwrap();
        let heights: Vec<_> = bytes[..256].chunks_exact(4).map(decode_height_ao).collect();
        assert!(heights.windows(2).all(|h| h[1] > h[0]));
        assert!((decode_height_ao(&bytes[bytes.len() - 4..]) - 0.49315).abs() < 1.0 / 65535.0);
        assert_eq!(bytes[2], 191);
    }
}
