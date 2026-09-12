//! Meshing the measured collar and shoulder carrier with shared plate topology.
use super::CollarCage;
use super::bib_clearance::{self, BibVertex, GorgetVertex};
use adventuresim_armor_model::{GarmentArmorDesign, PartMesh, generate_gorget_plates};
use anyhow::{Result, ensure};
use bevy::math::Vec3;
use std::cell::RefCell;

impl CollarCage {
    pub(super) fn mesh(
        &self,
        design: &GarmentArmorDesign,
        body: &[[f32; 3]],
        faces: &[[u32; 3]],
    ) -> Result<PartMesh> {
        let adventuresim_armor_model::GarmentPlateShape::Gorget { neck_clearance, .. } =
            design.plate_shape
        else {
            anyhow::bail!("gorget mesh requires collar clearance parameters");
        };
        let collar_padding = neck_clearance.metres() + design.wall_thickness.metres();
        let chart = RefCell::new(Vec::new());
        let curve_failure = RefCell::new(None);
        let mesh = generate_gorget_plates(
            design,
            |t, angle| {
                chart
                    .borrow_mut()
                    .push(GorgetVertex::Collar(self.collar_clearance(
                        t,
                        angle,
                        collar_padding,
                    )));
                self.collar_point(t, angle)
            },
            |t, angle| {
                chart.borrow_mut().push(GorgetVertex::Bib(BibVertex {
                    direction: Vec3::from_array(adventuresim_armor_model::gorget_bib_direction(
                        self.surface_angle(angle),
                    )),
                    ceiling: self.bib_point(0.0, angle)[1],
                    collar_inset: self.collar_clearance(1.0, angle, collar_padding),
                    transition: super::meridian::clearance_blend(t),
                }));
                match self.formed_bib_point(t, angle) {
                    Ok(point) => point,
                    Err(error) => {
                        curve_failure.borrow_mut().get_or_insert(error);
                        [f32::NAN; 3]
                    }
                }
            },
        );
        if let Some(error) = curve_failure.into_inner() {
            return Err(error);
        }
        let mesh = mesh?;
        let chart = chart.into_inner();
        let mut cursor = 0;
        let mut failure = None;
        let fitted = mesh.refit_surfaces(|positions, indices| {
            let end = cursor + positions.len();
            if let Err(error) = bib_clearance::seat(
                positions,
                indices,
                &chart[cursor..end],
                body,
                faces,
                design.clearance.metres(),
                design.wall_thickness.metres(),
            ) {
                failure = Some(error);
            }
            cursor = end;
        });
        ensure!(
            cursor == chart.len(),
            "gorget chart does not match generated carriers"
        );
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(fitted?)
    }
}
