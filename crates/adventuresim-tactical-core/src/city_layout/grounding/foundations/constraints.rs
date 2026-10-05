//! Complete source intersections bound an affine change to occupied floor levels.
use super::*;

#[derive(Clone, Copy, Debug)]
struct Control {
    point: Vec2,
    support: f64,
    source: f64,
    coefficient: f64,
}

pub(super) fn floor_shift(
    plan: &CompoundSupportPlan,
    geographic: &GeographicSurface,
) -> Result<f32, SupportDiagnostic> {
    let original = plan.mesh()?;
    let basis = plan.floor_translation_basis()?.mesh()?;
    let invalid_triangle = |point| {
        SupportDiagnostic::new(
            &plan.property,
            SupportConstraint::Reservation,
            SupportBoundary::PropertyReservation,
            point,
            1.0,
            0.0,
        )
    };
    let mut interval = FloorInterval {
        minimum: f64::NEG_INFINITY,
        maximum: f64::INFINITY,
        upper_control: None,
    };
    for (indices, shifted) in original
        .support_triangles
        .iter()
        .zip(&basis.support_triangles)
    {
        let support = GroundTriangle::new(indices.map(|i| original.positions[i as usize]))
            .ok_or_else(|| invalid_triangle(original.positions[indices[0] as usize].xz()))?;
        let shifted = GroundTriangle::new(shifted.map(|i| basis.positions[i as usize]))
            .ok_or_else(|| invalid_triangle(basis.positions[shifted[0] as usize].xz()))?;
        let mut covered = 0.0;
        for source in geographic.intersecting(&support) {
            let polygon = support.intersection(source);
            if polygon.len() < 3 || geometry::area(&polygon) <= f64::EPSILON {
                continue;
            }
            covered += geometry::area(&polygon);
            for point in polygon {
                let height = support.height_f64(point.as_dvec2());
                interval.constrain(
                    plan,
                    Control {
                        point,
                        support: height,
                        source: source.height_f64(point.as_dvec2()),
                        coefficient: shifted.height_f64(point.as_dvec2()) - height,
                    },
                )?;
            }
        }
        validate_coverage(&plan.support_surface()?, &support, covered)?;
    }
    interval.choose(plan)
}

struct FloorInterval {
    minimum: f64,
    maximum: f64,
    upper_control: Option<Control>,
}

impl FloorInterval {
    fn constrain(
        &mut self,
        plan: &CompoundSupportPlan,
        control: Control,
    ) -> Result<(), SupportDiagnostic> {
        let displacement = control.support - control.source;
        let permitted = f64::from(plan.limits.maximum_displacement_metres);
        if control.coefficient.abs() <= f64::from(f32::EPSILON) {
            if displacement.abs() > permitted {
                return Err(rejection(plan, control, 0.0));
            }
            return Ok(());
        }
        let a = (-permitted - displacement) / control.coefficient;
        let b = (permitted - displacement) / control.coefficient;
        self.minimum = self.minimum.max(a.min(b));
        if a.max(b) < self.maximum {
            self.maximum = a.max(b);
            self.upper_control = Some(control);
        }
        Ok(())
    }

    fn choose(self, plan: &CompoundSupportPlan) -> Result<f32, SupportDiagnostic> {
        if self.minimum > self.maximum {
            let control = self.upper_control.ok_or_else(|| {
                SupportDiagnostic::new(
                    &plan.property,
                    SupportConstraint::Reservation,
                    SupportBoundary::GeographicSurface,
                    plan.property.plot.centre_metres,
                    1.0,
                    0.0,
                )
            })?;
            return Err(rejection(plan, control, self.minimum));
        }
        let margin = f64::from(plan.limits.contact_tolerance_metres);
        let shift = if self.minimum + margin <= self.maximum - margin {
            0.0_f64.clamp(self.minimum + margin, self.maximum - margin)
        } else {
            (self.minimum + self.maximum) * 0.5
        };
        Ok(shift as f32)
    }
}

fn rejection(plan: &CompoundSupportPlan, control: Control, shift: f64) -> SupportDiagnostic {
    let measured = (control.support + control.coefficient * shift - control.source).abs() as f32;
    let mut error = SupportDiagnostic::new(
        &plan.property,
        SupportConstraint::CutFill,
        SupportBoundary::GeographicSurface,
        control.point,
        measured,
        plan.limits.maximum_displacement_metres,
    );
    error.attempted_treatment = SupportGradingAttempt::Compound(plan.treatment);
    error
}
