use super::super::building_review::ReviewLod;
use super::*;

pub(in crate::tactical_scene_viewer) const HEATING_REVIEW_VIEWS: [CaptureViewSpec; 30] = [
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
    CaptureViewSpec::new(
        "town-support",
        "town support",
        CapturePose::CityExterior { camera: 20 },
        96.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-upper-hearth",
        "town upper hearth",
        CapturePose::CityExterior { camera: 21 },
        78.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-floor",
        "town floor",
        CapturePose::CityExterior { camera: 22 },
        75.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-roof",
        "town roof",
        CapturePose::CityExterior { camera: 23 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-support",
        "merchant support",
        CapturePose::CityExterior { camera: 24 },
        96.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-upper-hearth",
        "merchant upper hearth",
        CapturePose::CityExterior { camera: 25 },
        78.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-kitchen-floor",
        "merchant kitchen floor",
        CapturePose::CityExterior { camera: 26 },
        75.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-upper-flue-floor",
        "merchant upper flue floor",
        CapturePose::CityExterior { camera: 27 },
        75.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-roof",
        "merchant roof",
        CapturePose::CityExterior { camera: 28 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
];
