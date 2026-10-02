//! A transverse wheel with a finite chord receiving the grip.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WheelPommelParameters {
    pub diameter: Metres,
    pub thickness: Metres,
    pub face_diameter: Metres,
    /// Thickness at the outer rim before the bevel rises to either face.
    pub rim_thickness: Metres,
    /// Distance from the wheel center to its flat grip receiving plane.
    pub seat_height: Metres,
}
