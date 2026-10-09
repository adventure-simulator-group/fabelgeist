use fabelgeist_math::vector::Vec3;

use super::surface::tests::cylinder;
use super::*;
use crate::{ArmorMorph, PlateFace, SurfaceGrid, Tiling};

/// A bare cylindrical piece skinned to one joint above and another below,
/// with a morph that shifts it sideways.
fn sleeve(columns_reversed: bool) -> GeneratedArmor {
    let (mut grid, positions, normals) = cylinder(0.05, 0.25, 11, 48);
    if columns_reversed {
        grid.vertices = (0..grid.rows)
            .flat_map(|row| (0..grid.columns).rev().map(move |c| row * grid.columns + c))
            .collect();
    }
    let count = positions.len();
    let (joint_indices, joint_weights) = positions
        .iter()
        .map(|p| {
            let upper = (p[1] / 0.25).clamp(0.0, 1.0);
            let mut joints = [0u32; 8];
            let mut weights = [0.0f32; 8];
            joints[..2].copy_from_slice(&[1, 2]);
            weights[..2].copy_from_slice(&[upper, 1.0 - upper]);
            (joints, weights)
        })
        .unzip();
    let shift = [0.01, 0.0, 0.0];
    GeneratedArmor {
        components: Vec::new(),
        design_hash: [0; 32],
        surface_domain: "test".into(),
        texcoords: vec![[0.0; 2]; count],
        joint_indices,
        joint_weights,
        indices: Vec::new(),
        faces: Vec::new(),
        trim: None,
        grids: vec![grid],
        morphs: vec![ArmorMorph {
            name: "shifted".into(),
            direct_positions: positions
                .iter()
                .map(|p| std::array::from_fn(|k| p[k] + shift[k]))
                .collect(),
            position_deltas: vec![shift; count],
            normal_deltas: vec![[0.0; 3]; count],
        }],
        positions,
        normals,
    }
}

fn radial(position: [f32; 3]) -> Vec3 {
    Vec3::new(position[0], 0.0, position[2]).normalize()
}

#[test]
fn a_solid_piece_is_left_as_it_is() {
    let piece = sleeve(false);
    let constructed = piece.clone().constructed(&Construction::Solid).unwrap();
    assert_eq!(constructed.plates, piece);
    assert!(constructed.lacing.is_none());
}

#[test]
fn scales_cover_the_piece_facing_out_and_follow_its_morphs() {
    for reversed in [false, true] {
        let constructed = sleeve(reversed)
            .constructed(&Construction::Scale(Tiling::scale()))
            .unwrap();
        for piece in [&constructed.plates, constructed.lacing.as_ref().unwrap()] {
            assert!(!piece.indices.is_empty());
            assert!(piece.positions.iter().flatten().all(|x| x.is_finite()));
            assert!(
                piece
                    .indices
                    .iter()
                    .all(|i| (*i as usize) < piece.positions.len())
            );
            for weights in &piece.joint_weights {
                assert!((weights.iter().sum::<f32>() - 1.0).abs() < 1e-5);
            }
            // The whole piece moves with the morph that shifts its surface.
            let morph = &piece.morphs[0];
            assert_eq!(morph.position_deltas.len(), piece.positions.len());
            for delta in &morph.position_deltas {
                assert!((delta[0] - 0.01).abs() < 1e-5 && delta[1].abs() < 1e-5);
            }
        }
        let plates = &constructed.plates;
        assert_eq!(plates.faces.len() * 3, plates.indices.len());
        for (triangle, face) in plates.indices.chunks(3).zip(&plates.faces) {
            if *face != PlateFace::Outer {
                continue;
            }
            for corner in triangle {
                let corner = *corner as usize;
                let normal = Vec3::from_array(plates.normals[corner]);
                assert!(
                    normal.dot(radial(plates.positions[corner])) > 0.8,
                    "reversed {reversed}"
                );
            }
        }
    }
}

#[test]
fn plates_stay_close_to_the_surface_they_are_laid_on() {
    let tiling = Tiling::lamellar();
    let plates = sleeve(false)
        .constructed(&Construction::Lamellar(tiling.clone()))
        .unwrap()
        .plates;
    for position in &plates.positions {
        let height = Vec3::new(position[0], 0.0, position[2]).length() - 0.05;
        // A gauge under the surface at a plate's top, its tilt above it at
        // its foot.
        assert!(height > -tiling.plate.thickness * 1.5, "{height}");
        assert!(height < tiling.plate.height * 0.2, "{height}");
    }
}

#[test]
fn lamellar_hangs_every_row_but_the_last_from_the_one_above() {
    let construction = Construction::Lamellar(Tiling::lamellar());
    let laced = sleeve(false).constructed(&construction).unwrap();
    let mut unlaced = construction.clone();
    unlaced.tiling_mut().unwrap().lacing = None;
    let bare = sleeve(false).constructed(&unlaced).unwrap();
    assert!(bare.lacing.is_none());
    let lacing = laced.lacing.unwrap();
    // Cords run down the top row's faces, from its lowest holes to its foot.
    let plate = Tiling::lamellar().plate;
    let center = 0.25 - plate.height * 0.5;
    let lowest_hole = center + plate.holes().last().unwrap()[1] - plate.hole_radius;
    let foot = center - plate.height * 0.5;
    let over_faces = lacing
        .positions
        .iter()
        .filter(|p| p[1] < lowest_hole && p[1] > foot + plate.thickness)
        .filter(|p| Vec3::new(p[0], 0.0, p[2]).length() > 0.05 + plate.thickness)
        .count();
    assert!(over_faces > 0);
    assert_ne!(laced.plates.design_hash, bare.plates.design_hash);
}

#[test]
fn a_piece_without_a_grid_cannot_take_plates() {
    let mut piece = sleeve(false);
    piece.grids.clear();
    assert_eq!(
        piece.constructed(&Construction::Scale(Tiling::scale())),
        Err(ConstructionError::NoSurfaceGrid)
    );
    let mut piece = sleeve(false);
    piece.grids = vec![SurfaceGrid::regular(1, 4, false, vec![0, 1, 2, 3])];
    assert_eq!(
        piece.constructed(&Construction::Scale(Tiling::scale())),
        Err(ConstructionError::NoSurfaceGrid)
    );
}

#[test]
fn trim_runs_along_each_plates_rim_and_leaves_its_face() {
    let construction = Construction::Scale(Tiling::scale());
    let plates = sleeve(false).constructed(&construction).unwrap().plates;
    let trim = construction.trim_on(&crate::trim::Trim::default());
    let trimmed = plates
        .trimmed(crate::TrimBand {
            width: trim.width,
            period: trim.period(),
        })
        .unwrap();
    let outer = |range: std::ops::Range<usize>| {
        range
            .step_by(3)
            .filter(|i| trimmed.faces[i / 3] == PlateFace::Outer)
            .count()
    };
    let surfaces = trimmed.surfaces();
    let band: usize = surfaces.iter().map(|s| outer(s.trim.clone())).sum();
    let face: usize = surfaces.iter().map(|s| outer(s.plate.clone())).sum();
    // Each scale keeps its own face inside a band along its own rim, however
    // close the scales around it lie.
    assert!(band > 0 && face > 0, "band {band}, face {face}");
}
