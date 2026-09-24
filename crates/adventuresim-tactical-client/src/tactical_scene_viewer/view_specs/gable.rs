use super::super::building_review::ReviewLod;
use super::*;

pub(in crate::tactical_scene_viewer) const GABLE_REVIEW_VIEWS: [CaptureViewSpec; 16] = [
    CaptureViewSpec::new(
        "warmup",
        "Gable material warmup",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .warmup()
    .vista(),
    CaptureViewSpec::new(
        "town-front-detail",
        "Town front detail",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-front-facade",
        "Town front facade",
        CapturePose::CityExterior { camera: 1 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-front-shell",
        "Town front shell",
        CapturePose::CityExterior { camera: 2 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Shell)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-rear-detail",
        "Town rear detail",
        CapturePose::CityExterior { camera: 3 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-attic-detail",
        "Town attic detail",
        CapturePose::CityExterior { camera: 4 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-front-detail",
        "Merchant front detail",
        CapturePose::CityExterior { camera: 5 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-front-facade",
        "Merchant front facade",
        CapturePose::CityExterior { camera: 6 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-front-shell",
        "Merchant front shell",
        CapturePose::CityExterior { camera: 7 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Shell)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-rear-detail",
        "Merchant rear detail",
        CapturePose::CityExterior { camera: 8 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-attic-detail",
        "Merchant attic detail",
        CapturePose::CityExterior { camera: 9 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-47-front",
        "Town 47 front",
        CapturePose::CityExterior { camera: 10 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-101-front",
        "Town 101 front",
        CapturePose::CityExterior { camera: 11 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-47-front",
        "Merchant 47 front",
        CapturePose::CityExterior { camera: 12 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-101-front",
        "Merchant 101 front",
        CapturePose::CityExterior { camera: 13 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "gable-street-overview",
        "Gable street overview",
        CapturePose::CityExterior { camera: 14 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
];
