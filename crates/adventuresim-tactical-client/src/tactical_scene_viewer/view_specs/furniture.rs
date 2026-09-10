use super::*;

pub(in crate::tactical_scene_viewer) const FURNITURE_REVIEW_VIEWS: [CaptureViewSpec; 11] = [
    v!(
        "warmup",
        "Furniture and road production pipeline warmup",
        CapturePose::CityExterior { camera: 0 },
        60.0,
        100
    )
    .warmup()
    .vista(),
    v!(
        "barrels",
        "Coopered casks beside working frontage",
        CapturePose::CityExterior { camera: 0 },
        55.0,
        100
    )
    .vista(),
    v!(
        "cargo",
        "Crates and stock with handling space",
        CapturePose::CityExterior { camera: 1 },
        55.0,
        100
    )
    .vista(),
    v!(
        "seating",
        "Outdoor table and benches",
        CapturePose::CityExterior { camera: 2 },
        60.0,
        100
    )
    .vista(),
    v!(
        "market-stall",
        "Supported canvas stall and market aisle",
        CapturePose::CityExterior { camera: 3 },
        65.0,
        100
    )
    .vista(),
    v!(
        "horse-stop",
        "Hitching rail, trough and horse standing room",
        CapturePose::CityExterior { camera: 4 },
        65.0,
        100
    )
    .vista(),
    v!(
        "market-oblique",
        "Market groups and connected street surfaces",
        CapturePose::CityExterior { camera: 5 },
        60.0,
        100
    )
    .vista(),
    v!(
        "road-detail",
        "Cobbles, joint fill and traffic wear at eye height",
        CapturePose::CityExterior { camera: 6 },
        60.0,
        100
    )
    .vista(),
    v!(
        "junction-oblique",
        "Curved carriage paths and shared junction mud",
        CapturePose::CityExterior { camera: 7 },
        55.0,
        100
    )
    .vista(),
    v!(
        "junction-close",
        "Overlapping axle widths and turning wheel paths",
        CapturePose::CityExterior { camera: 8 },
        55.0,
        100
    )
    .vista(),
    v!(
        "placement-clearances",
        "Accepted groups and reserved routes",
        CapturePose::CityExterior { camera: 5 },
        60.0,
        100
    )
    .vista()
    .overlay(),
];
