//! Exact clipping retains terraces across the thickness of every wall and post.
use super::*;

pub(super) fn project(
    property: &CityCompound,
    foundation: &PropertyFoundationMesh,
    limits: SupportLimits,
    embedment: FoundationEmbedment,
) -> Result<BoundarySupportProjection, BoundarySupportError> {
    if foundation.property_id != property.id
        || foundation.member_building_ids.len() != 2
        || ![property.front_building_id, property.rear_building_id]
            .iter()
            .all(|id| foundation.member_building_ids.contains(id))
    {
        return Err(BoundarySupportError::new(
            property,
            BoundarySupportElement::Owner,
            BoundarySupportConstraint::OwnerBinding,
            property.plot.centre_metres(),
            1.0,
            0.0,
        ));
    }
    let mut triangles: Vec<_> = foundation
        .support_triangles
        .iter()
        .filter_map(|indices| {
            GroundTriangle::new(indices.map(|i| foundation.positions[i as usize]))
        })
        .collect();
    triangles.sort_by(GroundTriangle::compare);
    let gate = gate_datum(property, &triangles, limits)?;
    let binding = BoundaryOwnerBinding::from_property(property, gate)
        .map_err(|cause| BoundarySupportError::binding(property, cause))?;
    let mut mesh = BoundarySupportMesh {
        binding,
        cells: Vec::new(),
    };
    for member in property
        .boundary
        .fixed_members()
        .map_err(|cause| BoundarySupportError::construction(property, cause))?
    {
        let element = member.pose.element();
        MemberProjection {
            property,
            triangles: &triangles,
            member,
            element,
            gate,
            limits,
            embedment,
        }
        .append(&mut mesh)?;
    }
    mesh.binding.validate_cells(&mesh.cells).map_err(|issue| {
        BoundarySupportError::cell(property, BoundarySupportElement::Owner, issue)
    })?;
    Ok(BoundarySupportProjection {
        mesh,
        gate_elevation: gate,
    })
}

fn gate_datum(
    property: &CityCompound,
    triangles: &[GroundTriangle],
    limits: SupportLimits,
) -> Result<SupportElevation, BoundarySupportError> {
    let point = property.boundary.gate.centre_metres;
    let mut heights: Vec<_> = triangles
        .iter()
        .filter(|t| t.contains(point, limits.contact_tolerance_metres.metres()))
        .map(|t| t.height_at(point))
        .collect();
    heights.sort_by(f32::total_cmp);
    let range = heights
        .first()
        .zip(heights.last())
        .map(|(low, high)| high - low);
    match range {
        Some(range) if range <= limits.contact_tolerance_metres.metres() => {
            Ok(SupportElevation(heights[0]))
        }
        Some(range) => Err(BoundarySupportError::new(
            property,
            BoundarySupportElement::GateLanding,
            BoundarySupportConstraint::AmbiguousGateDatum,
            point,
            f64::from(range),
            f64::from(limits.contact_tolerance_metres.metres()),
        )),
        None => Err(BoundarySupportError::new(
            property,
            BoundarySupportElement::GateLanding,
            BoundarySupportConstraint::MissingGateDatum,
            point,
            1.0,
            0.0,
        )),
    }
}

struct MemberProjection<'a> {
    property: &'a CityCompound,
    triangles: &'a [GroundTriangle],
    member: CityBoundaryMember,
    element: BoundarySupportElement,
    gate: SupportElevation,
    limits: SupportLimits,
    embedment: FoundationEmbedment,
}

impl MemberProjection<'_> {
    fn append(self, mesh: &mut BoundarySupportMesh) -> Result<(), BoundarySupportError> {
        let Self {
            property,
            triangles,
            member,
            element,
            gate: _,
            limits,
            embedment: _,
        } = self;
        let centre = member.pose.plan_metres().as_dvec2();
        let (sine, cosine) = f64::from(member.orientation.yaw_radians()).sin_cos();
        let tangent = DVec2::new(cosine, -sine);
        let inward = DVec2::new(sine, cosine);
        let half = member.size_metres.metres().xz().as_dvec2() * 0.5;
        let outline = [
            DVec2::NEG_ONE,
            DVec2::new(1.0, -1.0),
            DVec2::ONE,
            DVec2::new(-1.0, 1.0),
        ]
        .map(|p| centre + tangent * p.x * half.x + inward * p.y * half.y);
        let mut covered = 0.0;
        for triangle in triangles {
            let mut polygon = triangle.points().map(|p| p.xz().as_dvec2()).to_vec();
            for i in 0..outline.len() {
                let a = outline[i];
                let edge = outline[(i + 1) % outline.len()] - a;
                polygon = planar::clip(polygon, |p| edge.perp_dot(p - a));
            }
            let area = planar::signed_area(&polygon).abs();
            if area <= f64::EPSILON {
                continue;
            }
            covered += area;
            for i in 1..polygon.len().saturating_sub(1) {
                let points = [polygon[0], polygon[i], polygon[i + 1]];
                if planar::signed_area(&points).abs() <= f64::EPSILON {
                    continue;
                }
                mesh.cells.push(self.cell(triangle, points)?);
            }
        }
        let expected =
            f64::from(member.size_metres.metres().x) * f64::from(member.size_metres.metres().z);
        let permitted = 2.0
            * f64::from(member.size_metres.metres().x + member.size_metres.metres().z)
            * f64::from(limits.contact_tolerance_metres.metres());
        let discrepancy = (covered - expected).abs();
        if discrepancy > permitted {
            return Err(BoundarySupportError::new(
                property,
                element,
                BoundarySupportConstraint::Coverage,
                centre.as_vec2(),
                discrepancy,
                permitted,
            ));
        }
        Ok(())
    }
    fn cell(
        &self,
        triangle: &GroundTriangle,
        points: [DVec2; 3],
    ) -> Result<BoundarySupportCell, BoundarySupportError> {
        let mut top = [Position::<GateRelative>::ORIGIN; 3];
        let mut base = [Position::<GateRelative>::ORIGIN; 3];
        for (j, point) in points.into_iter().enumerate() {
            let soil = triangle.height_f64(point) as f32;
            let construction_error = |cause| {
                BoundarySupportError::construction(
                    self.property,
                    crate::city_layout::BoundaryGeometryError {
                        element: self.element,
                        cause,
                    },
                )
            };
            let levels = member_levels(self.member, soil, self.gate, self.embedment)
                .map_err(construction_error)?;
            if levels.head.metres() <= levels.base.metres() {
                return Err(BoundarySupportError::new(
                    self.property,
                    self.element,
                    BoundarySupportConstraint::PostHeadroom,
                    point.as_vec2(),
                    f64::from(levels.base.metres() - levels.head.metres()),
                    0.0,
                ));
            }
            top[j] = adventuresim_building_generator::spatial_geometry::Position::<
                crate::scene_coordinates::GateRelative,
            >::from_metres(Vec3::new(
                point.x as f32,
                levels.head.metres() - self.gate.metres(),
                point.y as f32,
            ))
            .map_err(construction_error)?;
            base[j] = adventuresim_building_generator::spatial_geometry::Position::<
                crate::scene_coordinates::GateRelative,
            >::from_metres(Vec3::new(
                point.x as f32,
                levels.base.metres() - self.gate.metres(),
                point.y as f32,
            ))
            .map_err(construction_error)?;
        }
        BoundarySupportCell::new(
            [top[0], top[1], top[2], base[0], base[1], base[2]],
            self.member.material,
            self.element,
        )
        .map_err(|cause| BoundarySupportError::cell(self.property, self.element, cause))
    }
}

struct MemberLevels {
    base: SupportElevation,
    head: SupportElevation,
}

fn member_levels(
    member: CityBoundaryMember,
    soil: f32,
    gate: SupportElevation,
    embedment: FoundationEmbedment,
) -> Result<MemberLevels, adventuresim_building_generator::spatial_geometry::GeometryError> {
    use crate::city_layout::CityBoundaryPose;
    let (centre, gate_relative, cap) = match member.pose {
        CityBoundaryPose::GatePost { centre, .. } => (centre.metres().y, true, false),
        CityBoundaryPose::Wall { centre, .. } => (centre.metres().y, false, false),
        CityBoundaryPose::WallCap { centre, .. } => (centre.metres().y, false, true),
    };
    let nominal_base = centre - member.size_metres.metres().y * 0.5;
    let nominal_head = centre + member.size_metres.metres().y * 0.5;
    let elevation = |metres| {
        SupportElevation::from_metres(metres).ok_or(
            adventuresim_building_generator::spatial_geometry::GeometryError::NonFinite {
                role: adventuresim_building_generator::spatial_geometry::GeometryRole::Elevation,
                axis: adventuresim_building_generator::spatial_geometry::CoordinateAxis::Y,
            },
        )
    };
    Ok(MemberLevels {
        base: elevation(if cap {
            soil + nominal_base
        } else {
            soil - embedment.metres()
        })?,
        head: elevation(if gate_relative {
            gate.metres() + nominal_head
        } else {
            soil + nominal_head
        })?,
    })
}
