//! Civilian skyline geometry directly from the compact construction programme.
//! No room allocation, structural members, collision, or solid compiler runs.
use super::*;
use crate::{BuildingArchetype, BuildingProgram, Footprint, RoofPiece};

const WINDOW_SPACING_METRES: f32 = 3.0;
const WINDOW_WIDTH_METRES: f32 = 0.8;
const WINDOW_HEIGHT_METRES: f32 = 1.2;
const WINDOW_SILL_METRES: f32 = 0.9;

/// Common rectangular civilian shells use recipe geometry. Other architectural
/// families require their specialized semantic shell compiler.
pub fn compile_program_shell(program: &BuildingProgram) -> Option<BuildingLod> {
    if !matches!(
        program.archetype,
        BuildingArchetype::TownHouse
            | BuildingArchetype::HallHouse
            | BuildingArchetype::FachwerkCottage
            | BuildingArchetype::FachwerkMerchantHouse
            | BuildingArchetype::StorageRange
    ) || !matches!(program.footprint, Footprint::Rectangle { .. })
        || program.church_program.is_some()
        || program.roof_demonstrator.is_some()
    {
        return None;
    }
    let (width, depth) = program.footprint.dimensions();
    let size = Vec2::new(f32::from(width), f32::from(depth)) * crate::CELL_SIZE_METRES;
    let mut lod = BuildingLod {
        level: BuildingLodLevel::Shell,
        facade_runs: Vec::new(),
        meshes: Vec::new(),
    };
    for level in 0..program.storeys.len() {
        let (material, _, thickness) =
            crate::generator::wall_material_and_thickness(program.archetype, true, level as u16);
        let expansion = thickness * 0.5
            + if level > 0 {
                program.upper_storey_projection_metres
            } else {
                0.0
            };
        let min = Vec2::splat(-expansion);
        let max = size + Vec2::splat(expansion);
        let points = [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)];
        let bottom = level as f32 * program.storey_height_metres;
        let top = bottom + program.storey_height_metres;
        for edge in 0..points.len() {
            let start = points[edge];
            let end = points[(edge + 1) % points.len()];
            let tangent = (end - start).normalize();
            let outward = Vec3::new(tangent.y, 0.0, -tangent.x);
            let positions = [
                plan_vertex(start, bottom),
                plan_vertex(end, bottom),
                plan_vertex(end, top),
                plan_vertex(start, top),
            ];
            lod.mesh_mut(BuildingLodMaterial::Wall(material)).push_quad(
                positions,
                outward,
                positions.map(|p| {
                    Vec2::new(p.x * tangent.x + p.z * tangent.y, p.y) / TEXTURE_REPEAT_METRES
                }),
            );
            windows(&mut lod, start, end, bottom, outward);
        }
    }
    roofs(&mut lod, RoofPiece::civilian(program));
    Some(lod)
}

fn windows(lod: &mut BuildingLod, start: Vec2, end: Vec2, bottom: f32, outward: Vec3) {
    let length = start.distance(end);
    let count = (length / WINDOW_SPACING_METRES).floor() as usize;
    let tangent = (end - start).normalize();
    for index in 0..count {
        let centre = start.lerp(end, (index as f32 + 0.5) / count as f32);
        let a = centre - tangent * WINDOW_WIDTH_METRES * 0.5;
        let b = centre + tangent * WINDOW_WIDTH_METRES * 0.5;
        let sill = bottom + WINDOW_SILL_METRES;
        let head = sill + WINDOW_HEIGHT_METRES;
        let positions = [
            plan_vertex(a, sill),
            plan_vertex(b, sill),
            plan_vertex(b, head),
            plan_vertex(a, head),
        ]
        .map(|p| p + outward * FACADE_DETAIL_OFFSET_METRES);
        let (u0, u1) = super::details::opening_atlas_interval(crate::OpeningUse::Window);
        lod.mesh_mut(BuildingLodMaterial::FacadeDetails).push_quad(
            positions,
            outward,
            [
                Vec2::new(u0, 0.0),
                Vec2::new(u1, 0.0),
                Vec2::new(u1, 1.0),
                Vec2::new(u0, 1.0),
            ],
        );
    }
}

fn roofs(lod: &mut BuildingLod, roof: RoofPiece) {
    let faces = crate::generator::roof_face_polygons(roof, None);
    let mut edges = Vec::new();
    for polygon in &faces {
        let normal = (polygon[1] - polygon[0])
            .cross(polygon[2] - polygon[0])
            .normalize();
        append_polygon(
            lod,
            BuildingLodMaterial::Roof(RoofMaterial::ClayTile),
            polygon,
            if normal.y < 0.0 { -normal } else { normal },
        );
        for index in 0..polygon.len() {
            edges.push((polygon[index], polygon[(index + 1) % polygon.len()]));
        }
    }
    for &(a, b) in &edges {
        if a.y.max(b.y) <= roof.base_height_metres
            || edges
                .iter()
                .filter(|&&(c, d)| (a == c && b == d) || (a == d && b == c))
                .count()
                != 1
        {
            continue;
        }
        let centre = (a + b) * 0.5;
        let direction = Vec3::new(centre.x - roof.centre.x, 0.0, centre.z - roof.centre.y);
        let normal = (b - a).cross(Vec3::Y).normalize_or_zero();
        let outward = if normal.dot(direction) < 0.0 {
            -normal
        } else {
            normal
        };
        let bottom = |p: Vec3| Vec3::new(p.x, roof.base_height_metres, p.z);
        append_polygon(
            lod,
            BuildingLodMaterial::Wall(WallMaterialClass::TimberInfill),
            &[bottom(a), bottom(b), b, a],
            outward,
        );
    }
}

#[cfg(test)]
mod tests;

fn append_polygon(
    lod: &mut BuildingLod,
    material: BuildingLodMaterial,
    points: &[Vec3],
    normal: Vec3,
) {
    for index in 1..points.len() - 1 {
        let positions = [points[0], points[index], points[index + 1]];
        if (positions[1] - positions[0])
            .cross(positions[2] - positions[0])
            .length_squared()
            <= f32::EPSILON
        {
            continue;
        }
        let triangle = crate::RoofSurfaceTriangle {
            positions,
            normal,
            surface: crate::RoofSurface::Weather,
        };
        let uv = if normal.y.abs() > f32::EPSILON {
            triangle.covering_uvs(TEXTURE_REPEAT_METRES)
        } else {
            triangle.planar_uvs(TEXTURE_REPEAT_METRES)
        };
        lod.mesh_mut(material).push_triangle(positions, normal, uv);
    }
}
