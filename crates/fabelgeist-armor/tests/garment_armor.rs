use fabelgeist_gpu::prelude::BufferUpload;
mod common;

use common::{assert_closed_solid, bounds, frame, gpu, reflected, scaled, shells};
use fabelgeist_armor::{
    BuiltPart, DevicePart, GarmentArmorDesign, GarmentArmorKind, GarmentPlateShape, GenerateError,
    Millimeters, PartFrame, Permille, PlateFluting,
    gpu::{frame_words, record_fauld, record_garment_tube, record_tassets},
};
use fabelgeist_compute::KernelBatch;
use fabelgeist_compute::KernelBatchLabel;
use fabelgeist_gpu::prelude::Buffer;

/// Every garment built from its part frame alone. Torso garments and the
/// gorget are fitted to a wearer's body and tested with one.
const KINDS: [GarmentArmorKind; 8] = [
    GarmentArmorKind::Fauld,
    GarmentArmorKind::MailChausses,
    GarmentArmorKind::MailSkirt,
    GarmentArmorKind::MailSleeve,
    GarmentArmorKind::PaddedChausses,
    GarmentArmorKind::PaddedSkirt,
    GarmentArmorKind::QuiltedSleeve,
    GarmentArmorKind::Tassets,
];

type Recorder = fn(
    &fabelgeist_armor::ArmorGpu,
    &mut KernelBatch,
    &GarmentArmorDesign,
    &Buffer,
) -> Result<DevicePart, GenerateError>;

fn recorder(kind: GarmentArmorKind) -> Recorder {
    match kind {
        GarmentArmorKind::Fauld => record_fauld,
        GarmentArmorKind::Tassets => record_tassets,
        _ => record_garment_tube,
    }
}

fn fit(kind: GarmentArmorKind) -> PartFrame {
    let half_extents = match kind {
        GarmentArmorKind::MailSleeve | GarmentArmorKind::QuiltedSleeve => [0.060, 0.280, 0.060],
        GarmentArmorKind::MailChausses | GarmentArmorKind::PaddedChausses => [0.100, 0.430, 0.110],
        _ => [0.180, 0.120, 0.140],
    };
    frame([0.0, 1.0, 0.0], half_extents)
}

fn build_with(
    design: &GarmentArmorDesign,
    fit: &PartFrame,
    record: Recorder,
) -> Result<BuiltPart, GenerateError> {
    gpu().build_in(&[*fit], |batch, frames| {
        record(gpu(), batch, design, frames[0])
    })
}

fn build(design: &GarmentArmorDesign, fit: &PartFrame) -> Result<BuiltPart, GenerateError> {
    build_with(design, fit, recorder(design.kind))
}

#[test]
fn every_garment_has_closed_consistently_wound_thickness() {
    for kind in KINDS {
        let design = GarmentArmorDesign::new(kind);
        let mesh = build(&design, &fit(kind)).unwrap_or_else(|e| panic!("{kind:?}: {e}"));
        assert_closed_solid(&mesh, &format!("{kind:?}"));
        let again = build(&design, &fit(kind)).unwrap();
        assert_eq!(mesh.positions, again.positions);
        assert_eq!(mesh.indices, again.indices);
    }
}

#[test]
fn supported_extreme_parameters_keep_solid_topology_on_small_and_large_frames() {
    for kind in KINDS {
        for large in [false, true] {
            let mut design = GarmentArmorDesign::new(kind);
            design.clearance = Millimeters(if large { 40 } else { 1 });
            design.wall_thickness = Millimeters(if large { 16 } else { 1 });
            design.length = Permille(if large { 1_300 } else { 500 });
            design.flare = Permille(if large { 500 } else { 0 });
            design.waist = Permille(if large { 1_100 } else { 800 });
            design.lame_count = if large { 8 } else { 1 };
            let placed = scaled(fit(kind), if large { 1.3 } else { 0.7 });
            let mesh =
                build(&design, &placed).unwrap_or_else(|e| panic!("{kind:?}, large={large}: {e}"));
            assert_closed_solid(&mesh, &format!("{kind:?}, large={large}"));
        }
    }
}

#[test]
fn reflected_anatomical_frames_keep_outward_winding() {
    for kind in KINDS {
        let mut placed = reflected(fit(kind));
        placed.origin = [0.3, 1.1, -0.2];
        let mesh = build(&GarmentArmorDesign::new(kind), &placed).unwrap();
        assert_closed_solid(&mesh, &format!("{kind:?}, reflected"));
    }
}

#[test]
fn garment_edits_retain_vertex_correspondence_unless_plate_count_changes() {
    for kind in KINDS {
        let mut design = GarmentArmorDesign::new(kind);
        let first = build(&design, &fit(kind)).unwrap();
        design.length = Permille(800);
        design.flare = Permille(350);
        design.waist = Permille(1_050);
        let second = build(&design, &fit(kind)).unwrap();
        assert_eq!(first.indices, second.indices, "{kind:?}");
        assert_ne!(first.positions, second.positions, "{kind:?}");
        design.lame_count += 1;
        let relamed = build(&design, &fit(kind)).unwrap();
        let plated = matches!(kind, GarmentArmorKind::Fauld | GarmentArmorKind::Tassets);
        assert_eq!(relamed.indices != second.indices, plated, "{kind:?}");
    }
}

#[test]
fn length_extends_a_sleeve_along_the_limb() {
    let kind = GarmentArmorKind::MailSleeve;
    let length = |permille| {
        let mut design = GarmentArmorDesign::new(kind);
        design.length = Permille(permille);
        common::extent(&build(&design, &fit(kind)).unwrap().positions, 1)
    };
    assert!(length(1_200) > length(800) + 0.1);
}

#[test]
fn invalid_parameters_and_frames_are_rejected() {
    let mut design = GarmentArmorDesign::new(GarmentArmorKind::Fauld);
    design.lame_count = 0;
    assert!(design.validate().is_err());
    assert!(build(&design, &fit(design.kind)).is_err());
    design.lame_count = 4;
    let mut skewed = fit(design.kind);
    skewed.axes[0] = skewed.axes[1];
    assert!(build(&design, &skewed).is_err());
    let mut unfluted = GarmentArmorDesign::new(GarmentArmorKind::MailSleeve);
    unfluted.fluting = Some(PlateFluting::default());
    assert!(build(&unfluted, &fit(unfluted.kind)).is_err());
}

#[test]
fn plate_builders_reject_other_garments() {
    let tassets = GarmentArmorDesign::new(GarmentArmorKind::Tassets);
    let fauld = GarmentArmorDesign::new(GarmentArmorKind::Fauld);
    assert!(build_with(&tassets, &fit(tassets.kind), record_fauld).is_err());
    assert!(build_with(&fauld, &fit(fauld.kind), record_tassets).is_err());
}

#[test]
fn lame_count_makes_separate_overlapping_plates() {
    for count in [1, 3, 8] {
        for (kind, sides) in [(GarmentArmorKind::Fauld, 1), (GarmentArmorKind::Tassets, 2)] {
            let mut design = GarmentArmorDesign::new(kind);
            design.lame_count = count;
            let mesh = build(&design, &fit(kind)).unwrap();
            let plates = shells(&mesh);
            assert_eq!(plates.len(), usize::from(count) * sides, "{kind:?}");
            // Each lame overlaps the one below it, as a lamed skirt does.
            let mut heights = plates
                .iter()
                .map(|plate| {
                    let points = plate
                        .iter()
                        .flat_map(|t| &mesh.indices[t * 3..t * 3 + 3])
                        .map(|&i| &mesh.positions[i as usize]);
                    let [low, high] = bounds(points);
                    [low[1], high[1]]
                })
                .collect::<Vec<_>>();
            heights.sort_by(|a, b| b[1].total_cmp(&a[1]));
            for side in heights.chunks(sides) {
                assert!(
                    side.windows(2)
                        .all(|pair| (pair[0][1] - pair[1][1]).abs() < 1e-4),
                    "{kind:?}: paired lames at different heights"
                );
            }
            let rows = heights.iter().step_by(sides).collect::<Vec<_>>();
            for pair in rows.windows(2) {
                assert!(
                    pair[0][0] < pair[1][1] - 1e-4,
                    "{kind:?}: adjacent lames must overlap"
                );
            }
        }
    }
}

/// Each lame's carrier points, row by row, read back from the device.
fn tasset_carriers(design: &GarmentArmorDesign, fit: &PartFrame) -> Vec<Vec<[f32; 3]>> {
    let frame = gpu()
        .upload(BufferUpload::from_elements(&frame_words(fit)))
        .unwrap();
    let mut batch = gpu().batch(KernelBatchLabel::from("tasset carriers"));
    let mut part = record_tassets(gpu(), &mut batch, design, &frame).unwrap();
    part.record_shells(gpu(), &mut batch).unwrap();
    batch.submit();
    let carriers: Vec<[f32; 3]> = gpu().read(part.carriers()).unwrap();
    (0..part.shell_count())
        .map(|shell| {
            let range = part.shell_carriers(shell);
            carriers[range.start as usize..range.end as usize].to_vec()
        })
        .collect()
}

#[test]
fn tasset_inner_cutaways_and_rounded_hems_do_not_reverse_narrow_lames() {
    // Rows of a lame run up its length.
    const ROW_LINES: usize = 9;
    for count in [3, 8] {
        for cutaway in [0, 400] {
            let mut design = GarmentArmorDesign::new(GarmentArmorKind::Tassets);
            design.lame_count = count;
            if let GarmentPlateShape::Tassets {
                inner_cutaway,
                hem_roundness,
                hem_point,
                ..
            } = &mut design.plate_shape
            {
                *inner_cutaway = Permille(cutaway);
                *hem_roundness = Permille(300);
                *hem_point = Permille(200);
            }
            let lames = tasset_carriers(&design, &fit(design.kind));
            assert_eq!(lames.len(), usize::from(count) * 2);
            for points in lames {
                assert_eq!(points.len() % ROW_LINES, 0);
                let stride = points.len() / ROW_LINES;
                for row in 1..ROW_LINES {
                    for col in 0..stride {
                        assert!(
                            points[row * stride + col][1] > points[(row - 1) * stride + col][1],
                            "lame folded back along its length"
                        );
                    }
                }
            }
        }
    }
}
