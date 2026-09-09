use super::*;
/// Periodic oak relief with continuous longitudinal furrows. A shared plate
/// field contributes only smoothly blended crown variation; subordinate checks
/// begin at major furrows and taper before they can outline closed cells.
pub(crate) fn oak_bark_height(params: &crate::TextureParameters, u: f32, v: f32) -> f32 {
    let tau = core::f32::consts::TAU;
    let point = Vec2::new(u.rem_euclid(1.0), v.rem_euclid(1.0));
    let (crown_height, tilt, fracture_phase) = crown(params, point);

    let (graph_normalized, graph_width, graph_depth) = oak_bark_graph_distance(params, point);
    let graph_core = (-graph_normalized.powi(4)).exp();
    let graph_valley = (-0.5 * (graph_normalized / 3.0).powi(2)).exp();
    let graph_shoulder = (params.surface.oak_bark_height_graph_shoulder_1
        + params.surface.oak_bark_height_graph_shoulder_2 * graph_depth)
        * (-0.5
            * ((graph_normalized - params.surface.oak_bark_height_graph_shoulder_3)
                / params.surface.oak_bark_height_graph_shoulder_4)
                .powi(2))
        .exp();
    let graph_relief = graph_depth
        * (-params.surface.oak_bark_height_graph_relief_1 * graph_core
            - params.surface.oak_bark_height_graph_relief_2 * graph_valley);
    let physical_graph_distance = graph_normalized * graph_width;
    let face_mask = smoothstep(
        params.surface.oak_bark_height_face_mask_1,
        params.surface.oak_bark_height_face_mask_2,
        physical_graph_distance,
    );
    let asymmetric_crown = (params.surface.oak_bark_height_asymmetric_crown_1
        + params.surface.oak_bark_height_asymmetric_crown_2 * fracture_phase)
        * smoothstep(
            params.surface.oak_bark_height_asymmetric_crown_3,
            params.surface.oak_bark_height_asymmetric_crown_4,
            physical_graph_distance,
        )
        * (params.surface.oak_bark_height_asymmetric_crown_5
            + params.surface.oak_bark_height_asymmetric_crown_6
                * (tau
                    * (point.y * params.surface.oak_bark_height_asymmetric_crown_7
                        + fracture_phase))
                    .sin());
    let (check_distance, check_taper) = oak_bark_check_distance(params, point);
    let check_relief = -params.surface.oak_bark_height_check_relief_1
        * (-(check_distance / params.surface.oak_bark_height_check_relief_2).powi(4)).exp()
        * check_taper;
    let (fiber_distance, fiber_envelope) = oak_bark_fiber_distance(params, point);
    let fiber_relief = -params.surface.oak_bark_height_fiber_relief_1
        * (-(fiber_distance / params.surface.oak_bark_height_fiber_relief_2).powi(2)).exp()
        * fiber_envelope
        * face_mask;
    // Irregular face breakup avoids directional sine bands across the plates.
    let broad_breakup = params.surface.oak_bark_height_broad_breakup_1
        * (crate::stamps::noise(
            params,
            point,
            bevy::math::IVec2::new(
                params
                    .surface
                    .oak_bark_height_broad_breakup_2
                    .round()
                    .max(1.0) as i32,
                params
                    .surface
                    .oak_bark_height_broad_breakup_3
                    .round()
                    .max(1.0) as i32,
            ),
            0xa915,
        ) - 0.5)
        * face_mask;
    let fine_breakup = params.surface.oak_bark_height_fine_breakup_1
        * (crate::stamps::noise(
            params,
            point,
            bevy::math::IVec2::new(
                params
                    .surface
                    .oak_bark_height_fine_breakup_2
                    .round()
                    .max(1.0) as i32,
                params
                    .surface
                    .oak_bark_height_fine_breakup_4
                    .round()
                    .max(1.0) as i32,
            ),
            0xd673,
        ) - 0.5)
        * face_mask;
    let fissure_relief = graph_relief.min(check_relief);

    (crown_height
        + tilt
        + asymmetric_crown
        + graph_shoulder
        + fissure_relief
        + fiber_relief
        + broad_breakup
        + fine_breakup
        + params.surface.plates.sample(params, point, 0x7319))
    .clamp(-0.5, 0.32)
}

fn crown(params: &crate::TextureParameters, point: Vec2) -> (f32, f32, f32) {
    let mut weight_sum = 0.0;
    let mut crown_height = 0.0;
    let mut tilt = 0.0;
    let mut fracture_phase = 0.0;
    // The coherent shared metric is sampled with a smooth compact-looking
    // kernel over every site. No nearest-site rank switch can introduce an
    // ownership seam into crown, tilt, or fracture phase.
    for index in 0..params.surface.oak_bark_plate_count {
        let site = oak_bark_plate_site(params, index);
        let offset = oak_bark_toroidal_offset(point, site.position);
        let distance = (offset.x.powi(2)
            + (offset.y * params.surface.oak_bark_height_distance).powi(2))
        .sqrt();
        let weight = (-(distance / params.surface.oak_bark_height_weight_1).powi(4)).exp()
            + params.surface.oak_bark_height_weight_2;
        let id = site.id;
        weight_sum += weight;
        crown_height += weight * (0.070 + 0.110 * oak_bark_site_value(params, id, 0x61e3));
        tilt += weight
            * ((oak_bark_site_value(params, id, 0x19d7) - 0.5) * offset.x * 0.30
                + (oak_bark_site_value(params, id, 0x2d91) - 0.5) * offset.y * 0.13);
        fracture_phase += weight * oak_bark_site_value(params, id, 0x8d31);
    }
    crown_height /= weight_sum;
    tilt /= weight_sum;
    fracture_phase /= weight_sum;

    (crown_height, tilt, fracture_phase)
}
