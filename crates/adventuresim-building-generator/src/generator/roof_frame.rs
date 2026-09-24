//! Roof trusses share the covering planes and their physical underside.
use super::*;

const FRAME_COVER_CLEARANCE_FACTOR: f32 = 0.525;
const MINIMUM_KNEE_LENGTH_METRES: f32 = 0.05;
const COLLAR_SECTION_FACTOR: f32 = 0.82;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shallow_wide_gables_preserve_continuous_members_at_nearby_joints() {
        for width in [8, 10, 12] {
            for pitch in [15.0, 30.0, 37.0, 55.0, 75.0] {
                let mut program = BuildingProgram::fixture(BuildingArchetype::StorageRange, 42);
                program.footprint = Footprint::Rectangle { width, depth: 4 };
                program.roof_pitch_degrees = pitch;
                let plan = crate::generate(&program)
                    .unwrap_or_else(|error| panic!("{width} cells at {pitch} degrees: {error:?}"));
                for member in &plan.timber_frame.as_ref().unwrap().members {
                    assert!(
                        member.start.distance(member.end)
                            > crate::MINIMUM_TIMBER_MEMBER_LENGTH_METRES
                    );
                }
            }
        }
    }
}

/// Retain the full tie-to-rake span when two subdivision joints nearly coincide.
/// The common bearing pass resolves contacts on the resulting continuous timber.
fn separated_joints(points: &[Vec3]) -> Vec<Vec3> {
    const JOINT_MERGE_TOLERANCE_METRES: f32 = 0.001;
    let minimum = crate::MINIMUM_TIMBER_MEMBER_LENGTH_METRES + JOINT_MERGE_TOLERANCE_METRES;
    let (Some(&first), Some(&last)) = (points.first(), points.last()) else {
        return Vec::new();
    };
    if first.distance(last) <= minimum {
        return Vec::new();
    }
    let mut result = vec![first];
    for &point in &points[1..points.len() - 1] {
        if point.distance(last) > minimum && point.distance(*result.last().unwrap()) > minimum {
            result.push(point);
        }
    }
    result.push(last);
    result
}

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
            let gable_centre = end_truss_centre(
                roof,
                roof_assemblies,
                frame_index,
                frame_count,
                roof.centre + ridge_tangent * along,
                ridge_tangent,
                section,
            );
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
            if roof.kind == RoofKind::Gable && (frame_index == 0 || frame_index == frame_count) {
                truss.frame_infill(builder, section);
            }
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

/// End trusses follow the upper wall envelope, including a jettied gable.
/// Their exposed faces sit just proud of infill, as on the storeys below.
fn end_truss_centre(
    roof: &RoofPiece,
    assemblies: &[RoofAssembly],
    index: usize,
    count: usize,
    nominal: Vec2,
    ridge_tangent: Vec2,
    section: Vec2,
) -> Vec2 {
    if roof.kind != RoofKind::Gable || (index != 0 && index != count) {
        return nominal;
    }
    let outward = ridge_tangent * if index == 0 { -1.0 } else { 1.0 };
    let Some(face) = assemblies[0].enclosure_faces.iter().find(|face| {
        face.material == RoofMaterial::TimberInfill
            && face.polygon.len() >= 3
            && crate::gable_frame::normal(face).dot(Vec3::new(outward.x, 0.0, outward.y)) > 0.99
    }) else {
        return nominal;
    };
    let point = face.polygon[0];
    let face_plane = Vec2::new(point.x, point.z).dot(outward);
    // Collars have the smallest section in this truss; they too must be visible.
    let centre_plane = face_plane - section.min_element() * COLLAR_SECTION_FACTOR * 0.5
        + crate::TIMBER_INFILL_FINISH_SETBACK_METRES;
    nominal + outward * (centre_plane - nominal.dot(outward))
}

#[derive(Clone, Copy)]
struct Truss {
    left_base: Vec3,
    right_base: Vec3,
    left_rafter: Vec3,
    right_rafter: Vec3,
    apex: Vec3,
}

impl Truss {
    /// Subdivide the end wall into timber-supported infill panels. Interior
    /// roof trusses remain open; only the two enclosing gables need this frame.
    fn frame_infill(self, builder: &mut TimberFrameBuilder<'_>, section: Vec2) {
        const MAXIMUM_INFILL_BAY_WIDTH_METRES: f32 = 1.8;
        const MAXIMUM_INFILL_PANEL_HEIGHT_METRES: f32 = 1.5;
        let centre = (self.left_base + self.right_base) * 0.5;
        let collar_height = self.left_rafter.lerp(self.apex, 0.58).y;
        let mut levels = vec![collar_height];
        for (low, high) in [(centre.y, collar_height), (collar_height, self.apex.y)] {
            let rows = ((high - low) / MAXIMUM_INFILL_PANEL_HEIGHT_METRES).ceil() as usize;
            levels.extend((1..rows).map(|row| low + (high - low) * row as f32 / rows as f32));
        }
        levels.sort_by(f32::total_cmp);
        for (base, rake) in [
            (self.left_base, self.left_rafter),
            (self.right_base, self.right_rafter),
        ] {
            let bays = (base.distance(centre) / MAXIMUM_INFILL_BAY_WIDTH_METRES).ceil() as usize;
            let mut posts = Vec::new();
            for station in 1..bays {
                let fraction = station as f32 / bays as f32;
                let foot = centre.lerp(base, fraction);
                let head = self.apex.lerp(rake, fraction);
                let mut joints = vec![foot];
                for &elevation in &levels {
                    if elevation > foot.y && elevation < head.y {
                        joints.push(foot.with_y(elevation));
                    }
                }
                joints.push(head);
                joints.sort_by(|a, b| a.y.total_cmp(&b.y));
                for pair in separated_joints(&joints).windows(2) {
                    builder.member(
                        crate::TimberMemberRole::GablePost,
                        pair[0],
                        pair[1],
                        section * COLLAR_SECTION_FACTOR,
                        crate::TimberFramePhase::RoofConstruction,
                    );
                }
                posts.push((foot, head));
            }
            for &elevation in &levels {
                // The structural collar already occupies this row.
                if elevation == collar_height || elevation <= rake.y {
                    continue;
                }
                let mut stations = vec![centre.with_y(elevation)];
                stations.extend(
                    posts
                        .iter()
                        .filter(|(_, head)| head.y > elevation)
                        .map(|(foot, _)| foot.with_y(elevation)),
                );
                stations.push(rake.lerp(self.apex, (elevation - rake.y) / (self.apex.y - rake.y)));
                for pair in separated_joints(&stations).windows(2) {
                    builder.member(
                        crate::TimberMemberRole::Rail,
                        pair[0],
                        pair[1],
                        section * COLLAR_SECTION_FACTOR,
                        crate::TimberFramePhase::RoofConstruction,
                    );
                }
            }
        }
    }

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
            section * COLLAR_SECTION_FACTOR,
            crate::TimberFramePhase::RoofConstruction,
        );

        (collar_left, apex, collar_right)
    }
}
