use crate::camera::authored_metres;
use super::*;

pub(in crate::tactical_scene_viewer) const PLANT_LOD_ISOLATED_VIEWS: [CaptureViewSpec; 12] =
    isolated();

const fn isolated() -> [CaptureViewSpec; 12] {
    let mut views = PLANT_LOD_REVIEW_VIEWS;
    let mut index = 0;
    while index < views.len() {
        views[index] = views[index]
            .suppress_grass()
            .suppress_understory()
            .hide_obstacles();
        views[index].suppress_leaves = true;
        views[index].vista_visible = false;
        views[index].label = "Isolated production botanical LOD diagnostic";
        index += 1;
    }
    views
}

pub(in crate::tactical_scene_viewer) const PLANT_LOD_REVIEW_VIEWS: [CaptureViewSpec; 12] = [
    v!(
        "warmup",
        "Production vegetation warmup",
        CapturePose::Ground,
        55.0,
        100
    )
    .warmup()
    .vista(),
    v!(
        "lod-contact",
        "Highest botanical tier",
        CapturePose::PlantLod { distance: authored_metres(0.45) },
        40.0,
        100
    )
    .settled_readback_pair(),
    v!(
        "lod-high",
        "Before high transition",
        CapturePose::PlantLod { distance: authored_metres(2.15) },
        40.0,
        100
    )
    .settled_readback_pair(),
    v!(
        "lod-high-blend",
        "High and medium crossfade",
        CapturePose::PlantLod { distance: authored_metres(2.5) },
        40.0,
        100
    )
    .settled_readback_pair(),
    v!(
        "lod-medium",
        "After high transition",
        CapturePose::PlantLod { distance: authored_metres(2.85) },
        40.0,
        100
    )
    .settled_readback_pair(),
    v!(
        "lod-medium-end",
        "Before low transition",
        CapturePose::PlantLod { distance: authored_metres(6.4) },
        40.0,
        100
    )
    .settled_readback_pair()
    .vista(),
    v!(
        "lod-low-blend",
        "Medium and low crossfade",
        CapturePose::PlantLod { distance: authored_metres(7.0) },
        40.0,
        100
    )
    .settled_readback_pair()
    .vista(),
    v!(
        "lod-low",
        "After low transition",
        CapturePose::PlantLod { distance: authored_metres(7.6) },
        40.0,
        100
    )
    .settled_readback_pair()
    .vista(),
    v!(
        "lod-distant",
        "Distant botanical tier",
        CapturePose::PlantLod { distance: authored_metres(20.0) },
        40.0,
        100
    )
    .settled_readback_pair()
    .vista(),
    v!(
        "lod-fade",
        "Final distance fade",
        CapturePose::PlantLod { distance: authored_metres(25.0) },
        40.0,
        100
    )
    .settled_readback_pair()
    .vista(),
    v!(
        "lod-culled",
        "Beyond botanical range",
        CapturePose::PlantLod { distance: authored_metres(28.0) },
        40.0,
        100
    )
    .settled_readback_pair()
    .vista(),
    v!(
        "lod-return",
        "Return to high tier",
        CapturePose::PlantLod { distance: authored_metres(2.15) },
        40.0,
        100
    )
    .settled_readback_pair(),
];
