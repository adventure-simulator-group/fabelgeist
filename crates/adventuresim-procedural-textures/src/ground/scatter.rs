//! Stable piece-local density and folded leaf placement.
use super::*;
pub(super) fn litter_leaf_field_with_detail(
    params: &crate::TextureParameters,
    point: Vec2,
    recipe: LitterStratumRecipe,
    detail: LitterDetail,
) -> LitterLeafImprint {
    let scaled = point * recipe.grid as f32;
    let base_cell = scaled.floor().as_ivec2();
    let mut field = LitterLeafImprint {
        order: f32::NEG_INFINITY,
        ..LitterLeafImprint::default()
    };
    for offset_y in -1..=1 {
        for offset_x in -1..=1 {
            let cell = base_cell + IVec2::new(offset_x, offset_y);
            let occupancy = soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x5de3);
            let effective_density = density(params, cell, recipe);
            if occupancy > effective_density {
                continue;
            }
            let centre = cell.as_vec2()
                + Vec2::new(
                    params.ground.litter_leaf_field_with_detail_centre_1
                        + soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x13a7)
                            * params.ground.litter_leaf_field_with_detail_centre_2,
                    params.ground.litter_leaf_field_with_detail_centre_3
                        + soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x91cb)
                            * params.ground.litter_leaf_field_with_detail_centre_4,
                );
            let angle = soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0xc72d)
                * core::f32::consts::TAU;
            let long_axis = Vec2::new(angle.cos(), angle.sin());
            let delta = scaled - centre;
            let local = Vec2::new(delta.dot(long_axis), delta.perp_dot(long_axis));
            let radius = recipe.minimum_radius
                + soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x27f1)
                    * recipe.radius_span;
            let aspect = recipe.minimum_aspect
                + soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0xe419)
                    * recipe.aspect_span;
            let phase = soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x41af)
                * core::f32::consts::TAU;
            let class = litter_shape_class(params, cell, recipe);
            if !litter_shape_visible(class, recipe.stratum, detail) {
                continue;
            }
            let side = (soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x998b)
                - 0.5)
                .signum();
            let shape = sample_litter_shape(params, class, local, radius, aspect, phase, side);
            if shape.coverage <= 0.0 {
                continue;
            }
            let order = shape.dome
                + shape.lift * params.ground.litter_fold_gain
                + soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0xd34f)
                    * params.ground.litter_leaf_field_with_detail_order;
            if order <= field.order {
                continue;
            }
            let pigment = soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0xa531);
            let pigment_tone = (params.ground.litter_leaf_field_with_detail_pigment_tone_1
                - recipe.decomposition
                    * params.ground.litter_leaf_field_with_detail_pigment_tone_2)
                + (pigment - 0.5) * params.ground.litter_leaf_field_with_detail_pigment_tone_3;
            field = LitterLeafImprint {
                coverage: shape.coverage,
                dome: order,
                // Pigment is constant on each leaf. Fold, vein and edge
                // shading belong to height and AO, never painted into albedo.
                tone: (pigment_tone.clamp(0.0, 1.0) * 2.0).round() * 0.5,
                vein: shape.vein,
                #[cfg(test)]
                edge: shape.edge,
                contact: shape.lift * shape.edge,
                order,
            };
        }
    }
    field
}

fn density(params: &crate::TextureParameters, cell: IVec2, recipe: LitterStratumRecipe) -> f32 {
    let decay_pocket = if recipe.stratum == LitterStratum::Lower {
        smoothstep(
            -params.ground.litter_leaf_field_with_detail_decay_pocket_1,
            params.ground.litter_leaf_field_with_detail_decay_pocket_2,
            soil_value_noise(
                params,
                cell.as_vec2() / recipe.grid as f32
                    + Vec2::new(
                        params.ground.litter_leaf_field_with_detail_decay_pocket_3,
                        params.ground.litter_leaf_field_with_detail_decay_pocket_4,
                    ),
                6,
                recipe.salt ^ 0x77a1,
            ),
        )
    } else {
        1.0
    };
    let patch = crate::stamps::noise(
        params,
        cell.as_vec2() / recipe.grid as f32,
        IVec2::splat(3),
        0x7591,
    );
    recipe.density
        * (1.0 - params.ground.litter_patch_strength + patch * params.ground.litter_patch_strength)
        * (params
            .ground
            .litter_leaf_field_with_detail_effective_density_1
            + decay_pocket
                * params
                    .ground
                    .litter_leaf_field_with_detail_effective_density_2)
}
