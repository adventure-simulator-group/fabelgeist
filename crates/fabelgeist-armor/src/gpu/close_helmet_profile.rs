//! The sections a close helmet is fitted to: the fit words the device
//! measures on the wearer, and a typed form of them for callers that author
//! one.

use super::recipe::own_frame_words;

/// Outer sectional bounds in metres in the head frame, padding and plate
/// reserve included. Anatomy sets these bounds; the design sets projection,
/// ridge, lip and openings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CloseHelmetProfile {
    pub skull_half_width: f32,
    pub temple_half_width: f32,
    pub skull_front: f32,
    pub skull_back: f32,
    pub jaw_half_width: f32,
    pub jaw_front: f32,
    pub submental_front: f32,
    pub neck_half_width: f32,
    pub throat_front: f32,
    pub nape_back: f32,
    pub nape_waist: f32,
}

impl CloseHelmetProfile {
    /// The fit buffer [`super::record_close_helmet`] reads for a head of
    /// `half_extents`: the helmet's own frame, then these sections in field
    /// order.
    pub fn fit_words(&self, half_extents: [f32; 3]) -> Vec<f32> {
        let mut words = own_frame_words(half_extents);
        words.extend([
            self.skull_half_width,
            self.temple_half_width,
            self.skull_front,
            self.skull_back,
            self.jaw_half_width,
            self.jaw_front,
            self.submental_front,
            self.neck_half_width,
            self.throat_front,
            self.nape_back,
            self.nape_waist,
        ]);
        words
    }
}
