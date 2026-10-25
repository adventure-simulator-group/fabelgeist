//! Regression checks against height slopes and Bevy's actual generated tangent frame.
use super::*;
use adventuresim_procedural_textures::{
    BakeResolution, BakedMap, BakedRecipe, MapChannel, PROCEDURAL_TEXTURE_CATALOGUE,
    TextureRecipeId,
};
use bevy::mesh::VertexAttributeValues;

#[test]
fn procedural_normals_light_raised_edges_from_above_and_left() {
    let parameters = adventuresim_procedural_textures::TextureParameters {
        resolution: BakeResolution::Draft,
        ..default()
    };
    let mut failures = Vec::new();
    let mut inspected = 0;
    for descriptor in PROCEDURAL_TEXTURE_CATALOGUE {
        let bake = BakedRecipe::generate(descriptor.id, &parameters);
        let Some(height) = bake.map(MapChannel::Height) else {
            continue;
        };
        let Some(normal) = bake
            .map(MapChannel::Normal)
            .or_else(|| bake.map(MapChannel::FrontNormal))
        else {
            continue;
        };
        let mesh = geometry::mesh(&bake, &crate::document::View::default());
        let Some(VertexAttributeValues::Float32x4(tangents)) =
            mesh.attribute(Mesh::ATTRIBUTE_TANGENT)
        else {
            panic!("missing tangents")
        };
        let t = Vec4::from_array(tangents[0]);
        let frame = Mat3::from_cols(t.truncate(), Vec3::Z.cross(t.truncate()) * t.w, Vec3::Z);
        let filtered = matches!(
            descriptor.id,
            TextureRecipeId::WattleAndDaub
                | TextureRecipeId::ClayRoofTile
                | TextureRecipeId::SlateRoof
                | TextureRecipeId::TimberShingle
        );
        let agreement = slope_agreement(
            height,
            normal,
            bake.map(MapChannel::Opacity),
            frame,
            filtered,
        );
        println!(
            "{}: left/top slope agreement {agreement:?}",
            descriptor.id.slug()
        );
        if agreement.min_element() < 0.99 {
            failures.push((descriptor.id, agreement));
        }
        inspected += 1;
    }
    assert_eq!(
        inspected, 19,
        "cover every surface and leaf height/normal pair"
    );
    assert!(
        failures.is_empty(),
        "normal/height disagreement: {failures:?}"
    );
}

fn slope_agreement(
    height: &BakedMap,
    normal: &BakedMap,
    opacity: Option<&BakedMap>,
    frame: Mat3,
    filtered: bool,
) -> Vec2 {
    let size = height.size as usize;
    assert_eq!(normal.size, height.size);
    let heights = reference_heights(height, filtered);
    let mut signed = Vec2::ZERO;
    let mut absolute = Vec2::ZERO;
    for y in 1..size - 1 {
        for x in 1..size - 1 {
            let i = y * size + x;
            let neighbors = [i - 1, i + 1, i - size, i + size];
            if opacity.is_some_and(|mask| neighbors.iter().any(|j| mask.bytes[j * 4] < 128)) {
                continue;
            }
            let h = |j: usize| heights[j];
            let delta = Vec2::new(h(i + 1) - h(i - 1), h(i + size) - h(i - size));
            let p = &normal.bytes[i * 4..i * 4 + 3];
            let n = frame * (Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32) / 127.5 - Vec3::ONE);
            let towards_light = Vec2::new(-n.x, n.y);
            for axis in 0..2 {
                // Ignore byte quantization and flat areas; only measurable slopes carry direction.
                if delta[axis].abs() <= 1.0 {
                    continue;
                }
                let product = delta[axis] * towards_light[axis];
                signed[axis] += product;
                absolute[axis] += product.abs();
            }
        }
    }
    assert!(absolute.min_element() > 0.0, "no measurable relief");
    signed / absolute
}

// The direction oracle uses the independently exported physical height. For
// footprint-filtered normals, compare equal spatial support: a raw thin ridge
// can slope the other way inside the surrounding one-texel filtered ridge.
fn reference_heights(height: &BakedMap, filtered: bool) -> Vec<f32> {
    let size = height.size as i32;
    let at = |x: i32, y: i32| {
        height.bytes[((y.rem_euclid(size) * size + x.rem_euclid(size)) * 4) as usize] as f32
    };
    let weights = [1.0, 4.0, 6.0, 4.0, 1.0];
    (0..size)
        .flat_map(|y| {
            (0..size).map(move |x| {
                if !filtered {
                    return at(x, y);
                }
                let mut sum = 0.0;
                for (dy, wy) in weights.iter().enumerate() {
                    for (dx, wx) in weights.iter().enumerate() {
                        sum += at(x + dx as i32 - 2, y + dy as i32 - 2) * wx * wy / 256.0;
                    }
                }
                sum
            })
        })
        .collect()
}
