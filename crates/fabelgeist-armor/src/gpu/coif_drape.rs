//! The drape a mail coif hangs by: the fit words the device measures on the
//! wearer, and a typed form of them for callers that author one.

use super::close_helmet::FIT_PROFILE_WORD;
use super::recipe::own_frame_words;

/// Horizontal body sections each flap's drape is measured at.
pub const COIF_DRAPE_SECTIONS: usize = 5;

/// Floats of a coif drape in a fit buffer: the neck boundary's seven, then
/// each flap's sections as height, centre and edge depth, front flap first.
pub const COIF_DRAPE_WORDS: usize = 7 + 2 * 3 * COIF_DRAPE_SECTIONS;

/// The lower neck boundary, along the neck/shoulder junction. Heights and
/// depths are metres in the head frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoifNeckDrape {
    pub front_height: f32,
    pub side_height: f32,
    pub back_height: f32,
    pub half_width: f32,
    pub center_depth: f32,
    pub front_depth: f32,
    pub back_depth: f32,
}

/// One horizontal body section under a flap, in metres in the head frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoifDrapeSection {
    pub height: f32,
    pub center_depth: f32,
    pub edge_depth: f32,
}

/// A flap's sections, ordered from its lowest hem toward the neck.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoifFlapDrape {
    pub sections: [CoifDrapeSection; COIF_DRAPE_SECTIONS],
}

/// A coif's whole drape: its neck boundary and both flaps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoifDrapeProfile {
    pub neck: CoifNeckDrape,
    pub front: CoifFlapDrape,
    pub back: CoifFlapDrape,
}

impl CoifDrapeProfile {
    /// The fit buffer [`super::record_coif`] reads for a head of
    /// `half_extents`: the coif's own frame, then this drape.
    pub fn fit_words(&self, half_extents: [f32; 3]) -> Vec<f32> {
        let mut words = own_frame_words(half_extents);
        debug_assert_eq!(words.len(), FIT_PROFILE_WORD);
        let n = self.neck;
        words.extend([
            n.front_height,
            n.side_height,
            n.back_height,
            n.half_width,
            n.center_depth,
            n.front_depth,
            n.back_depth,
        ]);
        for flap in [&self.front, &self.back] {
            for s in &flap.sections {
                words.extend([s.height, s.center_depth, s.edge_depth]);
            }
        }
        words
    }
}
