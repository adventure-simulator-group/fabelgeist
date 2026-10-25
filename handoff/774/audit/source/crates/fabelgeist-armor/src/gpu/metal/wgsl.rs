//! WGSL shared by the metal kernels.

use fabelgeist_compute::host_float;

/// Constants, correctly rounded arithmetic and `normalized`. A kernel that
/// includes it has a uniform `params.zero` of zero; see [`host_float`].
pub(super) fn math() -> String {
    format!(
        "{CONSTANTS}{}{}{NORMALIZED}",
        host_float::PARAMS_ZERO_HOOK,
        host_float::wgsl()
    )
}

const CONSTANTS: &str = r#"
const PI: f32 = 3.14159265358979;
const FRAC_PI_2: f32 = 1.57079632679490;
"#;

const NORMALIZED: &str = r#"
// `Vec3::normalize`: a zero vector stays zero.
fn normalized(v: vec3<f32>) -> vec3<f32> {
    let norm = host_length(v);
    if (norm > 0.0) {
        return host_div3(v, norm);
    }
    return v;
}
"#;

/// A linear congruential generator, with a jump ahead so that any
/// draw of the sequence can be taken without drawing the ones before it.
pub(super) const RANDOM: &str = r#"
const LCG_MULTIPLIER: u32 = 1664525u;
const LCG_INCREMENT: u32 = 1013904223u;
// The draw keeps the state's top 24 bits, as a fraction of one.
const DRAW_SCALE: f32 = 16777216.0;

// The state after `steps` draws from `seed`: the affine step composed with
// itself by squaring, in wrapping 32-bit arithmetic.
fn lcg_skip(seed: u32, steps: u32) -> u32 {
    var multiplier = LCG_MULTIPLIER;
    var increment = LCG_INCREMENT;
    var total_multiplier = 1u;
    var total_increment = 0u;
    var remaining = steps;
    loop {
        if (remaining == 0u) {
            break;
        }
        if ((remaining & 1u) != 0u) {
            total_multiplier = total_multiplier * multiplier;
            total_increment = total_increment * multiplier + increment;
        }
        increment = increment * multiplier + increment;
        multiplier = multiplier * multiplier;
        remaining = remaining >> 1u;
    }
    return seed * total_multiplier + total_increment;
}

fn lcg_next(state: u32) -> u32 {
    return state * LCG_MULTIPLIER + LCG_INCREMENT;
}

fn draw(state: u32) -> f32 {
    return f32(state >> 8u) / DRAW_SCALE;
}
"#;
