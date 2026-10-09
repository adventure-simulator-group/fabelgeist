//! Authored browser actions; Bevy retains the corresponding camera operations.
pub(super) const CONTROLS: [MapControl; 6] = [
    MapControl {
        action: MapControlAction::ZoomIn,
        label: "Zoom in",
        glyph: "+",
    },
    MapControl {
        action: MapControlAction::ZoomOut,
        label: "Zoom out",
        glyph: "−",
    },
    MapControl {
        action: MapControlAction::RotateLeft,
        label: "Rotate left",
        glyph: "↶",
    },
    MapControl {
        action: MapControlAction::RotateRight,
        label: "Rotate right",
        glyph: "↷",
    },
    MapControl {
        action: MapControlAction::Reset,
        label: "Reset map view",
        glyph: "⌂",
    },
    MapControl {
        action: MapControlAction::FrameRoute,
        label: "Frame route",
        glyph: "↔",
    },
];

pub(super) struct MapControl {
    pub action: MapControlAction,
    pub label: &'static str,
    pub glyph: &'static str,
}

pub(super) enum MapControlAction {
    ZoomIn,
    ZoomOut,
    RotateLeft,
    RotateRight,
    Reset,
    FrameRoute,
}

impl MapControlAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ZoomIn => "zoom-in",
            Self::ZoomOut => "zoom-out",
            Self::RotateLeft => "rotate-left",
            Self::RotateRight => "rotate-right",
            Self::Reset => "reset",
            Self::FrameRoute => "frame-route",
        }
    }
}
