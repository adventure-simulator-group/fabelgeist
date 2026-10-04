//! Frozen absolute poses keep terrain comparisons independent of generation.
use super::{capture_state::BuildingReviewCamera, capture_state::SceneCaptureState};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub(crate) const PROFILE: &str = "terrain-grounding";
const CONTRACT_VERSION: u16 = 2;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Camera {
    position_metres: [f32; 3],
    target_metres: [f32; 3],
}

impl Camera {
    fn validate(&self) -> bool {
        let position = Vec3::from_array(self.position_metres);
        let target = Vec3::from_array(self.target_metres);
        position.is_finite()
            && target.is_finite()
            && (target - position).cross(Vec3::Y).length_squared() > f32::EPSILON
    }

    fn review(&self) -> BuildingReviewCamera {
        BuildingReviewCamera {
            position: Vec3::from_array(self.position_metres),
            target: Vec3::from_array(self.target_metres),
            plaster_raking_light: None,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u16,
    /// Scene-relative metres, with the same seven indexes as city-review.
    exterior: Vec<Camera>,
    /// Frozen eye-height pose; scene-performance uses its established 80° FOV.
    benchmark: Option<Camera>,
    /// Explicit physical members to inspect with the production playable LODs.
    playable_building_ids: Vec<u64>,
}

#[derive(Resource)]
pub(super) struct Contract(Option<Document>);

impl Contract {
    pub(super) fn load(path: Option<PathBuf>, profile: &str) -> Self {
        assert!(
            path.is_some() || profile != PROFILE,
            "terrain-grounding requires --city-cameras"
        );
        let document = path.map(|path| {
            let bytes = std::fs::read(&path).expect("read explicit camera contract");
            let document: Document =
                serde_json::from_slice(&bytes).expect("decode camera contract");
            assert_eq!(
                document.version, CONTRACT_VERSION,
                "camera contract version"
            );
            assert!(
                document.exterior.len() == 7,
                "camera contract requires all seven fixed city poses"
            );
            assert!(document.exterior.iter().all(Camera::validate));
            assert!(document.benchmark.as_ref().is_none_or(Camera::validate));
            document
        });
        Self(document)
    }

    pub(super) fn promote(
        &self,
        input: &mut adventuresim_tactical_core::scene_input::TacticalSceneInput,
    ) {
        let Some(document) = &self.0 else { return };
        let requested: std::collections::BTreeSet<_> =
            document.playable_building_ids.iter().copied().collect();
        assert_eq!(requested.len(), document.playable_building_ids.len());
        for id in requested {
            if input.buildings.iter().any(|building| building.id == id) {
                continue;
            }
            let index = input
                .distant_buildings
                .iter()
                .position(|building| building.id == id)
                .expect("explicit capture member must exist");
            let distant = input.distant_buildings.remove(index);
            input.buildings.push(distant.into());
        }
    }
}

pub(super) fn apply(contract: Res<Contract>, mut state: ResMut<SceneCaptureState>) {
    let Some(document) = &contract.0 else { return };
    state.city_exterior_cameras = document.exterior.iter().map(Camera::review).collect();
    if let Some(camera) = &document.benchmark {
        state.ground_eye_position = Vec3::from_array(camera.position_metres);
        state.ground_eye_target = Vec3::from_array(camera.target_metres);
    }
    std::fs::write(
        state.output.join("fixed-camera-contract.json"),
        serde_json::to_vec_pretty(document).unwrap(),
    )
    .expect("retain frozen camera contract");
}

use super::{
    building_review::ReviewLod,
    view_specs::{CapturePose, CaptureViewSpec},
};

// Indices match the established city-review profile. Foundation LOD views use
// one identical pose, FOV and lighting; only the production LOD range changes.
pub(super) const VIEWS: [CaptureViewSpec; 9] = [
    CaptureViewSpec::new(
        "warmup",
        "Production pipeline warmup",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .warmup()
    .vista(),
    CaptureViewSpec::new(
        "foundation-detail",
        "Foundation and threshold: detail",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .building_lod(ReviewLod::Detail)
    .vista(),
    CaptureViewSpec::new(
        "foundation-facade",
        "Foundation and threshold: facade",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .building_lod(ReviewLod::Facade)
    .vista(),
    CaptureViewSpec::new(
        "foundation-shell",
        "Foundation and threshold: shell",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .building_lod(ReviewLod::Shell)
    .vista(),
    CaptureViewSpec::new(
        "street",
        "Street access and local relief",
        CapturePose::CityExterior { camera: 1 },
        72.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "property-oblique",
        "Whole property grading boundary",
        CapturePose::CityExterior { camera: 2 },
        60.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "city-oblique",
        "Settlement and geographic relief",
        CapturePose::CityExterior { camera: 4 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "overhead",
        "Development and bounded grading",
        CapturePose::CityExterior { camera: 5 },
        62.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "horizon",
        "Geographic horizon",
        CapturePose::CityExterior { camera: 6 },
        50.0,
        100,
    )
    .vista(),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_contract_rejects_degenerate_and_nonfinite_poses() {
        assert!(
            !Camera {
                position_metres: [0., 10., 0.],
                target_metres: [0.; 3]
            }
            .validate()
        );
        assert!(
            !Camera {
                position_metres: [f32::NAN, 2., 4.],
                target_metres: [0.; 3]
            }
            .validate()
        );
        assert!(
            Camera {
                position_metres: [0., 10., 0.01],
                target_metres: [0.; 3]
            }
            .validate()
        );
    }

    #[test]
    fn explicit_capture_promotion_preserves_complete_physical_bindings() {
        use adventuresim_tactical_core::scene_input::TacticalSceneInput;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/tactical-scenes/massive-city.json");
        let mut input = TacticalSceneInput::load(&path).unwrap();
        let original = input.clone();
        let member = *input.distant_buildings.last().unwrap();
        let contract = Contract(Some(Document {
            version: CONTRACT_VERSION,
            exterior: Vec::new(),
            benchmark: None,
            playable_building_ids: vec![member.id],
        }));
        contract.promote(&mut input);
        let promoted = input.buildings.iter().find(|b| b.id == member.id).unwrap();
        assert_eq!(*promoted, member.into());
        assert!(!input.distant_buildings.iter().any(|b| b.id == member.id));
        let placements = |input: &TacticalSceneInput| {
            input
                .buildings
                .iter()
                .cloned()
                .chain(input.distant_buildings.iter().copied().map(Into::into))
                .collect::<Vec<_>>()
        };
        let mut before = placements(&original);
        let mut after = placements(&input);
        before.sort_by_key(|b| b.id);
        after.sort_by_key(|b| b.id);
        assert_eq!(before, after);
        assert_eq!(original.grounding, input.grounding);
        assert_eq!(original.compounds, input.compounds);
        assert_eq!(original.gardens, input.gardens);
        assert_eq!(original.properties, input.properties);
        input.validate().unwrap();
    }

    #[test]
    fn forced_foundation_levels_have_identical_pose_fov_and_lighting() {
        let reference = VIEWS[1];
        for view in &VIEWS[2..=3] {
            assert_eq!(view.pose, reference.pose);
            assert_eq!(view.fov_degrees, reference.fov_degrees);
            assert_eq!(view.lighting_mode, reference.lighting_mode);
        }
        assert_eq!(VIEWS[1].building_lod_override, Some(ReviewLod::Detail));
        assert_eq!(VIEWS[2].building_lod_override, Some(ReviewLod::Facade));
        assert_eq!(VIEWS[3].building_lod_override, Some(ReviewLod::Shell));
    }
}
