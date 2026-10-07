//! DiffLocks' dataset as it is on disk: reading its samples, and the layout
//! [`crate::dataset`] writes ours in.
//!
//! After `download_dataset.sh` and `uncompress_data.py`, and as its
//! `data_loader/dataloader.py` reads it, the dataset is
//!
//! ```text
//! <root>/body_data/scalp.ply           the SMPL-X scalp, UVs as `s t`
//! <root>/body_data/smplx_base.ply      the mean SMPL-X body
//! <root>/generated_hairstyles/base_<X>_idx_<Y>/
//!     full_strands.npz                 positions (N x 256 x 3), root_uv (N x 2), root_normal (N x 3)
//!     guide_strands.npz, interpolated_strands.npz
//!     density.png                      the scalp's strand density, channel 0 over 255
//!     rgb.png                          the render, 768 x 768
//!     cam.npz                          RT (4 x 4, camera from world), K (3 x 3, pixels)
//!     metadata.json                    the draw: mirror_hair, the material's parameters, ...
//!     numpy_state.npz                  written last: the sample is complete
//! ```
//!
//! for base style `X` and sample `Y`. Positions are metres in SMPL-X's frame
//! -- `y` up, the face towards `+z`, like the [`Head`]'s, but with the head
//! 30 cm higher. Where `mirror_hair` is set the strands were mirrored and
//! their UVs were not: the loader flips `u`, and so does [`load_sample`].

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use fabelgeist_math::Vec3;

use crate::export::npz;
use crate::head::Head;
use crate::rng;
use crate::strands::{Attachment, Strands};
use crate::surface::Surface;

/// A sample's strands, as read.
pub struct LoadedSample {
    /// The sample's directory name (`base_12_idx_3456`), or the file's.
    pub name: String,
    pub strands: Strands,
    pub metadata: Option<serde_json::Value>,
}

/// The strands of a `full_strands.npz` (or any file with its `positions`,
/// `root_uv` and `root_normal`), each `radius` at the root.
pub fn load_strands(path: &Path, radius: f32) -> Result<Strands> {
    let arrays = npz::read(path)?;
    let get = |key: &str| {
        arrays
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, a)| a)
            .with_context(|| format!("{} has no `{key}`", path.display()))
    };
    let positions = get("positions")?;
    let [count, points, 3] = positions.shape[..] else {
        bail!(
            "positions are {:?}, not strands x points x 3",
            positions.shape
        );
    };
    if points < 2 {
        bail!("strands of {points} points");
    }
    let uv = get("root_uv").ok();
    let normal = get("root_normal").ok();
    let mut strands = Strands::with_capacity(points, count);
    let mut strand = vec![Vec3::default(); points];
    for s in 0..count {
        for (j, p) in strand.iter_mut().enumerate() {
            let at = (s * points + j) * 3;
            *p = Vec3::new(
                positions.data[at],
                positions.data[at + 1],
                positions.data[at + 2],
            );
        }
        let root_normal = normal
            .filter(|n| n.data.len() >= (s + 1) * 3)
            .map(|n| Vec3::new(n.data[s * 3], n.data[s * 3 + 1], n.data[s * 3 + 2]))
            .unwrap_or_else(|| (strand[1] - strand[0]).normalize());
        strands.push(
            &strand,
            Attachment {
                uv: uv
                    .filter(|u| u.data.len() >= (s + 1) * 2)
                    .map_or([0.0; 2], |u| [u.data[s * 2], u.data[s * 2 + 1]]),
                normal: root_normal,
                id: rng::hash(s as u32, 0xD1FF),
                group: -1,
                radius,
            },
        );
    }
    Ok(strands)
}

/// A sample: a sample directory (its `full_strands.npz`, and its
/// `metadata.json` if there is one) or a strand file itself.
pub fn load_sample(path: &Path, radius: f32) -> Result<LoadedSample> {
    let (dir, file) = if path.is_dir() {
        (path.to_path_buf(), path.join("full_strands.npz"))
    } else {
        (
            path.parent().map(Path::to_path_buf).unwrap_or_default(),
            path.to_path_buf(),
        )
    };
    let mut strands = load_strands(&file, radius)?;
    let metadata: Option<serde_json::Value> = std::fs::read(dir.join("metadata.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    if metadata
        .as_ref()
        .and_then(|m| m.get("mirror_hair"))
        .and_then(|m| m.as_bool())
        == Some(true)
    {
        for uv in &mut strands.root_uv {
            uv[0] = 1.0 - uv[0];
        }
    }
    let name = if path.is_dir() { &dir } else { path }
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(LoadedSample {
        name,
        strands,
        metadata,
    })
}

/// The dataset's root above a sample: the nearest directory up from it
/// with a `body_data` in it.
pub fn dataset_root(sample: &Path) -> Option<PathBuf> {
    sample
        .ancestors()
        .find(|dir| dir.join("body_data").is_dir())
        .map(Path::to_path_buf)
}

/// The dataset's scalp, and the body if it is there.
pub struct Body {
    pub scalp: Surface,
    pub body: Option<Surface>,
}

pub fn load_body(root: &Path) -> Result<Body> {
    let data = root.join("body_data");
    Ok(Body {
        scalp: Surface::load_ply(&data.join("scalp.ply"))?,
        body: Surface::load_ply(&data.join("smplx_base.ply")).ok(),
    })
}

/// The shift from DiffLocks' frame onto `head`'s: the top of the scalp (or
/// of the strands' roots, `points`) onto the top of the head, centred.
pub fn alignment(points: impl IntoIterator<Item = Vec3>, head: &Head) -> Vec3 {
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for p in points {
        lo = lo.min(p);
        hi = hi.max(p);
    }
    if !lo.is_finite() || !hi.is_finite() {
        return Vec3::default();
    }
    let top = Vec3::new(0.5 * (lo.x + hi.x), hi.y, 0.5 * (lo.z + hi.z));
    head.crown() - top
}

/// Move strands by `offset`.
pub fn shift(strands: &mut Strands, offset: Vec3) {
    for p in &mut strands.positions {
        *p += offset;
    }
}

/// Where a sample of base style `base`, index `index`, goes in a dataset
/// rooted at `root`.
pub fn sample_dir(root: &Path, base: usize, index: u64) -> PathBuf {
    root.join("generated_hairstyles")
        .join(format!("base_{base}_idx_{index}"))
}

/// A camera as `cam.npz` holds it: `RT` (world to camera, OpenCV's axes --
/// `x` right, `y` down, `z` ahead) and `K` (pixels, for a `size` square
/// image), from an eye, a target and a vertical field of view.
pub fn camera_arrays(
    eye: [f32; 3],
    target: [f32; 3],
    fov_y: f32,
    size: u32,
) -> ([f32; 16], [f32; 9]) {
    let eye = Vec3::from_array(eye);
    let ahead = (Vec3::from_array(target) - eye).normalize();
    let right = crate::strands::safe_normalize(ahead.cross(Vec3::up()), Vec3::right());
    let down = ahead.cross(right);
    let rows = [right, down, ahead];
    let mut rt = [0.0; 16];
    for (r, axis) in rows.iter().enumerate() {
        rt[r * 4] = axis.x;
        rt[r * 4 + 1] = axis.y;
        rt[r * 4 + 2] = axis.z;
        rt[r * 4 + 3] = -axis.dot(eye);
    }
    rt[15] = 1.0;
    let focal = 0.5 * size as f32 / (0.5 * fov_y).tan();
    let c = 0.5 * size as f32;
    let k = [focal, 0.0, c, 0.0, focal, c, 0.0, 0.0, 1.0];
    (rt, k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_written_sample_reads_back() {
        let mut strands = Strands::new(4);
        for i in 0..3 {
            let x = i as f32 * 0.01;
            let positions: Vec<Vec3> = (0..4)
                .map(|j| Vec3::new(x, 0.3 - j as f32 * 0.01, 0.0))
                .collect();
            strands.push(
                &positions,
                Attachment {
                    uv: [0.25, 0.5],
                    normal: Vec3::up(),
                    ..Default::default()
                },
            );
        }
        let dir = std::env::temp_dir().join(format!("fabelgeist-difflocks-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        crate::export::write_difflocks_npz(&dir.join("full_strands.npz"), &strands, 8).unwrap();
        std::fs::write(dir.join("metadata.json"), r#"{"mirror_hair": true}"#).unwrap();
        let sample = load_sample(&dir, 3e-5).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(sample.strands.len(), 3);
        assert_eq!(sample.strands.points, 8);
        assert!((sample.strands.root_uv[0][0] - 0.75).abs() < 1e-6);
        assert!((sample.strands.root(2).x - 0.02).abs() < 1e-5);
    }

    #[test]
    fn the_camera_looks_down_its_z() {
        let (rt, k) = camera_arrays([0.0, 0.0, 1.0], [0.0; 3], 0.5, 768);
        // The target is straight ahead: at the image's centre.
        let z = rt[8] * 0.0 + rt[9] * 0.0 + rt[10] * 0.0 + rt[11];
        assert!((z - 1.0).abs() < 1e-5);
        assert_eq!(k[2], 384.0);
    }
}
