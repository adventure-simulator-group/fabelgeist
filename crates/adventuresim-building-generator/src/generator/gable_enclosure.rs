//! Gable profiles are sections of the resolved covering at the wall envelope.
use bevy::math::{Vec2, Vec3};

use crate::{
    ROOF_ENCLOSURE_THICKNESS_METRES, RidgeAxis, RoofEnclosureFace, RoofFace, RoofPiece,
    WallAssembly, WallSourceId,
};

const WALL_TOP_TOLERANCE_METRES: f32 = 0.01;

pub(super) fn polygons(
    roof: RoofPiece,
    faces: &[RoofFace],
    walls: &[WallAssembly],
) -> [Vec<Vec3>; 2] {
    let (along, across, half_length, half_width) = match roof.ridge_axis {
        RidgeAxis::Z => (Vec2::Y, Vec2::X, roof.size.y * 0.5, roof.size.x * 0.5),
        RidgeAxis::X => (Vec2::X, Vec2::Y, roof.size.x * 0.5, roof.size.y * 0.5),
    };
    [-1.0, 1.0].map(|sign| {
        let outward = along * sign;
        let nominal = roof.centre + outward * half_length;
        let hosts = walls
            .iter()
            .filter(|wall| {
                matches!(wall.source, WallSourceId::StoreyWall { .. })
                    && wall.frame.outside_room.is_none()
                    && wall.frame.outward.dot(outward) > 0.99
                    && (wall.base_elevation_metres + wall.height_metres - roof.base_height_metres)
                        .abs()
                        < WALL_TOP_TOLERANCE_METRES
                    && (wall.frame.origin - nominal).dot(along).abs() < roof.eave_metres
                    && (wall.frame.origin - nominal).dot(across).abs() < half_width
            })
            .collect::<Vec<_>>();
        let mut centre = nominal;
        let mut low = -half_width;
        let mut high = half_width;
        if let Some(host) = hosts.first() {
            centre += outward
                * ((host.frame.origin - nominal).dot(outward)
                    + (host.thickness_metres * 0.5).min(ROOF_ENCLOSURE_THICKNESS_METRES * 0.5));
            low = hosts
                .iter()
                .map(|wall| {
                    (wall.frame.origin - roof.centre).dot(across)
                        - wall.length_metres * 0.5
                        - wall.thickness_metres * 0.5
                })
                .fold(f32::INFINITY, f32::min);
            high = hosts
                .iter()
                .map(|wall| {
                    (wall.frame.origin - roof.centre).dot(across)
                        + wall.length_metres * 0.5
                        + wall.thickness_metres * 0.5
                })
                .fold(f32::NEG_INFINITY, f32::max);
        }
        let point = |u: f32, y: f32| {
            let p = centre + across * u;
            Vec3::new(p.x, y, p.y)
        };
        let top = faces
            .iter()
            .flat_map(|face| &face.polygon)
            .map(|p| p.y)
            .fold(roof.base_height_metres, f32::max);
        let mut polygon = vec![
            point(low, roof.base_height_metres),
            point(low, top),
            point(high, top),
            point(high, roof.base_height_metres),
        ];
        for face in faces {
            polygon = clip_to_underside(&polygon, face);
        }
        polygon.dedup_by(|a, b| a.distance_squared(*b) < 1.0e-8);
        if polygon.len() < 3 {
            return polygon;
        }
        let normal = (polygon[1] - polygon[0]).cross(polygon[2] - polygon[0]);
        if Vec2::new(normal.x, normal.z).dot(outward) < 0.0 {
            polygon.reverse();
        }
        polygon
    })
}

fn clip_to_underside(polygon: &[Vec3], face: &RoofFace) -> Vec<Vec3> {
    let mut clipped = Vec::new();
    let signed = |p: Vec3| {
        face.plane.normal.dot(p)
            + face.plane.constant
            + face.thickness_metres * face.plane.normal.length()
    };
    for (&a, &b) in polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
    {
        let da = signed(a);
        let db = signed(b);
        if da <= 0.0 {
            clipped.push(a);
        }
        if (da < 0.0 && db > 0.0) || (da > 0.0 && db < 0.0) {
            clipped.push(a.lerp(b, da / (da - db)));
        }
    }
    clipped
}

pub(super) fn update_pitch(
    enclosures: &mut [RoofEnclosureFace],
    faces: &[RoofFace],
    recipe: Option<RoofPiece>,
    walls: &[WallAssembly],
    min_y: f32,
    scale_y: impl Fn(f32) -> f32,
) {
    if let Some(recipe) = recipe {
        for (enclosure, polygon) in enclosures.iter_mut().zip(polygons(recipe, faces, walls)) {
            enclosure.polygon = polygon;
        }
    } else {
        for enclosure in enclosures {
            for point in &mut enclosure.polygon {
                if point.y > min_y + WALL_TOP_TOLERANCE_METRES {
                    point.y = scale_y(point.y);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GableProfile, ResolvedItemId, RoofEnclosureFace, RoofKind, RoofMaterial};

    #[test]
    fn enclosure_sections_match_both_ridge_axes_across_pitch_and_overhang() {
        for ridge_axis in [RidgeAxis::X, RidgeAxis::Z] {
            for pitch_degrees in [15.0, 35.0, 57.0, 75.0] {
                for eave_metres in [0.0, 0.2, 0.55] {
                    let recipe = RoofPiece {
                        kind: RoofKind::Gable,
                        centre: Vec2::ZERO,
                        size: Vec2::new(8.0, 12.0),
                        base_height_metres: 6.0,
                        pitch_degrees,
                        ridge_axis,
                        eave_metres,
                        gable_profile: GableProfile::Plain,
                    };
                    let faces = super::super::roof_face_polygons(recipe, None)
                        .into_iter()
                        .map(|polygon| RoofFace {
                            id: ResolvedItemId(1),
                            plane: super::super::roof_plane(&polygon),
                            polygon,
                            cutouts: Vec::new(),
                            pitch_degrees,
                            thickness_metres: 0.13,
                            material: RoofMaterial::ClayTile,
                            support_nodes: Vec::new(),
                            drainage_catchment: ResolvedItemId(2),
                        })
                        .collect::<Vec<_>>();
                    for polygon in polygons(recipe, &faces, &[]) {
                        assert!(polygon.len() >= 3);
                        for point in polygon
                            .iter()
                            .filter(|p| p.y > recipe.base_height_metres + 0.001)
                        {
                            assert!(
                                faces.iter().any(|face| {
                                    (face.plane.normal.dot(*point)
                                        + face.plane.constant
                                        + face.thickness_metres)
                                        .abs()
                                        < 0.0001
                                }),
                                "detached gable vertex {point:?}"
                            );
                        }
                        let triangles = crate::tessellate_roof_enclosure(&RoofEnclosureFace {
                            id: ResolvedItemId(3),
                            polygon,
                            material: RoofMaterial::TimberInfill,
                            support_nodes: Vec::new(),
                        });
                        let positions = triangles
                            .iter()
                            .flat_map(|t| t.positions.map(|p| p.to_array()))
                            .collect::<Vec<_>>();
                        let indices = (0..positions.len() as u32).collect::<Vec<_>>();
                        assert!(
                            crate::audit_triangle_mesh(&positions, &indices).passes_closed_solid()
                        );
                    }
                }
            }
        }
    }
}
