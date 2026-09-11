use adventuresim_armor_model::{BoundaryNormals, PartFrame, PartMesh, ShellExtrusion};

fn plate() -> PartMesh {
    PartMesh::from_relief_surface(
        vec![
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ],
        vec![0, 1, 2, 0, 2, 3],
        0.002,
        BoundaryNormals::Separate,
        ShellExtrusion::Normal,
        Some(vec![0.0, 0.003, 0.001, 0.0]),
    )
    .unwrap()
}

#[test]
fn fitting_operates_on_the_carrier_and_preserves_relief_and_gauge() {
    let original = plate();
    let fitted = original
        .refit_surfaces(|points| {
            assert!(
                points.iter().all(|p| p[2] == 0.0),
                "fitter received relief instead of the carrier"
            );
            for point in points {
                point[2] = 0.2;
            }
        })
        .unwrap();
    assert_eq!(original.indices, fitted.indices);
    for (a, b) in original.positions.iter().zip(&fitted.positions) {
        assert!((b[2] - a[2] - 0.2).abs() < 1e-6);
    }
    for index in 0..4 {
        assert!((fitted.positions[index][2] - fitted.positions[index + 4][2] - 0.002).abs() < 1e-6);
    }
    let refitted = fitted.refit_surfaces(|_| {}).unwrap();
    assert_eq!(
        refitted.positions, fitted.positions,
        "repeated fitting accumulated relief"
    );
    assert!(refitted.normals().is_ok());
}

#[test]
fn relief_carriers_follow_reflected_frames_and_appended_shells() {
    let frame = PartFrame {
        origin: [3.0, 4.0, 5.0],
        axes: [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],
        half_extents: [1.0; 3],
    };
    let mut mesh = plate().transformed(&frame);
    mesh.append(plate().transformed(&PartFrame {
        origin: [8.0, 0.0, 0.0],
        ..frame
    }));
    let mut shells = 0;
    let fitted = mesh
        .refit_surfaces(|points| {
            shells += 1;
            assert!(
                points
                    .iter()
                    .all(|p| (p[1] - if shells == 1 { 4.0 } else { 0.0 }).abs() < 1e-6)
            );
        })
        .unwrap();
    assert_eq!(shells, 2);
    assert_eq!(mesh.indices, fitted.indices);
    for (original, rebuilt) in mesh.positions.iter().zip(&fitted.positions) {
        for axis in 0..3 {
            assert!(
                (original[axis] - rebuilt[axis]).abs() < 1e-6,
                "reflection/refitting changed a vertex or its boundary alias"
            );
        }
    }
}

#[test]
fn invalid_relief_arrays_fail_before_shell_generation() {
    for heights in [vec![0.0], vec![0.0, f32::NAN, 0.0], vec![0.0, -0.001, 0.0]] {
        assert!(
            PartMesh::from_relief_surface(
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                vec![0, 1, 2],
                0.002,
                BoundaryNormals::Separate,
                ShellExtrusion::Normal,
                Some(heights)
            )
            .is_err()
        );
    }
}

fn oblique_carrier() -> Vec<[f32; 3]> {
    vec![
        [-0.04, -0.04, 0.08],
        [0.04, -0.04, 0.08],
        [0.04, 0.04, 0.12],
        [-0.04, 0.04, 0.12],
    ]
}

fn directed_plate(extrusion: ShellExtrusion) -> PartMesh {
    PartMesh::from_relief_surface(
        oblique_carrier(),
        vec![0, 1, 2, 0, 2, 3],
        0.002,
        BoundaryNormals::Separate,
        extrusion,
        Some(vec![0.0, 0.003, 0.001, 0.0]),
    )
    .unwrap()
}

fn directed_extrusions() -> [ShellExtrusion; 2] {
    [
        ShellExtrusion::Along {
            direction: [0.0, 0.0, 1.0],
        },
        ShellExtrusion::Radial {
            origin: [0.01, 0.0, -0.02],
            axis: [0.0, 1.0, 0.0],
        },
    ]
}

#[test]
fn directed_relief_retains_oblique_normal_gauge_and_trim_coordinates() {
    // The fixture lies on 2z - y = 0.2. Signed distance to that plane gives
    // an independent physical measurement of relief and plate gauge.
    let distance = |p: [f32; 3]| (2.0 * p[2] - p[1] - 0.2) / 5.0_f32.sqrt();
    for extrusion in directed_extrusions() {
        let mesh = directed_plate(extrusion);
        for (index, height) in [0.0, 0.003, 0.001, 0.0].into_iter().enumerate() {
            let outer = mesh.positions[index];
            let inner = mesh.positions[index + 4];
            let carrier = oblique_carrier()[index];
            assert!((distance(outer) - height).abs() < 1e-6);
            assert!((distance(inner) - (height - 0.002)).abs() < 1e-6);
            for point in [outer, inner] {
                assert_eq!(point[1], carrier[1], "extrusion moved opening height");
                match extrusion {
                    ShellExtrusion::Along { .. } => {
                        assert_eq!(point[0], carrier[0], "extrusion moved lateral trim");
                    }
                    ShellExtrusion::Radial { origin, .. } => {
                        let ray_cross = (point[0] - origin[0]) * (carrier[2] - origin[2])
                            - (point[2] - origin[2]) * (carrier[0] - origin[0]);
                        assert!(ray_cross.abs() < 1e-8, "extrusion changed polar angle");
                    }
                    ShellExtrusion::Normal
                    | ShellExtrusion::AngleWeightedNormal
                    | ShellExtrusion::CappedAxis { .. } => unreachable!(),
                }
            }
        }
    }
}

#[test]
fn directed_shells_keep_extrusion_and_outward_winding_after_rigid_frames_and_refits() {
    for extrusion in directed_extrusions()
        .into_iter()
        .chain([ShellExtrusion::CappedAxis {
            origin: [0.01, 0.0, -0.02],
            axis: [0.0, 1.0, 0.0],
        }])
    {
        for handedness in [-1.0, 1.0] {
            let frame = PartFrame {
                origin: [0.3, 0.4, 0.5],
                axes: [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, handedness, 0.0]],
                half_extents: [1.0; 3],
            };
            frame.validate().unwrap();
            let original = directed_plate(extrusion);
            let original_normals = original.normals().unwrap();
            let transformed = original.transformed(&frame);
            let rebuilt = transformed.refit_surfaces(|_| {}).unwrap();
            assert_eq!(transformed.indices, rebuilt.indices);
            assert_eq!(transformed.positions.len(), rebuilt.positions.len());
            for (a, b) in transformed.positions.iter().zip(&rebuilt.positions) {
                assert!((0..3).all(|i| (a[i] - b[i]).abs() < 1e-6));
            }
            for (before, after) in original_normals.iter().zip(rebuilt.normals().unwrap()) {
                let expected: [f32; 3] = std::array::from_fn(|axis| {
                    (0..3).map(|i| before[i] * frame.axes[i][axis]).sum()
                });
                assert!(
                    (0..3).all(|i| (expected[i] - after[i]).abs() < 1e-4),
                    "transformed shell or return normals stopped pointing outward"
                );
            }
        }
    }
}

#[test]
fn directed_extrusion_rejects_invalid_and_tangent_directions() {
    let invalid = [
        [0.0; 3],
        [0.0, 0.0, 2.0],
        [f32::NAN, 0.0, 1.0],
        [0.0, f32::INFINITY, 1.0],
    ];
    let extrusions = invalid
        .into_iter()
        .flat_map(|direction| {
            [
                ShellExtrusion::Along { direction },
                ShellExtrusion::Radial {
                    origin: [0.0; 3],
                    axis: direction,
                },
                ShellExtrusion::CappedAxis {
                    origin: [0.0; 3],
                    axis: direction,
                },
            ]
        })
        .chain([
            ShellExtrusion::Along {
                direction: [1.0, 0.0, 0.0],
            },
            ShellExtrusion::Along {
                direction: [0.0, 0.0, -1.0],
            },
            ShellExtrusion::Radial {
                origin: [0.0, 0.0, 1.0],
                axis: [0.0, 1.0, 0.0],
            },
            ShellExtrusion::Radial {
                origin: [f32::NAN, 0.0, 0.0],
                axis: [0.0, 1.0, 0.0],
            },
            ShellExtrusion::CappedAxis {
                origin: [f32::NAN, 0.0, 0.0],
                axis: [0.0, 1.0, 0.0],
            },
        ]);
    for extrusion in extrusions {
        assert!(
            PartMesh::from_relief_surface(
                oblique_carrier(),
                vec![0, 1, 2, 0, 2, 3],
                0.002,
                BoundaryNormals::Separate,
                extrusion,
                Some(vec![0.003; 4]),
            )
            .is_err(),
            "accepted invalid extrusion {extrusion:?}"
        );
    }
}

#[test]
fn directed_extrusion_rejects_a_backward_carrier_with_or_without_relief() {
    for extrusion in directed_extrusions() {
        for relief in [None, Some(vec![0.003; 4])] {
            assert!(
                PartMesh::from_relief_surface(
                    oblique_carrier(),
                    vec![0, 2, 1, 0, 3, 2],
                    0.002,
                    BoundaryNormals::Separate,
                    extrusion,
                    relief,
                )
                .is_err()
            );
        }
    }
}
