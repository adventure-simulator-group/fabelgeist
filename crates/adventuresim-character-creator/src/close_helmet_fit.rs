//! Fit independent cranial, jaw and neck sections to the reference wearer.
use crate::armor_frames::{FitRegion, Wearer};
use crate::armor_layer::ArmorLayerSurface;
use adventuresim_armor_model::{
    CloseHelmetDesign, CloseHelmetProfile, PartFrame, PartMesh, generate_close_helmet,
};
use anyhow::{Context, Result};

pub fn fit(
    design: &CloseHelmetDesign,
    wearer: &Wearer<'_>,
    layers: &[ArmorLayerSurface<'_>],
) -> Result<PartMesh> {
    let frame = wearer.frame(FitRegion::Head)?;
    let mut support = wearer.support_indices(FitRegion::Head)?;
    support.extend(wearer.support_indices(FitRegion::Neck)?);
    support.sort_unstable();
    support.dedup();
    let samples: Vec<_> = support.iter().map(|i| wearer.positions[*i]).collect();
    let (fitted_design, profile) = fit_hem(
        design,
        &frame,
        &samples,
        wearer.positions,
        wearer.faces,
        layers,
    )?;
    Ok(generate_close_helmet(&fitted_design, &frame, &profile)?)
}

/// Neck length is the requested maximum below the jaw. A short neck or raised
/// plate stack may require a higher hem, while unconstrained wearers retain the
/// authored length. Check each millimetre so disjoint plate courses cannot
/// invalidate a monotonic-search assumption.
fn fit_hem(
    design: &CloseHelmetDesign,
    frame: &PartFrame,
    samples: &[[f32; 3]],
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
    layers: &[ArmorLayerSurface<'_>],
) -> Result<(CloseHelmetDesign, CloseHelmetProfile)> {
    adventuresim_armor_model::HelmetDesign::CloseHelmet(*design).validate()?;
    let mut last_error = None;
    for length in (0..=design.neck_length.0).rev() {
        let mut fitted = *design;
        fitted.neck_length = adventuresim_armor_model::Millimeters(length);
        match fitted_profile(&fitted, frame, samples, positions, faces, layers) {
            Ok(profile) => return Ok((fitted, profile)),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.context("no close helmet neck length was evaluated")?)
        .context("no supported close helmet hem clears the wearer and plate stack")
}

fn fitted_profile(
    design: &CloseHelmetDesign,
    frame: &PartFrame,
    samples: &[[f32; 3]],
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
    layers: &[ArmorLayerSurface<'_>],
) -> Result<CloseHelmetProfile> {
    let mut profile = CloseHelmetProfile::from_surface(
        design,
        frame,
        samples,
        positions,
        faces,
        adventuresim_armor_model::Millimeters(0),
    )?;
    let mut enclosing_surface = None;
    if !layers.is_empty() {
        let mut enclosing_positions = positions.to_vec();
        let mut enclosing_faces = faces.to_vec();
        for layer in layers {
            let offset = enclosing_positions.len() as u32;
            enclosing_positions.extend_from_slice(layer.positions);
            enclosing_faces.extend(layer.faces.iter().map(|face| face.map(|i| i + offset)));
        }
        let mut metal_fit = *design;
        const PLATE_SEPARATION_MM: u16 = 8;
        metal_fit.face_clearance = adventuresim_armor_model::Millimeters(PLATE_SEPARATION_MM);
        let enclosed = CloseHelmetProfile::from_surface(
            &metal_fit,
            frame,
            samples,
            &enclosing_positions,
            &enclosing_faces,
            // The sloping inner return reaches below the outer rim station.
            adventuresim_armor_model::Millimeters(design.fit.wall_thickness.0 + 1),
        )?;
        // A collar changes the lower enclosure, never the cranial bowl or ears.
        profile.neck_half_width = profile.neck_half_width.max(enclosed.neck_half_width);
        profile.submental_front = profile.submental_front.max(enclosed.submental_front);
        profile.throat_front = profile.throat_front.max(enclosed.throat_front);
        profile.nape_back = profile.nape_back.min(enclosed.nape_back);
        profile.nape_waist = profile.nape_waist.min(enclosed.nape_waist);
        enclosing_surface = Some((enclosing_positions, enclosing_faces));
    }
    // A raised plate stack can reach forward beneath the throat. Its lower
    // enclosure must continue up through the chin-to-neck transition instead
    // of making a tight inward pocket immediately above that plate edge.
    profile.submental_front = profile.submental_front.max(profile.throat_front);
    if let Some((positions, faces)) = enclosing_surface {
        profile.enclose_neck(
            design,
            frame,
            &positions,
            &faces,
            adventuresim_armor_model::Millimeters(design.fit.wall_thickness.0 + 1),
        )?;
    }
    profile.validate()?;
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::Millimeters;

    #[test]
    fn collar_enlarges_neck_enclosure_without_widening_skull_or_jaw() {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.085, 0.115, 0.105],
        };
        let ring = |y: f32, width: f32, front: f32, back: f32| {
            [
                [-width, y, 0.0],
                [width, y, 0.0],
                [0.0, y, front],
                [0.0, y, back],
            ]
        };
        let body = [
            ring(0.03, 0.08, 0.09, -0.10),
            ring(0.08, 0.06, 0.06, -0.08),
            ring(-0.09, 0.06, 0.095, -0.08),
            ring(-0.105, 0.06, 0.095, -0.08),
            ring(-0.114, 0.055, 0.07, -0.075),
            ring(-0.125, 0.05, 0.045, -0.071),
            ring(-0.138, 0.05, 0.04, -0.07),
        ]
        .concat();
        let surface = [
            ring(-0.18, 0.05, 0.04, -0.07),
            ring(0.02, 0.05, 0.04, -0.07),
        ]
        .concat();
        let faces = [
            [0, 2, 6],
            [0, 6, 4],
            [2, 1, 5],
            [2, 5, 6],
            [1, 3, 7],
            [1, 7, 5],
            [3, 0, 4],
            [3, 4, 7],
        ];
        let collar = [
            ring(-0.15, 0.08, 0.07, -0.10),
            ring(-0.12, 0.08, 0.07, -0.10),
        ]
        .concat();
        let layer = ArmorLayerSurface {
            positions: &collar,
            faces: &faces,
            relief: Millimeters(0),
        };
        let design = CloseHelmetDesign::default();
        // Head measurement has no neck-band vertices; the actual triangle
        // sections below must supply all neck dimensions.
        let body = body
            .into_iter()
            .filter(|point| point[1] >= -0.105)
            .collect::<Vec<_>>();
        let bare = fitted_profile(&design, &frame, &body, &surface, &faces, &[]).unwrap();
        let worn = fitted_profile(&design, &frame, &body, &surface, &faces, &[layer]).unwrap();
        assert!(worn.neck_half_width > bare.neck_half_width);
        assert!(worn.throat_front > bare.throat_front);
        assert!(worn.nape_back < bare.nape_back);
        assert_eq!(worn.skull_half_width, bare.skull_half_width);
        assert_eq!(worn.temple_half_width, bare.temple_half_width);
        assert_eq!(worn.jaw_half_width, bare.jaw_half_width);
        let before = generate_close_helmet(&design, &frame, &bare).unwrap();
        let after = generate_close_helmet(&design, &frame, &worn).unwrap();
        assert_eq!(before.indices, after.indices);
        assert_eq!(before.components, after.components);
    }

    #[test]
    fn requested_hem_is_preserved_or_shortened_to_the_largest_valid_plate_clearance() {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.085, 0.115, 0.105],
        };
        let ring = |height: f32, width: f32, front: f32, back: f32| {
            [
                [-width, height, 0.0],
                [width, height, 0.0],
                [0.0, height, front],
                [0.0, height, back],
            ]
        };
        let head = [
            ring(0.03, 0.08, 0.09, -0.10),
            ring(0.08, 0.06, 0.06, -0.08),
            ring(-0.105, 0.06, 0.095, -0.08),
        ]
        .concat();
        let body = [
            ring(-0.22, 0.05, 0.04, -0.07),
            ring(0.02, 0.05, 0.04, -0.07),
        ]
        .concat();
        let faces = [
            [0, 2, 6],
            [0, 6, 4],
            [2, 1, 5],
            [2, 5, 6],
            [1, 3, 7],
            [1, 7, 5],
            [3, 0, 4],
            [3, 4, 7],
        ];
        let plate = [
            ring(-0.20, 0.08, 0.16, -0.10),
            ring(-0.12, 0.08, 0.04, -0.10),
        ]
        .concat();
        let layer = ArmorLayerSurface {
            positions: &plate,
            faces: &faces,
            relief: Millimeters(0),
        };
        let design = CloseHelmetDesign {
            neck_length: Millimeters(35),
            face_clearance: Millimeters(12),
            ..Default::default()
        };
        let (roomy, roomy_profile) = fit_hem(&design, &frame, &head, &body, &faces, &[]).unwrap();
        assert_eq!(roomy.neck_length, design.neck_length);
        let (fitted, profile) = fit_hem(&design, &frame, &head, &body, &faces, &[layer]).unwrap();
        assert!(fitted.neck_length.0 > 0 && fitted.neck_length.0 < design.neck_length.0);
        assert!(profile.submental_front >= profile.throat_front);
        let mut lower = fitted;
        lower.neck_length.0 += 1;
        let layer = ArmorLayerSurface {
            positions: &plate,
            faces: &faces,
            relief: Millimeters(0),
        };
        assert!(fitted_profile(&lower, &frame, &head, &body, &faces, &[layer]).is_err());
        let reference = generate_close_helmet(&roomy, &frame, &roomy_profile).unwrap();
        let constrained = generate_close_helmet(&fitted, &frame, &profile).unwrap();
        assert_eq!(reference.indices, constrained.indices);
        constrained.normals().unwrap();
        let mut zero = design;
        zero.neck_length = Millimeters(0);
        let (zero, profile) = fit_hem(&zero, &frame, &head, &body, &faces, &[]).unwrap();
        let minimum = generate_close_helmet(&zero, &frame, &profile).unwrap();
        assert_eq!(reference.indices, minimum.indices);
        minimum.normals().unwrap();
        let impossible = [ring(-0.22, 0.08, 0.2, -0.10), ring(0.02, 0.08, 0.2, -0.10)].concat();
        let layer = ArmorLayerSurface {
            positions: &impossible,
            faces: &faces,
            relief: Millimeters(0),
        };
        assert!(fit_hem(&design, &frame, &head, &body, &faces, &[layer]).is_err());
    }
}
