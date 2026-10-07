//! The USC-HairSalon `.data` format: a `u32` strand count, a `u32` point
//! count, a `u16` point count per strand, then every point as three `f32`,
//! little-endian. DiffLocks reads and writes it (`utils/strand_util.py`).

use anyhow::{Result, bail};
use fabelgeist_math::Vec3;

use crate::strands::{Attachment, Strands, resample_into};

pub fn write(strands: &Strands) -> Result<Vec<u8>> {
    if strands.points > u16::MAX as usize {
        bail!("{} points a strand do not fit the format", strands.points);
    }
    let mut out = Vec::with_capacity(8 + strands.len() * (2 + strands.points * 12));
    out.extend_from_slice(&(strands.len() as u32).to_le_bytes());
    out.extend_from_slice(&(strands.positions.len() as u32).to_le_bytes());
    for _ in 0..strands.len() {
        out.extend_from_slice(&(strands.points as u16).to_le_bytes());
    }
    for p in &strands.positions {
        for v in p.to_array() {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    Ok(out)
}

/// Read a `.data` file, resampling each strand to `points` points. Strands
/// of fewer than two points are skipped.
pub fn read(bytes: &[u8], points: usize) -> Result<Strands> {
    let word = |at: usize| -> Result<u32> {
        let b = bytes
            .get(at..at + 4)
            .ok_or_else(|| anyhow::anyhow!("truncated"))?;
        Ok(u32::from_le_bytes(b.try_into()?))
    };
    let count = word(0)? as usize;
    let mut cursor = 8;
    let mut sizes = Vec::with_capacity(count);
    for _ in 0..count {
        let b = bytes
            .get(cursor..cursor + 2)
            .ok_or_else(|| anyhow::anyhow!("truncated"))?;
        sizes.push(u16::from_le_bytes([b[0], b[1]]) as usize);
        cursor += 2;
    }
    let mut strands = Strands::new(points);
    let mut buffer = vec![Vec3::default(); points];
    let mut lengths = Vec::new();
    for (s, &size) in sizes.iter().enumerate() {
        let mut strand = Vec::with_capacity(size);
        for _ in 0..size {
            let b = bytes
                .get(cursor..cursor + 12)
                .ok_or_else(|| anyhow::anyhow!("truncated"))?;
            let f = |k: usize| f32::from_le_bytes(b[k * 4..k * 4 + 4].try_into().unwrap());
            strand.push(Vec3::new(f(0), f(1), f(2)));
            cursor += 12;
        }
        if size < 2 {
            continue;
        }
        resample_into(&strand, &mut lengths, &mut buffer);
        let normal = (strand[1] - strand[0]).normalize();
        strands.push(
            &buffer,
            Attachment {
                normal,
                id: s as u32,
                group: -1,
                radius: 35e-6,
                ..Default::default()
            },
        );
    }
    Ok(strands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let mut strands = Strands::new(4);
        for s in 0..3 {
            let positions: Vec<Vec3> = (0..4).map(|j| Vec3::new(s as f32, j as f32, 0.0)).collect();
            strands.push(&positions, Attachment::default());
        }
        let back = read(&write(&strands).unwrap(), 4).unwrap();
        assert_eq!(back.len(), 3);
        for (a, b) in strands.positions.iter().zip(&back.positions) {
            assert!((*a - *b).length() < 1e-6);
        }
    }
}
