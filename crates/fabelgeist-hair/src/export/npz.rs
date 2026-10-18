//! NumPy `.npy` arrays, and `.npz` archives of them (an uncompressed zip,
//! which `numpy.load` reads as well as a deflated one).
//!
//! Reading takes what NumPy writes: stored or deflated entries
//! (`np.savez`, `np.savez_compressed`), the ZIP64 sizes it gives every
//! entry, and little-endian `float16`, `float32` and `float64` arrays in C
//! order -- read as `f32`.

use std::io::Write;
use std::path::Path;

use crate::export::error::{HairDataError, HairDataFormat, Result};

/// A little-endian `float32` array in `.npy` format, version 1.0.
pub fn npy(shape: &[usize], data: &[f32]) -> Result<Vec<u8>> {
    let count: usize = shape.iter().product();
    if count != data.len() {
        return Err(HairDataError::ShapeMismatch {
            shape: shape.to_vec(),
            actual_values: data.len(),
        });
    }
    let dims = match shape {
        [one] => format!("({one},)"),
        _ => format!(
            "({})",
            shape
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    let mut header = format!("{{'descr': '<f4', 'fortran_order': False, 'shape': {dims}, }}");
    // Magic, version and length take ten bytes; the header is padded with
    // spaces to a multiple of 64 and ends in a newline.
    let total = (10 + header.len() + 1).next_multiple_of(64);
    header.push_str(&" ".repeat(total - 10 - header.len() - 1));
    header.push('\n');
    let mut out = Vec::with_capacity(total + data.len() * 4);
    out.extend_from_slice(b"\x93NUMPY\x01\x00");
    out.extend_from_slice(&(header.len() as u16).to_le_bytes());
    out.extend_from_slice(header.as_bytes());
    for v in data {
        out.extend_from_slice(&v.to_le_bytes());
    }
    Ok(out)
}

/// Named arrays into a stored (uncompressed) zip: `name.npy` each.
pub fn archive(arrays: &[(&str, &[usize], &[f32])]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, shape, data) in arrays {
        let body = npy(shape, data)?;
        let file = format!("{name}.npy");
        if body.len() >= u32::MAX as usize || out.len() >= u32::MAX as usize {
            return Err(HairDataError::ZipEntryTooLarge { name: file });
        }
        let crc = crc32fast::hash(&body);
        let offset = out.len() as u32;
        let header = |signature: u32, central: bool| {
            let mut h = Vec::new();
            h.extend_from_slice(&signature.to_le_bytes());
            if central {
                h.extend_from_slice(&20u16.to_le_bytes()); // made by
            }
            h.extend_from_slice(&20u16.to_le_bytes()); // needed to extract
            h.extend_from_slice(&0u16.to_le_bytes()); // flags
            h.extend_from_slice(&0u16.to_le_bytes()); // stored
            h.extend_from_slice(&0u16.to_le_bytes()); // time
            h.extend_from_slice(&0x21u16.to_le_bytes()); // date: 1980-01-01
            h.extend_from_slice(&crc.to_le_bytes());
            h.extend_from_slice(&(body.len() as u32).to_le_bytes());
            h.extend_from_slice(&(body.len() as u32).to_le_bytes());
            h.extend_from_slice(&(file.len() as u16).to_le_bytes());
            h.extend_from_slice(&0u16.to_le_bytes()); // extra
            if central {
                h.extend_from_slice(&0u16.to_le_bytes()); // comment
                h.extend_from_slice(&0u16.to_le_bytes()); // disk
                h.extend_from_slice(&0u16.to_le_bytes()); // internal attributes
                h.extend_from_slice(&0u32.to_le_bytes()); // external attributes
                h.extend_from_slice(&offset.to_le_bytes());
            }
            h.extend_from_slice(file.as_bytes());
            h
        };
        out.extend_from_slice(&header(0x0403_4b50, false));
        out.extend_from_slice(&body);
        central.extend_from_slice(&header(0x0201_4b50, true));
    }
    let central_offset = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(arrays.len() as u16).to_le_bytes());
    out.extend_from_slice(&(arrays.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    Ok(out)
}

/// An array read from a `.npy`: its shape, and its values as `f32`.
#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
}

/// Parse a `.npy` file.
pub fn parse_npy(bytes: &[u8]) -> Result<Array> {
    if bytes.len() < 10 || &bytes[..6] != b"\x93NUMPY" {
        return Err(HairDataError::InvalidSignature {
            format: HairDataFormat::Npy,
        });
    }
    let (length, start) = match bytes[6] {
        1 => (u16::from_le_bytes([bytes[8], bytes[9]]) as usize, 10),
        2 | 3 => (
            u32::from_le_bytes(
                bytes
                    .get(8..12)
                    .ok_or(HairDataError::Truncated {
                        format: HairDataFormat::Npy,
                    })?
                    .try_into()?,
            ) as usize,
            12,
        ),
        version => return Err(HairDataError::NpyVersion { version }),
    };
    let header = std::str::from_utf8(bytes.get(start..start + length).ok_or_else(|| {
        HairDataError::NpyField {
            name: "header".into(),
        }
    })?)?;
    let field = |key: &str| -> Result<&str> {
        let at = header
            .find(&format!("'{key}'"))
            .ok_or_else(|| HairDataError::NpyField { name: key.into() })?;
        let rest = &header[at + key.len() + 2..];
        Ok(rest.trim_start_matches([':', ' ']))
    };
    let descr = field("descr")?;
    let descr = descr
        .strip_prefix('\'')
        .and_then(|rest| rest.split_once('\'').map(|(descr, _)| descr))
        .ok_or_else(|| HairDataError::NpyField {
            name: "descr".into(),
        })?;
    if field("fortran_order")?.starts_with("True") {
        return Err(HairDataError::FortranOrder);
    }
    let shape_text = field("shape")?;
    let shape_text =
        &shape_text[1..shape_text
            .find(')')
            .ok_or_else(|| HairDataError::NpyField {
                name: "shape".into(),
            })?];
    let shape: Vec<usize> = shape_text
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<usize>())
        .collect::<std::result::Result<_, _>>()?;
    let count: usize = shape.iter().product();
    let body = &bytes[start + length..];
    let data: Vec<f32> = match descr {
        "<f4" | "=f4" => body
            .as_chunks::<4>()
            .0
            .iter()
            .take(count)
            .map(|c| f32::from_le_bytes(*c))
            .collect(),
        "<f8" | "=f8" => body
            .as_chunks::<8>()
            .0
            .iter()
            .take(count)
            .map(|c| f64::from_le_bytes(*c) as f32)
            .collect(),
        "<f2" | "=f2" => body
            .as_chunks::<2>()
            .0
            .iter()
            .take(count)
            .map(|c| half_to_f32(u16::from_le_bytes(*c)))
            .collect(),
        other => return Err(HairDataError::NpyDtype { name: other.into() }),
    };
    if data.len() != count {
        return Err(HairDataError::ShapeMismatch {
            shape,
            actual_values: data.len(),
        });
    }
    Ok(Array { shape, data })
}

fn half_to_f32(h: u16) -> f32 {
    let sign = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exponent = ((h >> 10) & 0x1F) as i32;
    let mantissa = (h & 0x3FF) as f32;
    sign * match exponent {
        0 => mantissa * 2f32.powi(-24),
        31 => {
            if mantissa == 0.0 {
                f32::INFINITY
            } else {
                f32::NAN
            }
        }
        e => (1.0 + mantissa / 1024.0) * 2f32.powi(e - 15),
    }
}

struct ArchiveBytes<'a>(&'a [u8]);
impl ArchiveBytes<'_> {
    fn u16_at(&self, at: usize) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.0
                .get(at..at + 2)
                .ok_or(HairDataError::Truncated {
                    format: HairDataFormat::Npz,
                })?
                .try_into()?,
        ))
    }
    fn u32_at(&self, at: usize) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.0
                .get(at..at + 4)
                .ok_or(HairDataError::Truncated {
                    format: HairDataFormat::Npz,
                })?
                .try_into()?,
        ))
    }
    fn u64_at(&self, at: usize) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.0
                .get(at..at + 8)
                .ok_or(HairDataError::Truncated {
                    format: HairDataFormat::Npz,
                })?
                .try_into()?,
        ))
    }
}

/// The arrays in an `.npz`, by name (without `.npy`).
pub fn read_archive(bytes: &[u8]) -> Result<Vec<(String, Array)>> {
    let archive = ArchiveBytes(bytes);
    // The end of the central directory, and ZIP64's if the sizes overflow.
    let end = (0..bytes.len().saturating_sub(21))
        .rev()
        .find(|&i| bytes[i..i + 4] == 0x0605_4b50u32.to_le_bytes())
        .ok_or(HairDataError::InvalidSignature {
            format: HairDataFormat::Npz,
        })?;
    let mut entries = archive.u16_at(end + 10)? as u64;
    let mut directory = archive.u32_at(end + 16)? as u64;
    if (entries == 0xFFFF || directory == 0xFFFF_FFFF) && end >= 20 {
        let locator = end - 20;
        if archive.u32_at(locator)? == 0x0706_4b50 {
            let zip64 = archive.u64_at(locator + 8)? as usize;
            entries = archive.u64_at(zip64 + 32)?;
            directory = archive.u64_at(zip64 + 48)?;
        }
    }
    let mut at = directory as usize;
    let mut out = Vec::new();
    for _ in 0..entries {
        if archive.u32_at(at)? != 0x0201_4b50 {
            return Err(HairDataError::ZipDirectory);
        }
        let method = archive.u16_at(at + 10)?;
        let mut compressed = archive.u32_at(at + 20)? as u64;
        let mut size = archive.u32_at(at + 24)? as u64;
        let name_length = archive.u16_at(at + 28)? as usize;
        let extra_length = archive.u16_at(at + 30)? as usize;
        let comment_length = archive.u16_at(at + 32)? as usize;
        let mut offset = archive.u32_at(at + 42)? as u64;
        let name = std::str::from_utf8(bytes.get(at + 46..at + 46 + name_length).ok_or(
            HairDataError::Truncated {
                format: HairDataFormat::Npz,
            },
        )?)?
        .to_string();
        // ZIP64: the fields that overflowed, in this order.
        let mut extra = at + 46 + name_length;
        let extra_end = extra + extra_length;
        while extra + 4 <= extra_end {
            let id = archive.u16_at(extra)?;
            let length = archive.u16_at(extra + 2)? as usize;
            if id == 0x0001 {
                let mut field = extra + 4;
                if size == 0xFFFF_FFFF {
                    size = archive.u64_at(field)?;
                    field += 8;
                }
                if compressed == 0xFFFF_FFFF {
                    compressed = archive.u64_at(field)?;
                    field += 8;
                }
                if offset == 0xFFFF_FFFF {
                    offset = archive.u64_at(field)?;
                }
            }
            extra += 4 + length;
        }
        at = extra_end + comment_length;

        let local = offset as usize;
        if archive.u32_at(local)? != 0x0403_4b50 {
            return Err(HairDataError::ZipLocalHeader { name });
        }
        let start = local
            + 30
            + archive.u16_at(local + 26)? as usize
            + archive.u16_at(local + 28)? as usize;
        let raw =
            bytes
                .get(start..start + compressed as usize)
                .ok_or(HairDataError::Truncated {
                    format: HairDataFormat::Npz,
                })?;
        let body = match method {
            0 => raw.to_vec(),
            8 => miniz_oxide::inflate::decompress_to_vec_with_limit(raw, size as usize).map_err(
                |details| HairDataError::Deflate {
                    name: name.clone(),
                    details,
                },
            )?,
            method => return Err(HairDataError::ZipCompression { name, method }),
        };
        let key = name.strip_suffix(".npy").unwrap_or(&name).to_string();
        out.push((
            key,
            parse_npy(&body).map_err(|source| HairDataError::ArchiveEntry {
                name,
                source: Box::new(source),
            })?,
        ));
    }
    Ok(out)
}

/// Read the arrays in an `.npz` file.
pub fn read(path: &Path) -> Result<Vec<(String, Array)>> {
    let bytes = std::fs::read(path).map_err(|source| HairDataError::FileIo {
        path: path.to_owned(),
        source,
    })?;
    read_archive(&bytes).map_err(|source| HairDataError::File {
        path: path.to_owned(),
        source: Box::new(source),
    })
}

/// Write named arrays to an `.npz` file.
pub fn write(path: &Path, arrays: &[(&str, &[usize], &[f32])]) -> Result<()> {
    let bytes = archive(arrays)?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npy_headers_are_aligned_and_describe_the_shape() {
        let bytes = npy(&[2, 3], &[0.0; 6]).unwrap();
        let length = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
        assert_eq!((10 + length) % 64, 0);
        let header = std::str::from_utf8(&bytes[10..10 + length]).unwrap();
        assert!(header.contains("'shape': (2, 3)"));
        assert!(header.ends_with('\n'));
        assert_eq!(bytes.len(), 10 + length + 24);
    }

    #[test]
    fn an_archive_reads_back() {
        let bytes = archive(&[("a", &[1], &[1.0]), ("b", &[2, 1], &[2.0, -3.5])]).unwrap();
        let arrays = read_archive(&bytes).unwrap();
        assert_eq!(arrays.len(), 2);
        assert_eq!(arrays[1].0, "b");
        assert_eq!(arrays[1].1.shape, vec![2, 1]);
        assert_eq!(arrays[1].1.data, vec![2.0, -3.5]);
        assert_eq!(half_to_f32(0x3C00), 1.0);
        assert_eq!(half_to_f32(0xC000), -2.0);
    }

    #[test]
    fn an_archive_ends_in_its_directory() {
        let bytes = archive(&[("a", &[1], &[1.0]), ("b", &[2], &[2.0, 3.0])]).unwrap();
        let end = &bytes[bytes.len() - 22..];
        assert_eq!(&end[..4], &0x0605_4b50u32.to_le_bytes());
        assert_eq!(u16::from_le_bytes([end[10], end[11]]), 2);
    }
}
