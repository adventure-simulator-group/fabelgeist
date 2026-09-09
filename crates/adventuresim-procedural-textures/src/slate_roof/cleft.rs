//! Slaty cleavage: warped terraces, aligned split ledges and edge delamination.
use super::hash_unit;
use crate::TextureParameters;

crate::parameters::parameter_block! {
    pub struct Parameters {
        layer_count: i32 = 2;
        layer_depth: f32 = 0.24;
        layer_edge_width: f32 = 0.10;
        split_direction: f32 = 0.45;
        direction_variation: f32 = 0.80;
        warp_frequency: f32 = 3.0;
        warp_strength: f32 = 0.22;
        fracture_frequency: f32 = 13.0;
        fracture_strength: f32 = 0.06;
        flake_frequency: f32 = 7.0;
        flake_fraction: f32 = 0.20;
        flake_reach: f32 = 0.14;
        flake_depth: f32 = 0.075;
        flake_edge_width: f32 = 0.18;
    }
}

fn smooth(value: f32) -> f32 {
    let x = value.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn noise(params: &TextureParameters, x: f32, y: f32, id: u64) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let random = |dx: i32, dy: i32| {
        hash_unit(
            params,
            id ^ ((ix.wrapping_add(dx) as u32 as u64) << 32) ^ iy.wrapping_add(dy) as u32 as u64,
        )
    };
    let tx = smooth(x - x.floor());
    let ty = smooth(y - y.floor());
    let a = random(0, 0);
    let b = random(1, 0);
    let c = random(0, 1);
    let d = random(1, 1);
    let top = a + (b - a) * tx;
    top + (c + (d - c) * tx - top) * ty
}

pub(super) struct Split {
    pub height: f32,
    pub edge: f32,
}

pub(super) fn sample(
    params: &TextureParameters,
    x: f32,
    y: f32,
    id: u64,
    edge_distance: f32,
) -> Split {
    let p = &params.slate_roof.cleft;
    let angle = p.split_direction + (hash_unit(params, id ^ 0x347b) - 0.5) * p.direction_variation;
    let (sin, cos) = angle.sin_cos();
    let along = x * cos + y * sin;
    let across = -x * sin + y * cos;
    let broad = noise(
        params,
        along * p.warp_frequency,
        across * p.warp_frequency,
        id ^ 0x6ca1,
    );
    let fracture = noise(
        params,
        along * p.fracture_frequency,
        across * p.fracture_frequency,
        id ^ 0x3187,
    ) - 0.5;
    let layers = p.layer_count as f32;
    let level = (across
        + (broad - 0.5) * p.warp_strength
        + fracture * p.fracture_strength
        + hash_unit(params, id ^ 0x459a))
        * layers;
    let phase = level - level.floor();
    // Softening is a geometric ledge bevel bounded below by a texel footprint.
    let texel = params
        .slate_roof
        .courses
        .max(params.slate_roof.pieces_per_course) as f32
        / params.size(super::SLATE_ROOF_TEXTURE_SIZE) as f32;
    let bevel = p.layer_edge_width.max(texel * layers).min(1.0);
    let ledge = smooth((phase - (1.0 - bevel)) / bevel);
    let terrace = (level.floor() + ledge) / layers;
    let flake_noise = noise(
        params,
        along * p.flake_frequency,
        across * p.flake_frequency,
        id ^ 0xe479,
    );
    let reach = (1.0 - edge_distance.max(0.0) / p.flake_reach).clamp(0.0, 1.0);
    let flake =
        smooth((flake_noise - (1.0 - p.flake_fraction)) / p.flake_edge_width) * smooth(reach);
    Split {
        // Warping changes the ledge contour, not the smooth planes between ledges.
        // Removing only the phase offset also keeps it independent of piece thickness.
        height: (terrace - across - hash_unit(params, id ^ 0x459a) + 0.5 / layers) * p.layer_depth
            - flake * p.flake_depth,
        edge: (ledge * (1.0 - ledge) * 4.0).max(flake),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_faces_stay_planar_between_ledge_risers() {
        let p = TextureParameters::default();
        let step = 0.002;
        let mut planar = 0;
        for i in 0..200 {
            let x = (i as f32 + 0.5) / 200.0 - 0.5;
            let y = 0.43;
            let center = sample(&p, x, y, 77, 1.0);
            let left = sample(&p, x - step, y, 77, 1.0);
            let right = sample(&p, x + step, y, 77, 1.0);
            if [center.edge, left.edge, right.edge]
                .into_iter()
                .all(|e| e == 0.0)
            {
                assert!(
                    (left.height + right.height - 2.0 * center.height).abs() < 0.000001,
                    "Coordinate warping must not wrinkle an unbroken cleft face"
                );
                planar += 1;
            }
        }
        assert!(
            planar > 50,
            "There must be broad intact cleft planes: {planar}"
        );
    }

    #[test]
    fn split_layers_are_directional_and_edge_loss_is_local() {
        let p = TextureParameters::default();
        let mut unchipped = p.clone();
        unchipped.slate_roof.cleft.flake_depth = 0.0;
        let mut damage = 0;
        let mut steps = 0;
        for i in 0..300 {
            let x = (i as f32 + 0.5) / 300.0 - 0.5;
            let edge = sample(&p, x, 0.8, 77, 0.0);
            let clean = sample(&unchipped, x, 0.8, 77, 0.0);
            let face = sample(&p, x, 0.8, 77, 1.0);
            let unchipped_face = sample(&unchipped, x, 0.8, 77, 1.0);
            assert!(edge.height <= clean.height);
            assert_eq!(face.height, unchipped_face.height);
            if edge.height < clean.height - 0.001 {
                damage += 1;
            }
            if face.edge > 0.5 {
                steps += 1;
            }
        }
        assert!(damage > 5, "flake coverage: {damage}");
        assert!(steps > 5, "cleft risers: {steps}");
    }
}
