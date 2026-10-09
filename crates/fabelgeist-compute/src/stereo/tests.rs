use super::rig::*;
use super::synthetic::{self, Scene};
use super::*;
use fabelgeist_gpu::globals::WgpuContext;

const DEGREE: f32 = std::f32::consts::PI / 180.0;

/// Every projection takes a direction to its picture and back.
#[test]
fn every_projection_takes_a_direction_to_the_picture_and_back() {
    let mut projections = vec![
        Projection::Pinhole {
            focal: [0.55, 0.7],
            principal: [0.48, 0.52],
            distortion: [-0.12, 0.03, 0.001, -0.002, 0.0],
        },
        Projection::equirectangular(360.0 * DEGREE, 180.0 * DEGREE),
        Projection::Equirectangular {
            longitude: [-100.0 * DEGREE, 80.0 * DEGREE],
            latitude: [-70.0 * DEGREE, 85.0 * DEGREE],
        },
    ];
    for model in FisheyeModel::ALL {
        let fov = if model == FisheyeModel::Orthographic {
            170.0
        } else {
            200.0
        };
        projections.push(Projection::fisheye(model, fov * DEGREE));
    }
    for projection in projections {
        for v in 1..20 {
            for u in 1..20 {
                let uv = [u as f32 / 20.0, v as f32 / 20.0];
                let Some(direction) = projection.unproject(uv) else {
                    continue;
                };
                let back = projection
                    .project(direction)
                    .unwrap_or_else(|| panic!("{projection:?} lost {uv:?}"));
                assert!(
                    (back[0] - uv[0]).abs() < 2e-3 && (back[1] - uv[1]).abs() < 2e-3,
                    "{projection:?}: {uv:?} came back as {back:?}"
                );
            }
        }
    }
}

/// The conventions every file agrees on: the middle of an equirectangular
/// picture is straight ahead, right is right, and up is the top.
#[test]
fn an_equirectangular_picture_faces_the_way_the_viewer_does() {
    let sphere = Projection::equirectangular(360.0 * DEGREE, 180.0 * DEGREE);
    let at = |d| sphere.project(d).unwrap();
    let near = |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4;
    assert!(near(at([0.0, 0.0, 1.0]), [0.5, 0.5]));
    assert!(near(at([1.0, 0.0, 0.0]), [0.75, 0.5]));
    assert!(near(at(normalize([0.0, -1.0, 1.0])), [0.5, 0.25]));
}

/// Two cameras turned differently and not quite level with each other, and a
/// pinhole grid and an epipolar one over them: a point lands on the same row
/// in both rectified pictures, and each rectified pixel reads the place in its
/// eye's image where that eye really saw the point.
#[test]
fn a_crooked_rig_rectifies_onto_rows() {
    let rig = crooked_rig(Projection::pinhole(90.0 * DEGREE, 4.0 / 3.0));
    let baseline = match rig.geometry {
        Geometry::Parallel { baseline } => baseline,
        _ => unreachable!(),
    };
    for grid in [
        Grid::Pinhole {
            horizontal_fov: 70.0 * DEGREE,
        },
        Grid::Epipolar {
            horizontal: [-35.0 * DEGREE, 35.0 * DEGREE],
            vertical: [30.0 * DEGREE, -30.0 * DEGREE],
        },
    ] {
        let rectified = Rectified::new(
            rig,
            Rectification {
                grid,
                width: 400,
                height: 300,
                pitch: 2.0 * DEGREE,
            },
        )
        .unwrap();
        let to_rig = rectified.to_rig();
        let mut checked = 0;
        for i in 0..200 {
            let t = i as f32 * 0.37;
            let point = [t.sin() * 0.8, (t * 1.3).cos() * 0.5, 2.0 + (t * 0.7).sin()];
            let (Some(left), Some(right)) = (
                rectified.project_point(0, point),
                rectified.project_point(1, point),
            ) else {
                continue;
            };
            assert!(
                (left[1] - right[1]).abs() < 1e-2,
                "{grid:?}: rows {left:?} {right:?}"
            );
            assert!(
                left[0] > right[0],
                "{grid:?}: the right eye sees it further left"
            );
            // Where each eye really saw it, from its own centre.
            let rig_point = {
                let p = transform(&to_rig, point);
                [
                    p[0] - baseline[0] * 0.5,
                    p[1] - baseline[1] * 0.5,
                    p[2] - baseline[2] * 0.5,
                ]
            };
            for (eye, pixel) in [(0, left), (1, right)] {
                let centre = rig.origin(eye, [0.0, 0.0, 1.0]);
                let seen = rig
                    .eye(eye)
                    .place(normalize([
                        rig_point[0] - centre[0],
                        rig_point[1] - centre[1],
                        rig_point[2] - centre[2],
                    ]))
                    .unwrap();
                let read = rectified.source(eye, pixel[0], pixel[1], (1, 1)).unwrap();
                assert!(
                    (read[0] - seen[0]).abs() < 1e-3 && (read[1] - seen[1]).abs() < 1e-3,
                    "{grid:?} eye {eye}: reads {read:?}, saw {seen:?}"
                );
            }
            checked += 1;
        }
        assert!(checked > 100, "{checked} points were in view");
    }
}

/// Omnidirectional stereo is rectified already: the two eyes' pictures of a
/// point share a latitude, and the left one sees it further right.
#[test]
fn omnidirectional_stereo_keeps_a_point_on_its_row() {
    let rig = omnidirectional_rig();
    let rectified = Rectified::new(
        rig,
        Rectification {
            grid: Grid::Panorama {
                longitude: [-180.0 * DEGREE, 180.0 * DEGREE],
                latitude: [80.0 * DEGREE, -80.0 * DEGREE],
            },
            width: 1440,
            height: 640,
            pitch: 0.0,
        },
    )
    .unwrap();
    for point in [[0.0, 0.0, 2.0], [1.5, -0.4, -1.0], [-2.0, 0.8, 0.3]] {
        let left = rectified.project_point(0, point).unwrap();
        let right = rectified.project_point(1, point).unwrap();
        assert!(
            (left[1] - right[1]).abs() < 0.2,
            "{point:?}: {left:?} {right:?}"
        );
        assert!(left[0] > right[0], "{point:?}: {left:?} {right:?}");
    }
}

#[test]
fn a_grid_that_cannot_rectify_a_geometry_is_refused() {
    let rig = crooked_rig(Projection::pinhole(90.0 * DEGREE, 1.0));
    let panorama = Rectification {
        grid: Grid::Panorama {
            longitude: [-1.0, 1.0],
            latitude: [1.0, -1.0],
        },
        width: 64,
        height: 64,
        pitch: 0.0,
    };
    assert!(Rectified::new(rig, panorama).is_err());
}

fn crooked_rig(projection: Projection) -> StereoRig {
    StereoRig {
        left: Eye::new([0.0, 0.0, 0.5, 1.0], projection).with_rotation(multiply(
            &rotation_y(3.0 * DEGREE),
            &rotation_z(1.0 * DEGREE),
        )),
        right: Eye::new([0.5, 0.0, 0.5, 1.0], projection).with_rotation(multiply(
            &rotation_x(-1.5 * DEGREE),
            &rotation_y(-2.0 * DEGREE),
        )),
        geometry: Geometry::Parallel {
            baseline: [0.1, 0.004, -0.003],
        },
    }
}

fn omnidirectional_rig() -> StereoRig {
    StereoRig {
        left: Eye::new(
            [0.0, 0.0, 1.0, 0.5],
            Projection::equirectangular(360.0 * DEGREE, 180.0 * DEGREE),
        ),
        right: Eye::new(
            [0.0, 0.5, 1.0, 0.5],
            Projection::equirectangular(360.0 * DEGREE, 180.0 * DEGREE),
        ),
        geometry: Geometry::Omnidirectional { ipd: 0.064 },
    }
}

async fn context() -> WgpuContext {
    WgpuContext::new().await.expect("a GPU device")
}

/// The scene's own disparity, as a perfect matcher would report it.
fn true_match(truth: &synthetic::Truth) -> (Vec<f32>, Vec<f32>) {
    truth
        .disparity
        .iter()
        .map(|d| {
            if d.is_finite() {
                (*d, 1.0)
            } else {
                (-1.0, 0.0)
            }
        })
        .unzip()
}

fn median(mut values: Vec<f32>) -> f32 {
    values.sort_by(f32::total_cmp);
    values.get(values.len() / 2).copied().unwrap_or(f32::NAN)
}

/// Handed the disparity the scene really has, the card puts every grid pixel
/// at the distance the scene is along its ray, and every pixel of each eye's
/// own picture at the distance the scene is from that eye -- for a rig of each
/// kind of grid.
#[tokio::test]
async fn the_true_disparity_comes_back_as_each_eyes_distance() {
    let context = context().await;
    let scene = Scene::default();
    let cases = [
        (
            "distorted crooked pinhole",
            crooked_rig(Projection::Pinhole {
                focal: [0.6, 0.8],
                principal: [0.5, 0.5],
                distortion: [-0.08, 0.01, 0.0005, -0.0005, 0.0],
            }),
            Grid::Pinhole {
                horizontal_fov: 70.0 * DEGREE,
            },
            (320, 240),
            (160, 120),
        ),
        (
            "crooked fisheye",
            crooked_rig(Projection::fisheye(
                FisheyeModel::Equidistant,
                190.0 * DEGREE,
            )),
            Grid::Epipolar {
                horizontal: [-80.0 * DEGREE, 80.0 * DEGREE],
                vertical: [80.0 * DEGREE, -80.0 * DEGREE],
            },
            (320, 320),
            (160, 160),
        ),
        (
            "omnidirectional panorama",
            omnidirectional_rig(),
            Grid::Panorama {
                longitude: [-180.0 * DEGREE, 180.0 * DEGREE],
                latitude: [75.0 * DEGREE, -75.0 * DEGREE],
            },
            (640, 267),
            (320, 160),
        ),
    ];
    for (name, rig, grid, (width, height), view) in cases {
        let rectified = Rectified::new(
            rig,
            Rectification {
                grid,
                width,
                height,
                pitch: 0.0,
            },
        )
        .unwrap();
        let truth = synthetic::truth(&rectified, &scene);
        let (disparity, confidence) = true_match(&truth);
        let widest = disparity.iter().fold(0.0f32, |most, d| most.max(*d));
        let mut depth = StereoDepth::new(
            &context,
            rectified,
            TriangulationSettings {
                max_disparity: widest.ceil() as u32 + 2,
                ..Default::default()
            },
            None,
        )
        .unwrap()
        .with_views(&context, [view, view])
        .unwrap();
        depth.upload(&context, &disparity, &confidence).unwrap();
        depth.run(&context, None).unwrap();

        let distance: Vec<f32> = depth.distance().read(&context).await.unwrap();
        let errors: Vec<f32> = distance
            .iter()
            .zip(&truth.distance)
            .zip(&disparity)
            .filter(|((found, real), d)| {
                **d >= 0.0 && real.is_finite() && **real < 30.0 && **found > 0.0
            })
            .map(|((found, real), _)| (found - real).abs() / real)
            .collect();
        assert!(
            errors.len() > 1000,
            "{name}: only {} grid pixels to compare",
            errors.len()
        );
        let grid_error = median(errors);
        assert!(
            grid_error < 0.01,
            "{name}: grid distance off by {grid_error} (median)"
        );

        for eye in 0..2 {
            let found: Vec<f32> = depth
                .view_distance(eye)
                .unwrap()
                .read(&context)
                .await
                .unwrap();
            let expected = synthetic::view_truth(&rectified, &scene, eye, view);
            let score = synthetic::score_view(&expected, &found);
            assert!(
                score.coverage > 0.8 && score.median_relative_error < 0.02,
                "{name} eye {eye}: {score:?}"
            );
        }
    }
}

/// On a still scene the history agrees with every new frame, so the filter
/// keeps every point it was handed and does not move any of them further than
/// the tolerance allows.
#[tokio::test]
async fn the_temporal_filter_holds_a_still_scene() {
    let context = context().await;
    let rig = crooked_rig(Projection::pinhole(90.0 * DEGREE, 800.0 / 600.0));
    let rectified = Rectified::new(
        rig,
        Rectification {
            grid: Grid::Pinhole {
                horizontal_fov: 70.0 * DEGREE,
            },
            width: 256,
            height: 192,
            pitch: 0.0,
        },
    )
    .unwrap();
    let truth = synthetic::truth(&rectified, &Scene::default());
    let (disparity, confidence) = true_match(&truth);
    let mut depth = StereoDepth::new(
        &context,
        rectified,
        TriangulationSettings::default(),
        Some(TemporalSettings::default()),
    )
    .unwrap();
    for _ in 0..4 {
        depth.upload(&context, &disparity, &confidence).unwrap();
        depth.run(&context, None).unwrap();
    }
    let raw: Vec<f32> = depth.distance().read(&context).await.unwrap();
    let filtered: Vec<f32> = depth.filtered_distance().read(&context).await.unwrap();
    let kept = raw.iter().filter(|r| **r > 0.0).count();
    let held = filtered.iter().filter(|r| **r > 0.0).count();
    assert!(kept > 1000, "only {kept} points to hold");
    assert!(held >= kept, "the filter lost points: {held} of {kept}");
    for (r, f) in raw.iter().zip(&filtered) {
        if *r > 0.0 && *r < 30.0 {
            assert!((1.0 / r - 1.0 / f).abs() < 0.05, "{r} became {f}");
        }
    }
}
