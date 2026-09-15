//! Roof trusses share the covering planes and their physical underside.
use super::*;

const FRAME_COVER_CLEARANCE_FACTOR: f32 = 0.525;
const MINIMUM_KNEE_LENGTH_METRES: f32 = 0.05;

pub(super) fn build(
    builder: &mut TimberFrameBuilder<'_>,
    roofs: &[RoofPiece],
    roof_assemblies: &[RoofAssembly],
    dormers: &[RoofDormer],
    top: f32,
    section: Vec2,
) {
    if let Some(roof) = roofs.first() {
        let half_width = if roof.ridge_axis == RidgeAxis::X {
            roof.size.y * 0.5
        } else {
            roof.size.x * 0.5
        };
        let ridge_tangent = if roof.ridge_axis == RidgeAxis::X {
            Vec2::X
        } else {
            Vec2::Y
        };
        let gable_tangent = Vec2::new(-ridge_tangent.y, ridge_tangent.x);
        let half_length = if roof.ridge_axis == RidgeAxis::X {
            roof.size.x * 0.5
        } else {
            roof.size.y * 0.5
        };
        let frame_count = ((half_length * 2.0) / 1.80).ceil().max(1.0) as usize;
        let mut roof_frames = Vec::new();
        for frame_index in 0..=frame_count {
            let along = -half_length + half_length * 2.0 * frame_index as f32 / frame_count as f32;
            let gable_centre = roof.centre + ridge_tangent * along;
            if frame_index != 0
                && frame_index != frame_count
                && dormers.iter().any(|dormer| {
                    (dormer.centre - gable_centre).dot(ridge_tangent).abs()
                        <= dormer.width_metres * 0.5 + 0.40
                })
            {
                // The child roof owns its cut and four-sided trimmer frame;
                // a regular parent truss may not continue through that cut.
                continue;
            }
            let truss = Truss::new(
                roof,
                roof_assemblies,
                gable_centre,
                gable_tangent,
                half_width,
                half_length,
                along,
                top,
                section,
            );
            roof_frames.push(truss.build(builder, top, section));
        }
        for pair in roof_frames.windows(2) {
            for (left, right) in [
                (pair[0].0, pair[1].0),
                (pair[0].1, pair[1].1),
                (pair[0].2, pair[1].2),
            ] {
                builder.member(
                    crate::TimberMemberRole::Purlin,
                    left,
                    right,
                    section * 1.05,
                    crate::TimberFramePhase::RoofConstruction,
                );
            }
        }
    }
}

struct Truss {
    left_base: Vec3,
    right_base: Vec3,
    left_rafter: Vec3,
    right_rafter: Vec3,
    apex: Vec3,
}

impl Truss {
    fn new(
        roof: &RoofPiece,
        roof_assemblies: &[RoofAssembly],
        gable_centre: Vec2,
        gable_tangent: Vec2,
        half_width: f32,
        half_length: f32,
        along: f32,
        top: f32,
        section: Vec2,
    ) -> Self {
        let rise = half_width * roof.pitch_degrees.to_radians().tan();
        let left = gable_centre - gable_tangent * half_width;
        let right = gable_centre + gable_tangent * half_width;
        // A half-hip does not have the full ridge elevation at its end
        // frames.  The former full-height A-frame recipe was structurally
        // grounded but projected through the two upper hip faces.  Match
        // the Stage 4 half-hip construction: the retained lower gable
        // reaches 55% of the rise at the end, then the frame apex climbs
        // along the short hip to the main ridge.
        let station_rise = if roof.kind == RoofKind::HalfHip {
            let hip_run = (half_width * 0.45).max(0.001);
            let distance_from_end = (half_length - along.abs()).max(0.0);
            rise * (0.55 + 0.45 * (distance_from_end / hip_run).clamp(0.0, 1.0))
        } else {
            rise
        };
        let roof_seat = |point: Vec2, half_hip_y: f32| {
            if roof.kind != RoofKind::Gable {
                return half_hip_y;
            }
            roof_assemblies[0]
                .faces
                .iter()
                .map(|face| {
                    face.underside_height_at(point)
                        - section.max_element() * FRAME_COVER_CLEARANCE_FACTOR
                            / face.plane.normal.normalize().y
                })
                .fold(f32::INFINITY, f32::min)
        };
        let apex = Vec3::new(
            gable_centre.x,
            roof_seat(gable_centre, top + station_rise),
            gable_centre.y,
        );
        let left_base = Vec3::new(left.x, top, left.y);
        let right_base = Vec3::new(right.x, top, right.y);
        let left_rafter = Vec3::new(left.x, roof_seat(left, top).max(top), left.y);
        let right_rafter = Vec3::new(right.x, roof_seat(right, top).max(top), right.y);

        Self {
            left_base,
            right_base,
            left_rafter,
            right_rafter,
            apex,
        }
    }

    fn build(
        self,
        builder: &mut TimberFrameBuilder<'_>,
        top: f32,
        section: Vec2,
    ) -> (Vec3, Vec3, Vec3) {
        let Self {
            left_base,
            right_base,
            left_rafter,
            right_rafter,
            apex,
        } = self;
        let gable_centre = Vec2::new(apex.x, apex.z);
        for (base, seat) in [(left_base, left_rafter), (right_base, right_rafter)] {
            if seat.y - base.y > MINIMUM_KNEE_LENGTH_METRES {
                builder.member(
                    crate::TimberMemberRole::GablePost,
                    base,
                    seat,
                    section,
                    crate::TimberFramePhase::RoofConstruction,
                );
            }
        }
        builder.member(
            crate::TimberMemberRole::GableTie,
            left_base,
            right_base,
            section,
            crate::TimberFramePhase::RoofConstruction,
        );
        builder.member(
            crate::TimberMemberRole::GablePost,
            Vec3::new(gable_centre.x, top, gable_centre.y),
            apex,
            section,
            crate::TimberFramePhase::RoofConstruction,
        );
        let collar_left = left_rafter.lerp(apex, 0.58);
        let collar_right = right_rafter.lerp(apex, 0.58);
        for (base, collar) in [(left_rafter, collar_left), (right_rafter, collar_right)] {
            builder.member(
                crate::TimberMemberRole::Rafter,
                base,
                collar,
                section * 0.9,
                crate::TimberFramePhase::RoofConstruction,
            );
            builder.member(
                crate::TimberMemberRole::Rafter,
                collar,
                apex,
                section * 0.9,
                crate::TimberFramePhase::RoofConstruction,
            );
        }
        builder.member(
            crate::TimberMemberRole::Collar,
            collar_left,
            collar_right,
            section * 0.82,
            crate::TimberFramePhase::RoofConstruction,
        );

        (collar_left, apex, collar_right)
    }
}
