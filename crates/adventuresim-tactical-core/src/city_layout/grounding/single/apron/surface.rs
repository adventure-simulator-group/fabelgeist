//! Generate an accepted ramp or discrete stair approach with a full landing.
use super::*;

pub(super) struct ApronSurfaceBounds {
    pub stairs: CourtStairLimits,
    pub limits: SupportLimits,
}

pub(super) struct ApronSurfaceGeometry {
    pub outer: Vec2,
    pub direction: Vec2,
    pub offset: Vec2,
    pub run: f32,
    pub landing: f32,
    pub source: [f32; 2],
    pub floor: SupportElevation,
}

impl ApronSurfaceGeometry {
    pub fn compile(
        self,
        owner: &PropertySupportSurface,
        bounds: ApronSurfaceBounds,
    ) -> Result<PropertySupportMesh, SupportDiagnostic> {
        let Self {
            outer,
            direction,
            offset,
            run,
            landing,
            source,
            floor,
        } = self;
        let width = offset.length() * 2.0;
        let reject = |constraint, point, measured, permitted| {
            owner.rejection(
                constraint,
                SupportBoundary::StreetLanding,
                point,
                measured,
                permitted,
            )
        };
        let stairs = bounds.stairs;
        let flight_run = run - landing;
        let across_grade = (source[0] - source[1]).abs() / width;
        let along_grade = source
            .iter()
            .map(|h| (floor.metres() - h).abs() / flight_run)
            .fold(0.0, f32::max);
        let mut mesh = PropertySupportMesh {
            positions: Vec::new(),
            support_triangles: Vec::new(),
            retaining_triangles: Vec::new(),
            ..owner.mesh.clone()
        };
        let edge_at = |distance: f32, heights: [f32; 2]| {
            let centre = outer - direction * distance;
            let points = [centre - offset, centre + offset];
            [0, 1].map(|i| Vec3::new(points[i].x, heights[i], points[i].y))
        };
        let start = edge_at(0.0, source);
        let end = edge_at(flight_run, [floor.metres(); 2]);
        if along_grade.hypot(across_grade) <= bounds.limits.maximum_grade {
            mesh.quad(
                [start[1], start[0], end[0], end[1]],
                SupportFaceRole::Bearing,
            )?;
        } else {
            let rise = source
                .iter()
                .map(|h| (floor.metres() - h).abs())
                .fold(0.0, f32::max);
            let count = (rise / stairs.maximum_riser_metres).ceil();
            let required = count * stairs.minimum_going_metres;
            if count > f32::from(u16::MAX)
                || required > flight_run + bounds.limits.contact_tolerance_metres
            {
                return Err(reject(
                    SupportConstraint::StairGoing,
                    outer,
                    required,
                    flight_run,
                ));
            }
            if across_grade > bounds.limits.maximum_grade {
                return Err(reject(
                    SupportConstraint::AccessGrade,
                    outer,
                    across_grade * width,
                    bounds.limits.maximum_grade * width,
                ));
            }
            for i in 0..count as u16 {
                let t = f32::from(i) / count;
                let next = f32::from(i + 1) / count;
                let heights = |t| source.map(|h| h + (floor.metres() - h) * t);
                let a = edge_at(flight_run * t, heights(t));
                let b = edge_at(flight_run * next, heights(t));
                let c = edge_at(flight_run * next, heights(next));
                mesh.quad([a[1], a[0], b[0], b[1]], SupportFaceRole::Bearing)?;
                mesh.quad([b[1], b[0], c[0], c[1]], SupportFaceRole::Retaining)?;
            }
        }
        let inside = edge_at(run, [floor.metres(); 2]);
        mesh.quad(
            [end[1], end[0], inside[0], inside[1]],
            SupportFaceRole::Bearing,
        )?;
        if mesh.maximum_grade() > bounds.limits.maximum_grade {
            return Err(reject(
                SupportConstraint::AccessGrade,
                outer,
                mesh.maximum_grade() * run,
                bounds.limits.maximum_grade * run,
            ));
        }
        Ok(mesh)
    }
}
