//! Where the tactical client's sounds go.
//!
//! Gameplay systems decide what plays, where, how loud and at what speed;
//! this module hands those decisions to the platform. In the browser that is
//! Web Audio: `window.sfx*` (`static/sfx.js` beside `tactical.html`) decodes
//! the files with the browser's own decoder and one AudioWorklet mixes every
//! voice on the audio thread, so no decoding or mixing runs on the frame
//! loop. Native builds, used for development and profiling, are silent.
//!
//! Asset paths are relative to the Bevy asset root, as an `AssetServer` would
//! take them.

use bevy::math::Vec3;

#[cfg(not(target_family = "wasm"))]
#[path = "sound_output/silent.rs"]
mod backend;
#[cfg(target_family = "wasm")]
#[path = "sound_output/web.rs"]
mod backend;

/// Distance between the two ears of the fixed spatial listener, metres.
///
/// The client has never placed a listener in the world, so spatial sounds
/// were heard by Bevy's default `SpatialListener`: ears four metres apart,
/// straddling the world origin on the X axis. This keeps that listener.
const LISTENER_EAR_GAP_METRES: f32 = 4.0;

/// Linear gain applied to a playing sound: `0` (the default) silent, `1` as
/// recorded.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct LinearVolume(f32);

/// Per-ear linear gains of one spatial one-shot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StereoGain {
    left: f32,
    right: f32,
}

/// A recorded sound with interchangeable takes, stored as
/// `<directory>/<stem><take>.ogg` for each take index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SoundFamily {
    directory: &'static str,
    stem: &'static str,
}

/// Playback speed: `1` as recorded; above it plays faster and higher.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlaybackSpeed(f32);

impl LinearVolume {
    pub(crate) fn new(gain: f32) -> Self {
        Self(gain)
    }

    pub(crate) fn linear(self) -> f32 {
        self.0
    }
}

impl StereoGain {
    /// Gains for a sound at `emitter` (scene metres) heard by the fixed
    /// listener at the world origin.
    ///
    /// This is rodio's spatial law as Bevy audio applied it: each ear falls
    /// off with the inverse square of its distance (capped at full level
    /// inside one metre), weighted by the difference between the two ear
    /// distances.
    pub(crate) fn at(emitter: Vec3, volume: LinearVolume) -> Self {
        let half_gap = Vec3::X * (LISTENER_EAR_GAP_METRES / 2.0);
        let left_squared = emitter.distance_squared(-half_gap);
        let right_squared = emitter.distance_squared(half_gap);
        let difference = (left_squared.sqrt() - right_squared.sqrt()) / LISTENER_EAR_GAP_METRES;
        let ear = |difference: f32, distance_squared: f32| {
            let balance = ((difference + 1.0) / 4.0 + 0.5).min(1.0);
            balance * distance_squared.recip().min(1.0) * volume.0
        };
        Self {
            left: ear(difference, left_squared),
            right: ear(-difference, right_squared),
        }
    }
}

impl SoundFamily {
    /// Every family ships this many takes.
    pub(crate) const TAKES: usize = 3;

    pub(crate) const fn new(directory: &'static str, stem: &'static str) -> Self {
        Self { directory, stem }
    }

    /// The family's name, which also keys its deterministic take selection.
    pub(crate) fn stem(self) -> &'static str {
        self.stem
    }

    /// Asset path of take `index`, below [`Self::TAKES`].
    pub(crate) fn take(self, index: usize) -> String {
        format!("{}/{}{index}.ogg", self.directory, self.stem)
    }

    /// [`preload`] every take.
    pub(crate) fn preload(self) {
        for index in 0..Self::TAKES {
            preload(&self.take(index));
        }
    }
}

impl PlaybackSpeed {
    pub(crate) const RECORDED: Self = Self(1.0);

    pub(crate) fn new(speed: f32) -> Self {
        Self(speed)
    }
}

impl std::ops::Mul for PlaybackSpeed {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}

/// Fetch and decode a one-shot ahead of its first play, so that play is not
/// lost to decoding.
pub(crate) fn preload(asset_path: &str) {
    backend::preload(asset_path);
}

/// Play `asset_path` once as a spatial sound: its channels mixed to one and
/// sent to each ear at that ear's gain.
pub(crate) fn play_spatial(asset_path: &str, gain: StereoGain, speed: PlaybackSpeed) {
    backend::play_spatial(asset_path, gain.left, gain.right, speed.0);
}

/// Hold the endless, non-spatial loop of `asset_path` at `volume`. A loop
/// starts the first time it is given a volume above silence and keeps its
/// place in the recording from then on.
pub(crate) fn set_loop_volume(asset_path: &str, volume: LinearVolume) {
    backend::set_loop_volume(asset_path, volume.0);
}
