//! One shape and finish compiler shared by full detail and exterior selection.
use super::*;

pub(crate) struct SolidDetailCompiler<'a> {
    plan: &'a BuildingPlan,
    panel_boundary_edges: BTreeSet<(crate::WallAssemblyId, PanelEdgeKey)>,
}

impl<'a> SolidDetailCompiler<'a> {
    pub(crate) fn new(plan: &'a BuildingPlan) -> Self {
        Self {
            plan,
            panel_boundary_edges: panel_boundary_edges(plan),
        }
    }

    pub(crate) fn compile(&self, solid: &ResolvedSolid) -> BuildingDetail {
        let mut detail = BuildingDetail { meshes: Vec::new() };
        self.append(&mut detail, solid);
        detail
    }

    pub(super) fn append(&self, detail: &mut BuildingDetail, solid: &ResolvedSolid) {
        if stove_tiles::append(detail, self.plan, solid) {
            return;
        }
        if matches!(solid.shape, ResolvedSolidShape::RoundTowerShell { .. }) {
            return;
        }
        let material = material_for_solid(self.plan, solid);
        let wall = wall_for_solid(self.plan, solid);
        if matches!(
            solid.role,
            SolidRole::OpeningJamb
                | SolidRole::OpeningSill
                | SolidRole::OpeningHead
                | SolidRole::OpeningSpandrel
                | SolidRole::OpeningReveal
        ) && self.plan.timber_frame.as_ref().is_some_and(|frame| {
            wall.is_some_and(|wall| frame.bays.iter().any(|bay| bay.wall == Some(wall.id)))
        }) {
            // These are recessed structural bearing solids, not a second
            // visible finish. The resolved infill and timber opening frame
            // already own the exposed Fachwerk surface.
            return;
        }
        match solid.shape {
            ResolvedSolidShape::TimberPanelPrism {
                vertices,
                outward,
                depth_metres,
            } => append_timber_panel(
                detail,
                material,
                vertices,
                outward,
                depth_metres,
                wall,
                std::array::from_fn(|edge| {
                    wall.is_none_or(|wall| {
                        self.panel_boundary_edges.contains(&(
                            wall.id,
                            panel_edge_key(vertices[edge], vertices[(edge + 1) % 3]),
                        ))
                    })
                }),
            ),
            _ => append_shaped_solid(detail, self.plan, material, solid, wall),
        }
    }
}

fn append_shaped_solid(
    detail: &mut BuildingDetail,
    plan: &BuildingPlan,
    material: BuildingLodMaterial,
    solid: &ResolvedSolid,
    wall: Option<&crate::WallAssembly>,
) {
    if matches!(solid.shape, ResolvedSolidShape::CylinderAlongX) {
        detail.meshes.push(crate::axle::mesh(solid));
        return;
    }
    if matches!(solid.shape, ResolvedSolidShape::BellShell) {
        detail.meshes.extend(crate::bell::meshes(solid));
        return;
    }
    if solid.role == SolidRole::LeadedGlazing && matches!(solid.shape, ResolvedSolidShape::Cuboid) {
        append_window_leaf(
            detail,
            plan,
            solid,
            wall,
            crate::WindowLeafKind::LeadedGlass,
        );
        return;
    }
    if solid.role == SolidRole::OpeningClosure
        && plan.opening_assemblies.iter().any(|opening| {
            opening.use_kind == crate::OpeningUse::Window
                && opening.closure_solids.contains(&solid.id)
        })
    {
        append_window_leaf(
            detail,
            plan,
            solid,
            wall,
            crate::WindowLeafKind::TimberShutter,
        );
        return;
    }
    if !arches::append(detail, material, solid, wall) {
        append_oriented_cuboid(detail, material, solid, wall);
    }
}

fn append_window_leaf(
    detail: &mut BuildingDetail,
    plan: &BuildingPlan,
    solid: &ResolvedSolid,
    wall: Option<&crate::WallAssembly>,
    kind: crate::WindowLeafKind,
) {
    let (size, rotation) = wall.map_or(
        (solid.size, Quat::from_rotation_y(solid.yaw_radians)),
        |wall| {
            let tangent = wall.frame.tangent.abs();
            let outward = wall.frame.outward.abs();
            (
                Vec3::new(
                    tangent.x * solid.size.x + tangent.y * solid.size.z,
                    solid.size.y,
                    outward.x * solid.size.x + outward.y * solid.size.z,
                ),
                Quat::from_rotation_y(-wall.frame.tangent.y.atan2(wall.frame.tangent.x)),
            )
        },
    );
    let state = plan
        .opening_assemblies
        .iter()
        .find(|opening| opening.closure_solids.contains(&solid.id))
        .map_or(crate::ClosureState::Closed, |opening| opening.closure.state);
    for mesh in crate::compile_window_leaf(size, kind, state) {
        let target = detail.mesh_mut(mesh.material);
        for indices in mesh.indices.as_chunks::<3>().0 {
            let vertices = indices.map(|index| mesh.vertices[index as usize]);
            target.push_triangle(
                vertices.map(|v| solid.centre + rotation * v.position),
                rotation * vertices[0].normal,
                vertices.map(|v| v.uv),
            );
        }
    }
}

type PanelEdgeKey = ([i64; 3], [i64; 3]);

fn panel_edge_key(first: Vec3, second: Vec3) -> PanelEdgeKey {
    let quantize = |point: Vec3| {
        [
            (point.x * 100_000.0).round() as i64,
            (point.y * 100_000.0).round() as i64,
            (point.z * 100_000.0).round() as i64,
        ]
    };
    let first = quantize(first);
    let second = quantize(second);
    if first <= second {
        (first, second)
    } else {
        (second, first)
    }
}

fn panel_boundary_edges(plan: &BuildingPlan) -> BTreeSet<(crate::WallAssemblyId, PanelEdgeKey)> {
    let mut counts = BTreeMap::new();
    for wall in &plan.wall_assemblies {
        for solid in plan
            .resolved_geometry
            .solids
            .iter()
            .filter(|solid| wall.host_solids.contains(&solid.id))
        {
            let ResolvedSolidShape::TimberPanelPrism { vertices, .. } = solid.shape else {
                continue;
            };
            for edge in 0..3 {
                *counts
                    .entry((
                        wall.id,
                        panel_edge_key(vertices[edge], vertices[(edge + 1) % 3]),
                    ))
                    .or_insert(0_u8) += 1;
            }
        }
    }
    counts
        .into_iter()
        .filter_map(|(edge, count)| (count == 1).then_some(edge))
        .collect()
}

fn append_timber_panel(
    detail: &mut BuildingDetail,
    material: BuildingLodMaterial,
    vertices: [Vec3; 3],
    outward: Vec2,
    depth_metres: f32,
    wall: Option<&crate::WallAssembly>,
    boundary_edges: [bool; 3],
) {
    let normal = Vec3::new(outward.x, 0.0, outward.y).normalize_or_zero();
    let offset = normal * depth_metres * 0.5;
    let front = vertices.map(|point| point + offset);
    let back = vertices.map(|point| point - offset);
    let tangent = Vec3::new(-normal.z, 0.0, normal.x);
    let uv = |point: Vec3| {
        wall.map_or_else(
            || Vec2::new(point.dot(tangent), point.y) / BUILDING_DETAIL_UV_METRES_PER_UNIT,
            |wall| wall_surface_uv(wall, point),
        )
    };
    detail
        .mesh_mut(material)
        .push_triangle(front, normal, front.map(uv));
    let back_positions = [back[2], back[1], back[0]];
    detail
        .mesh_mut(interior_face_material(material, wall, -normal))
        .push_triangle(back_positions, -normal, back_positions.map(uv));
    for edge in 0..3 {
        if !boundary_edges[edge] {
            continue;
        }
        let next = (edge + 1) % 3;
        let positions = [front[edge], back[edge], back[next], front[next]];
        let side_normal = (back[edge] - front[edge])
            .cross(front[next] - front[edge])
            .normalize_or_zero();
        let edge_length = front[edge].distance(front[next]);
        detail.mesh_mut(material).push_quad(
            positions,
            side_normal,
            [
                Vec2::ZERO,
                Vec2::new(depth_metres, 0.0),
                Vec2::new(depth_metres, edge_length),
                Vec2::new(0.0, edge_length),
            ]
            .map(|uv| uv / BUILDING_DETAIL_UV_METRES_PER_UNIT),
        );
    }
}
