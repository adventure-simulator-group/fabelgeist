//! Reject malformed compact geometry before indexing or source compilation.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum SupportSurfaceIssue {
    #[error("invalid property identity or member bindings")]
    Members,
    #[error("accepted treatment does not match the property members")]
    Treatment,
    #[error("invalid limits, regions or clipping outlines")]
    Bounds,
    #[error("invalid support vertices or triangle indices")]
    Topology,
    #[error("support grade {measured_grade} exceeds {permitted_grade} m/m")]
    Grade {
        measured_grade: f32,
        permitted_grade: f32,
    },
}

impl PropertySupportSurface {
    pub(crate) fn validate_encoded(&self) -> Result<(), SupportSurfaceIssue> {
        let members = &self.mesh.member_building_ids;
        if self.property_id().0 == 0
            || self.property_id().0 > crate::city_layout::MAX_CITY_LOTS as u64
            || members.is_empty()
            || members.len() > 2
            || members.contains(&0)
            || members
                .iter()
                .enumerate()
                .any(|(i, id)| members[..i].contains(id))
        {
            return Err(SupportSurfaceIssue::Members);
        }
        let treatment_matches_members = match self.treatment {
            SupportGradingAttempt::SingleBuildingFloorAndEntrances => members.len() == 1,
            SupportGradingAttempt::Compound(_) => members.len() == 2,
            SupportGradingAttempt::NotSelected => false,
        };
        if !treatment_matches_members {
            return Err(SupportSurfaceIssue::Treatment);
        }
        if SupportLimits::new(
            self.limits.maximum_grade,
            self.limits.maximum_displacement_metres,
            self.limits.contact_tolerance_metres,
        ) != Some(self.limits)
            || self.mesh.contact_tolerance_metres != self.limits.contact_tolerance_metres
            || self.regions.is_empty()
            || self.regions.len() != self.clipping_outlines.len()
            || self.regions.iter().any(|region| !region.is_valid())
            || self
                .clipping_outlines
                .iter()
                .any(|outline| !valid_outline(outline))
            || self
                .regions
                .iter()
                .zip(&self.clipping_outlines)
                .any(|(region, outline)| {
                    outline
                        .iter()
                        .any(|point| !region.contains(point.as_vec2()))
                })
        {
            return Err(SupportSurfaceIssue::Bounds);
        }
        if self.mesh.positions.is_empty()
            || self.mesh.positions.iter().any(|p| !p.is_finite())
            || self.mesh.support_triangles.is_empty()
            || self
                .mesh
                .support_triangles
                .iter()
                .chain(&self.mesh.retaining_triangles)
                .any(|indices| {
                    indices
                        .iter()
                        .any(|i| *i as usize >= self.mesh.positions.len())
                        || indices[0] == indices[1]
                        || indices[1] == indices[2]
                        || indices[2] == indices[0]
                })
            || self.mesh.support_triangles.iter().any(|indices| {
                let [a, b, c] = indices.map(|i| self.mesh.positions[i as usize]);
                let normal = (b - a).cross(c - a);
                !normal.is_finite() || normal.y.abs() <= f32::EPSILON
            })
        {
            return Err(SupportSurfaceIssue::Topology);
        }
        if self.mesh.positions.iter().any(|point| {
            !self
                .clipping_outlines
                .iter()
                .any(|outline| contains(outline, Vec2::new(point.x, point.z)))
        }) {
            return Err(SupportSurfaceIssue::Bounds);
        }
        let measured_grade = self.mesh.maximum_grade();
        if measured_grade > self.limits.maximum_grade {
            return Err(SupportSurfaceIssue::Grade {
                measured_grade,
                permitted_grade: self.limits.maximum_grade,
            });
        }
        Ok(())
    }
}

fn valid_outline(outline: &[bevy::math::DVec2]) -> bool {
    outline.len() >= 3
        && outline.iter().all(|p| p.is_finite())
        && planar::signed_area(outline) > f64::EPSILON
        && (0..outline.len()).all(|i| {
            let a = outline[i];
            let edge = outline[(i + 1) % outline.len()] - a;
            edge.length_squared() > f64::EPSILON
                && outline.iter().all(|point| {
                    edge.perp_dot(*point - a)
                        >= -edge.length() * CityPlotBounds::COORDINATE_TOLERANCE_METRES
                })
        })
}

fn contains(outline: &[bevy::math::DVec2], point: Vec2) -> bool {
    (0..outline.len()).all(|i| {
        let a = outline[i];
        let edge = outline[(i + 1) % outline.len()] - a;
        edge.perp_dot(point.as_dvec2() - a)
            >= -edge.length() * CityPlotBounds::COORDINATE_TOLERANCE_METRES
    })
}

#[cfg(test)]
mod tests;
