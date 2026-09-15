//! Lead cames follow the same polygonal crown as the glazing panel.
use super::*;

const PANE_PITCH_METRES: f32 = 0.19;
const CAME_HALF_WIDTH_METRES: f32 = 0.004;
const CAME_PROJECTION_METRES: f32 = 0.003;

pub(super) fn append(detail: &mut BuildingDetail, strips: &[crate::arch_geometry::ArchStrip]) {
    let Some(first) = strips.first() else {
        return;
    };
    let origin = first.front[0];
    let tangent = (first.front[1] - origin).normalize();
    let normal = -first.depth.normalize();
    let width = (strips.last().unwrap().front[1] - origin).dot(tangent);
    let crown = strips
        .iter()
        .map(|strip| {
            (
                (strip.front[0] - origin).dot(tangent),
                (strip.front[1] - origin).dot(tangent),
                strip.front[3].y - origin.y,
                strip.front[2].y - origin.y,
            )
        })
        .collect::<Vec<_>>();
    let height_at = |x: f32| {
        crown
            .iter()
            .find(|&&(a, b, _, _)| x >= a && x <= b)
            .map_or(0.0, |&(a, b, ya, yb)| ya + (yb - ya) * (x - a) / (b - a))
    };
    let mut rectangles = Vec::<(f32, f32, f32, f32)>::new();
    let columns = (width / PANE_PITCH_METRES).ceil() as usize;
    for column in 1..columns {
        let x = width * column as f32 / columns as f32;
        let lo = x - CAME_HALF_WIDTH_METRES;
        let hi = x + CAME_HALF_WIDTH_METRES;
        let height = height_at(lo).min(height_at(hi));
        if height > 0.0 {
            rectangles.push((lo, hi, 0.0, height));
        }
    }
    let height = crown
        .iter()
        .map(|&(_, _, a, b)| a.max(b))
        .fold(0.0_f32, f32::max);
    let rows = (height / PANE_PITCH_METRES).ceil() as usize;
    for row in 1..rows {
        let y = height * row as f32 / rows as f32;
        let cap = y + CAME_HALF_WIDTH_METRES;
        let mut left = f32::INFINITY;
        let mut right = f32::NEG_INFINITY;
        for &(a, b, ya, yb) in &crown {
            if ya.max(yb) < cap {
                continue;
            }
            let crossing = if ya == yb {
                a
            } else {
                a + (b - a) * (cap - ya) / (yb - ya)
            };
            left = left.min(if ya >= cap { a } else { crossing });
            right = right.max(if yb >= cap { b } else { crossing });
        }
        if right > left {
            rectangles.push((left, right, y - CAME_HALF_WIDTH_METRES, cap));
        }
    }
    let mesh = detail.mesh_mut(BuildingLodMaterial::LeadAlloy);
    for (offset, outward) in [
        (normal * CAME_PROJECTION_METRES, normal),
        (first.depth - normal * CAME_PROJECTION_METRES, -normal),
    ] {
        for &(left, right, bottom, top) in &rectangles {
            let point = |x, y| origin + tangent * x + Vec3::Y * y + offset;
            let mut face = [
                point(left, bottom),
                point(right, bottom),
                point(right, top),
                point(left, top),
            ];
            if (face[1] - face[0]).cross(face[2] - face[0]).dot(outward) < 0.0 {
                face.reverse();
            }
            mesh.push_quad(
                face,
                outward,
                face.map(|p| Vec2::new(p.dot(tangent), p.y) / BUILDING_DETAIL_UV_METRES_PER_UNIT),
            );
        }
    }
}
