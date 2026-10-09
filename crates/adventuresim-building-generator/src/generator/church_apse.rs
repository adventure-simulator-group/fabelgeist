//! Frozen five-facet apse set-out, including the radial exterior eave.
use super::*;
pub(super) struct ApseRoof<'a> {
    pub walls: Vec<&'a crate::WallAssembly>,
    pub outline: Option<Vec<Vec2>>,
    pub polygons: Vec<Vec<Vec3>>,
}
impl<'a> ApseRoof<'a> {
    pub(super) fn from_source(
        roof: RoofPiece,
        source_piece_index: Option<usize>,
        walls: &'a [crate::WallAssembly],
        shed_high_side: Option<Direction>,
    ) -> Self {
        let mut apse_walls = walls
            .iter()
            .filter(|wall| matches!(wall.source, crate::WallSourceId::ChurchApse { .. }))
            .collect::<Vec<_>>();
        apse_walls.sort_by_key(|wall| match wall.source {
            crate::WallSourceId::ChurchApse { facet } => facet,
            _ => unreachable!(),
        });
        let is_church_apse = source_piece_index == Some(4) && apse_walls.len() == 5;
        let outline: Option<Vec<Vec2>> = is_church_apse.then(|| {
            let first = apse_walls[0];
            let mut points =
                vec![first.frame.origin - first.frame.tangent * first.length_metres * 0.5];
            points.extend(
                apse_walls
                    .iter()
                    .map(|wall| wall.frame.origin + wall.frame.tangent * wall.length_metres * 0.5),
            );
            let diameter_mid = (points[0] + points[points.len() - 1]) * 0.5;
            points
                .into_iter()
                .map(|point| {
                    // The chord wall is 0.90 m thick; a 0.75 m radial eave keeps
                    // the physical gutter outside the masonry even at the acute
                    // five-sided shoulders.  This is a frozen coarse-detail gate,
                    // not a universal historic apse overhang.
                    point + (point - diameter_mid).normalize_or_zero() * roof.eave_metres.max(0.75)
                })
                .collect()
        });
        let polygons = if let Some(outline) = &outline {
            let diameter_mid = (outline[0] + outline[outline.len() - 1]) * 0.5;
            let radius = outline
                .iter()
                .map(|point| point.distance(diameter_mid))
                .fold(0.0_f32, f32::max);
            let apex = Vec3::new(
                diameter_mid.x,
                roof.base_height_metres + radius * roof.pitch_degrees.to_radians().tan(),
                diameter_mid.y,
            );
            outline
                .windows(2)
                .map(|pair| {
                    vec![
                        Vec3::new(pair[0].x, roof.base_height_metres, pair[0].y),
                        Vec3::new(pair[1].x, roof.base_height_metres, pair[1].y),
                        apex,
                    ]
                })
                .collect::<Vec<_>>()
        } else {
            roof_face_polygons(roof, shed_high_side)
        };
        Self {
            walls: apse_walls,
            outline,
            polygons,
        }
    }
}
