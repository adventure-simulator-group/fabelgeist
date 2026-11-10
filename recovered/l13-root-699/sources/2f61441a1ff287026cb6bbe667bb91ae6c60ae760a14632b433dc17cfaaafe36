//! Where the garment sits on the body, before it is let go.
//!
//! A pattern places its own panels: every `PanelSpec` carries a translation
//! and a rotation, and those are what put a sleeve beside an arm and a back
//! panel behind a back. What the pattern cannot know is the body. It was
//! drawn for a set of measurements, and the mesh it is draped on is matched
//! to those by height alone -- so a garment can start a few centimetres too
//! low, facing the wrong way, or with a panel run through a limb. No amount
//! of simulation recovers from that: the first substep only makes it
//! permanent.
//!
//! This is the adjustment, and it is made in **body space** -- the solver's
//! own frame, metres, +Y up, the body standing with its feet at the origin
//! and +Z the way the pattern's front panels face. A [`GarmentPose`] is one
//! rigid [`Move`] for the whole garment, plus one per panel for the pieces
//! the whole-garment move cannot fix on its own.
//!
//! Rigid on purpose. Every rest length, bending weight and particle mass in a
//! `GarmentMesh` was measured from its placed positions, so a move that
//! preserves distances leaves all of them correct, and the garment can be
//! re-placed without being meshed again -- which is what lets a slider show
//! its answer at once rather than after a rebuild. A scale could not: it
//! would stretch the fabric's own rest shape, which is a change to the
//! pattern rather than to where the pattern is worn.

use std::ops::Range;

use fabelgeist_cloth::GarmentMesh;
use fabelgeist_cloth::garment::rotate_xyz;
use fabelgeist_math::Vec3;

use super::CM_TO_M;

/// A rigid move in body space: a turn about a pivot, then an offset.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Move {
    /// Metres.
    pub offset: Vec3,
    /// Degrees about the vertical axis -- the turn that puts a garment on
    /// back to front, and the one that takes it off again.
    pub yaw: f32,
    /// Degrees about the left-right axis.
    pub pitch: f32,
    /// Degrees about the front-back axis.
    pub roll: f32,
}

impl Move {
    pub const IDENTITY: Self = Self {
        offset: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
    };

    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }

    /// Apply the move to one point, turning it about `pivot`.
    pub fn apply(&self, point: Vec3, pivot: Vec3) -> Vec3 {
        // `rotate_xyz` is the order the pattern format itself uses -- X, then
        // Y, then Z -- reused rather than reimplemented, so that a panel
        // nudged here turns the same way as a panel placed by the pattern.
        rotate_xyz(point - pivot, Vec3::new(self.pitch, self.yaw, self.roll)) + pivot + self.offset
    }
}

/// One of the six numbers a [`Move`] is made of, as a control.
///
/// The point of naming them is that the interface can be a loop rather than
/// six near-identical blocks, and that the units live in one place: a shift
/// is read and written in centimetres, which is what a sewing pattern is
/// drawn in, while the solver keeps metres.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Sideways,
    Vertical,
    Depth,
    Yaw,
    Pitch,
    Roll,
}

impl Axis {
    pub const ALL: [Axis; 6] = [
        Axis::Sideways,
        Axis::Vertical,
        Axis::Depth,
        Axis::Yaw,
        Axis::Pitch,
        Axis::Roll,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Axis::Sideways => "Left / right",
            Axis::Vertical => "Up / down",
            Axis::Depth => "Front / back",
            Axis::Yaw => "Turn",
            Axis::Pitch => "Tip",
            Axis::Roll => "Lean",
        }
    }

    pub fn is_angle(self) -> bool {
        matches!(self, Axis::Yaw | Axis::Pitch | Axis::Roll)
    }

    pub fn unit(self) -> &'static str {
        if self.is_angle() { "°" } else { " cm" }
    }

    /// How far the control travels either side of zero.
    ///
    /// A turn goes all the way round, because facing the wrong way is a whole
    /// half-turn wrong. Tipping and leaning do not: past about a quarter turn
    /// a garment is not being placed on a body any more.
    pub fn limit(self) -> f32 {
        match self {
            Axis::Yaw => 180.0,
            Axis::Pitch | Axis::Roll => 45.0,
            _ => 40.0,
        }
    }

    pub fn step(self) -> f32 {
        if self.is_angle() { 1.0 } else { 0.5 }
    }

    /// Read the axis in the control's own units.
    pub fn get(self, moved: &Move) -> f32 {
        match self {
            Axis::Sideways => moved.offset.x / CM_TO_M,
            Axis::Vertical => moved.offset.y / CM_TO_M,
            Axis::Depth => moved.offset.z / CM_TO_M,
            Axis::Yaw => moved.yaw,
            Axis::Pitch => moved.pitch,
            Axis::Roll => moved.roll,
        }
    }

    /// Write the axis in the control's own units.
    pub fn set(self, moved: &mut Move, value: f32) {
        match self {
            Axis::Sideways => moved.offset.x = value * CM_TO_M,
            Axis::Vertical => moved.offset.y = value * CM_TO_M,
            Axis::Depth => moved.offset.z = value * CM_TO_M,
            Axis::Yaw => moved.yaw = value,
            Axis::Pitch => moved.pitch = value,
            Axis::Roll => moved.roll = value,
        }
    }
}

/// What a pose is applied against: the panels it can move, and the point each
/// of them turns about.
///
/// Taken from the mesh as it was built and never from the posed positions, so
/// that editing a pose is not a walk: moving a panel and moving it back
/// leaves it exactly where it started, and a pose read off one build means
/// the same thing on the next.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pivots {
    /// The whole garment's centre.
    pub garment: Vec3,
    /// Per panel, in the mesh's own panel order: which particles it owns, and
    /// its own centre.
    pub panels: Vec<(Range<usize>, Vec3)>,
}

impl Pivots {
    pub fn of(mesh: &GarmentMesh) -> Self {
        Self {
            garment: center(&mesh.positions),
            panels: (0..mesh.panel_count())
                .map(|panel| {
                    let range = mesh.panel_range(panel);
                    let pivot = center(&mesh.positions[range.clone()]);
                    (range, pivot)
                })
                .collect(),
        }
    }
}

/// The centre of a set of points' bounding box.
///
/// The box rather than the mean: a mean is pulled about by how finely a panel
/// happened to be meshed, so it moves when the resolution slider does -- and
/// a pivot that moves makes the same pose mean two different things on two
/// builds of the same pattern.
fn center(points: &[Vec3]) -> Vec3 {
    if points.is_empty() {
        return Vec3::default();
    }
    let mut lowest = points[0];
    let mut highest = points[0];
    for point in &points[1..] {
        lowest = lowest.min(*point);
        highest = highest.max(*point);
    }
    (lowest + highest) * 0.5
}

/// Where a garment has been put: one move for all of it, and one per panel.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GarmentPose {
    pub garment: Move,
    /// Indexed as the mesh's panels are. A short list -- or an empty one --
    /// leaves the rest of the panels where the pattern put them.
    pub panels: Vec<Move>,
}

impl GarmentPose {
    /// An untouched pose with a slot for every panel of a mesh.
    pub fn for_mesh(mesh: &GarmentMesh) -> Self {
        Self {
            garment: Move::IDENTITY,
            panels: vec![Move::IDENTITY; mesh.panel_count()],
        }
    }

    pub fn is_identity(&self) -> bool {
        self.garment.is_identity() && self.panels.iter().all(Move::is_identity)
    }

    pub fn panel(&self, index: usize) -> Move {
        self.panels.get(index).copied().unwrap_or(Move::IDENTITY)
    }

    pub fn panel_mut(&mut self, index: usize) -> &mut Move {
        if self.panels.len() <= index {
            self.panels.resize(index + 1, Move::IDENTITY);
        }
        &mut self.panels[index]
    }

    /// Place a mesh's particles: each panel's own move first, then the whole
    /// garment's over the top of it.
    ///
    /// `base` is the mesh's own positions -- where the *pattern* put the
    /// panels -- not wherever the garment happens to be now. Composing
    /// against the base is what keeps a pose a description of a place rather
    /// than of a journey to it.
    pub fn place(&self, base: &[Vec3], pivots: &Pivots) -> Vec<Vec3> {
        let mut placed = base.to_vec();

        for (index, (range, pivot)) in pivots.panels.iter().enumerate() {
            let nudge = self.panel(index);
            if nudge.is_identity() {
                continue;
            }
            let range = range.start.min(placed.len())..range.end.min(placed.len());
            for position in &mut placed[range] {
                *position = nudge.apply(*position, *pivot);
            }
        }

        if !self.garment.is_identity() {
            for position in &mut placed {
                *position = self.garment.apply(*position, pivots.garment);
            }
        }

        placed
    }
}
