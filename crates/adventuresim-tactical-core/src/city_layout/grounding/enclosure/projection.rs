//! Exact clipping retains terraces across the thickness of every wall and post.
use super::*;

pub(super) fn project(
    property: &CityCompound,
    foundation: &PropertyFoundationMesh,
    limits: SupportLimits,
    embedment: FoundationEmbedment,
) -> Result<(BoundarySupportMesh, SupportElevation), BoundarySupportError> {
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
            property.plot.centre_metres,
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
    let mut mesh = BoundarySupportMesh { cells: Vec::new() };
    let kinds = (0..property.boundary.walls.len())
        .flat_map(|i| {
            [
                BoundarySupportElement::Wall(i),
                BoundarySupportElement::WallCap(i),
            ]
        })
        .chain([
            BoundarySupportElement::GatePost(PropertySide::Left),
            BoundarySupportElement::GatePost(PropertySide::Right),
        ]);
    for (member, element) in property.boundary.fixed_members().into_iter().zip(kinds) {
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
    Ok((mesh, gate))
}

fn gate_datum(
    property: &CityCompound,
    triangles: &[GroundTriangle],
    limits: SupportLimits,
) -> Result<SupportElevation, BoundarySupportError> {
    let point = property.boundary.gate.centre_metres;
    let mut heights: Vec<_> = triangles
        .iter()
        .filter(|t| t.contains(point, limits.contact_tolerance_metres))
        .map(|t| t.height_at(point))
        .collect();
    heights.sort_by(f32::total_cmp);
    let range = heights
        .first()
        .zip(heights.last())
        .map(|(low, high)| high - low);
    match range {
        Some(range) if range <= limits.contact_tolerance_metres => Ok(SupportElevation(
            *heights.first().expect("measured gate support"),
        )),
        Some(range) => Err(BoundarySupportError::new(
            property,
            BoundarySupportElement::GateLanding,
            BoundarySupportConstraint::AmbiguousGateDatum,
            point,
            f64::from(range),
            f64::from(limits.contact_tolerance_metres),
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
        let centre = member.centre_metres.xz().as_dvec2();
        let (sine, cosine) = f64::from(member.yaw_radians).sin_cos();
        let tangent = DVec2::new(cosine, -sine);
        let inward = DVec2::new(sine, cosine);
        let half = member.size_metres.xz().as_dvec2() * 0.5;
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
        let expected = f64::from(member.size_metres.x) * f64::from(member.size_metres.z);
        let permitted = 2.0
            * f64::from(member.size_metres.x + member.size_metres.z)
            * f64::from(limits.contact_tolerance_metres);
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
        let mut top = [Vec3::ZERO; 3];
        let mut base = [Vec3::ZERO; 3];
        for (j, point) in points.into_iter().enumerate() {
            let soil = triangle.height_f64(point) as f32;
            let (bottom, head) =
                member_levels(self.member, self.element, soil, self.gate, self.embedment);
            if head <= bottom {
                return Err(BoundarySupportError::new(
                    self.property,
                    self.element,
                    BoundarySupportConstraint::PostHeadroom,
                    point.as_vec2(),
                    f64::from(bottom - head),
                    0.0,
                ));
            }
            top[j] = Vec3::new(point.x as f32, head - self.gate.metres(), point.y as f32);
            base[j] = Vec3::new(point.x as f32, bottom - self.gate.metres(), point.y as f32);
        }
        Ok(BoundarySupportCell {
            positions_metres: [top[0], top[1], top[2], base[0], base[1], base[2]],
            material: self.member.material,
            element: self.element,
        })
    }
}

fn member_levels(
    member: CityBoundaryMember,
    element: BoundarySupportElement,
    soil: f32,
    gate: SupportElevation,
    embedment: FoundationEmbedment,
) -> (f32, f32) {
    let nominal_base = member.centre_metres.y - member.size_metres.y * 0.5;
    let nominal_head = member.centre_metres.y + member.size_metres.y * 0.5;
    match element {
        BoundarySupportElement::GatePost(_) => {
            (soil - embedment.metres(), gate.metres() + nominal_head)
        }
        BoundarySupportElement::Wall(_) => (soil - embedment.metres(), soil + nominal_head),
        BoundarySupportElement::WallCap(_) => (soil + nominal_base, soil + nominal_head),
        BoundarySupportElement::Owner | BoundarySupportElement::GateLanding => {
            unreachable!("fixed member element")
        }
    }
}
