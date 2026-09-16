use super::super::building_review::ReviewLod;
use super::*;

pub(in crate::tactical_scene_viewer) const HEATING_REVIEW_VIEWS: [CaptureViewSpec; 21] = [
    CaptureViewSpec::new(
        "warmup",
        "Heating material warmup",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .warmup()
    .vista(),
    CaptureViewSpec::new(
        "cottage-hearth",
        "cottage hearth",
        CapturePose::CityExterior { camera: 0 },
        78.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "cottage-stove",
        "cottage stove",
        CapturePose::CityExterior { camera: 1 },
        68.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "cottage-roof-detail",
        "cottage roof detail",
        CapturePose::CityExterior { camera: 2 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "cottage-roof-facade",
        "cottage roof facade",
        CapturePose::CityExterior { camera: 3 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "cottage-roof-shell",
        "cottage roof shell",
        CapturePose::CityExterior { camera: 4 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Shell)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-hearth",
        "hall hearth",
        CapturePose::CityExterior { camera: 5 },
        78.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-stove",
        "hall stove",
        CapturePose::CityExterior { camera: 6 },
        68.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-roof-detail",
        "hall roof detail",
        CapturePose::CityExterior { camera: 7 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-roof-facade",
        "hall roof facade",
        CapturePose::CityExterior { camera: 8 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-roof-shell",
        "hall roof shell",
        CapturePose::CityExterior { camera: 9 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Shell)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "cottage-47-roof",
        "cottage 47 roof",
        CapturePose::CityExterior { camera: 10 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "cottage-101-roof",
        "cottage 101 roof",
        CapturePose::CityExterior { camera: 11 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-47-roof",
        "hall 47 roof",
        CapturePose::CityExterior { camera: 12 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-101-roof",
        "hall 101 roof",
        CapturePose::CityExterior { camera: 13 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "heated-houses-overview",
        "heated houses overview",
        CapturePose::CityExterior { camera: 14 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "deep-hall-hearth",
        "deep hall hearth",
        CapturePose::CityExterior { camera: 15 },
        110.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "deep-hall-stove",
        "deep hall stove",
        CapturePose::CityExterior { camera: 16 },
        68.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "deep-hall-roof-detail",
        "deep hall roof detail",
        CapturePose::CityExterior { camera: 17 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "deep-hall-roof-facade",
        "deep hall roof facade",
        CapturePose::CityExterior { camera: 18 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "deep-hall-roof-shell",
        "deep hall roof shell",
        CapturePose::CityExterior { camera: 19 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Shell)
    .settled_readback_pair(),
];
