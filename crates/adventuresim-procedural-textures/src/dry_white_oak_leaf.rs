//! Senescent material treatment for the shared pedunculate-oak leaf form.
//!
//! Drying changes pigment and relief, not the species-defining lobe layout.
//! The silhouette and venation therefore remain owned by `foliage` while this
//! module owns the independently reviewable dry-state response.

pub(super) fn relief(params: &crate::TextureParameters, living_height: f32, u: f32, v: f32) -> f32 {
    let blade_y = 1.0 - v;
    let t = ((blade_y - params.dry_white_oak_leaf.relief_t_1)
        / params.dry_white_oak_leaf.relief_t_2)
        .clamp(0.0, 1.0);
    let axis = params.dry_white_oak_leaf.relief_axis_1
        * (t - params.dry_white_oak_leaf.relief_axis_2).powi(2)
        - params.dry_white_oak_leaf.relief_axis_3;
    let transverse =
        ((u - 0.5 - axis) / params.dry_white_oak_leaf.relief_transverse).clamp(-1.0, 1.0);

    // Broad margin lift and low-amplitude puckering read as a dried lamina at
    // close range while keeping the shared molded-material silhouette intact.
    let edge_curl = transverse
        .abs()
        .powf(params.dry_white_oak_leaf.relief_edge_curl_1)
        * (params.dry_white_oak_leaf.relief_edge_curl_2
            + params.dry_white_oak_leaf.relief_edge_curl_3
                * (t * core::f32::consts::TAU + params.dry_white_oak_leaf.relief_edge_curl_4)
                    .sin());
    let pucker = ((u * params.dry_white_oak_leaf.relief_pucker_1
        + v * params.dry_white_oak_leaf.relief_pucker_2)
        .sin()
        + (u * params.dry_white_oak_leaf.relief_pucker_3
            - v * params.dry_white_oak_leaf.relief_pucker_4
            + params.dry_white_oak_leaf.relief_pucker_5)
            .cos())
        * params.dry_white_oak_leaf.relief_pucker_6;
    (living_height * 0.82 + edge_curl + pucker).clamp(0.012, 0.34)
}

pub(super) fn albedo(
    blade: [u8; 3],
    vein: [u8; 3],
    back_blade: [u8; 3],
    is_vein: bool,
    _tissue_mottle: f32,
) -> ([u8; 3], [u8; 3]) {
    if is_vein {
        return (vein, vein.map(|channel| channel.saturating_add(11)));
    }

    (blade, back_blade)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_relief_is_deterministic_and_bounded() {
        let params = &crate::TextureParameters::default();
        let first = relief(params, 0.21, 0.18, 0.63);
        assert_eq!(first.to_bits(), relief(params, 0.21, 0.18, 0.63).to_bits());
        for y in 0..=32 {
            for x in 0..=32 {
                let height = relief(params, 0.21, x as f32 / 32.0, y as f32 / 32.0);
                assert!((0.012..=0.34).contains(&height));
            }
        }
    }
}

mod controls;
pub use controls::Parameters;
